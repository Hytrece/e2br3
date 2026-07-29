# AS2 Submitter — FDA Transport Design

## Purpose

Make the AS2 submitter capable of a complete FDA ESG NextGen submission round
trip for E2B(R3) ICSRs on the CDER/AERS path:

1. Send a signed and encrypted submission with correct FDA routing.
2. Receive and verify the MDN, and report it to the backend as ACK1.
3. **Receive the ACK3 `.ack` file that FDA pushes back over AS2**, and report it
   to the backend as ACK3.

Step 3 is the reason this work exists. FDA delivers acknowledgements to the
submitter's own AS2 gateway, so the submitter must be an AS2 *receiver*, not
only a sender. Today it is only a sender.

Requirements are taken from the FDA *ESG NextGen AS2 Guide for Industry Users*
v2.1 and v2.2, the *ESG NextGen Gateway Configuration Information* form, and the
ESG NextGen Submission Acknowledgements page. The extracted requirements are
recorded in
[`docs/superpowers/research/2026-07-27-as2-official-requirements.md`](../research/2026-07-27-as2-official-requirements.md).

## Current State

The submitter lives outside this repository at `~/projects/java/as2-submitter`
(Java 21, Maven, BouncyCastle, `com.sun.net.httpserver`). It is a single
`Main.java` of 1083 lines on branch `main`, with an uncommitted working tree
holding the Dockerfile, compose file and scripts.

What works today:

- `/health`, `/internal/status`, `POST /submit`, `POST /callbacks/ack` on
  `127.0.0.1:9090`.
- The HTTP contract that `crates/services/web-server/src/submission/gateway.rs`
  already expects, in both directions.
- Idempotent submit, file-backed state, ACK forward retry queue with backoff.
- CMS sign (SHA256withRSA) and encrypt (AES256-CBC) via BouncyCastle.
- Two transport modes: `openas2` (writes payload files into an OpenAS2 outbox,
  the current default) and `direct` (posts AS2 itself, documented as fallback).

A companion OpenAS2 4.8.0 mock receiver exists at
`~/projects/java/mock-mfds-openas2`. It has real received traffic from
2026-03-03: two payloads in `data/inbox/MFDS_MOCK/` and two signed MDNs with
matching MICs. The captured request headers show `Content-Type: application/xml`
— those runs were **plaintext**, so the crypto path has never been exercised
end to end.

Gaps against the FDA specification:

| Gap | Detail |
| --- | --- |
| No AS2 receiver | Nothing decrypts an inbound AS2 message, verifies it, or returns a signed MDN. `/callbacks/ack` accepts JSON only. |
| No FDA routing | Only a flat `AS2_FDA_TO_ID`. None of the three documented routing options exist. |
| Synchronous MDN only | No `Receipt-Delivery-Option`, so asynchronous MDNs cannot be received. |
| MDN signature unverified | `parseMdn` lowercases the body and does string matching. No CMS verification. |
| Weak receipt request | `Disposition-Notification-Options` asks for `signed-receipt-protocol=optional`. A partner that does not sign still passes. |
| Wrong ACK model | Levels 1–4 are treated uniformly. On the AERS path ACK2 never arrives. |
| Almost no tests | `MainTest.java` is 84 lines covering token checks and TLS config validation. Zero protocol coverage. |

## Non-Goals

- MFDS transport. No official MFDS AS2 specification has been located, so that
  path stays as it is and is not extended on guesswork.
- Migrating the submitter to Rust, or moving it into this repository.
- Replacing the backend's retry, dispatch-state and reconciliation logic, which
  already lives in `web-server` and stays there.
- Multi-file submissions and zip/gzip packaging. FDA requires an archive only
  when a submission carries more than one file; an E2B(R3) ICSR is a single XML
  document. AERS attachments are a separate submission type and are out of
  scope here.

## Chosen Architecture

### Transport ownership

The submitter speaks AS2 itself in both directions. The `direct` transport
becomes the only transport and `openas2` mode is removed.

The alternative — keeping OpenAS2 as the AS2 engine with the submitter as a thin
file-shuffling adapter — was rejected. It scatters protocol behavior into
`partnerships.xml`, makes directory polling the correlation mechanism, and, most
importantly, removes any independent check on our own AS2 correctness: OpenAS2
would be talking to OpenAS2 while our code only moved files.

Owning the transport instead lets OpenAS2 serve as an independent counterparty
in testing. A green run then means two different implementations agree, which is
the closest available proxy for FDA interoperability. It also gives direct
control over the `X-Cyclone-*` routing headers, which are awkward to inject
through OpenAS2.

### Module decomposition

`Main.java` cannot absorb a receiver, FDA routing and MDN verification and stay
readable. The single file is split:

```
com.example.as2
├── Main                    wiring only: validate config, start listeners, schedule background work
├── config/Env              envTruthy / normalize / firstNonBlank / requireEnv
├── config/AuthorityProfile per-authority endpoint, AS2-To, routing headers, certificates, TLS
├── api/ApiServer           127.0.0.1:9090 internal API and its handlers
├── as2/As2Sender           header construction, sign and encrypt, POST, response handling
├── as2/As2Receiver         0.0.0.0:4080 inbound AS2 endpoint
├── as2/Cms                 sign / encrypt / decrypt / verify
├── as2/Mdn                 MDN construction, parsing, signature verification, MIC
├── ack/AckClassifier       inbound payload to (level, success, code, message)
├── ack/AckCorrelator       inbound message to submission record
├── ack/AckForwarder        backend callback and retry queue
└── state/Store             submissions, idempotency keys, pending ACKs, persistence
```

Each unit is independently testable: `Cms` and `Mdn` are pure functions over
bytes, `AuthorityProfile` is pure configuration, and `AckClassifier` /
`AckCorrelator` take a parsed message and return a decision without touching the
network.

### Two listeners

| Listener | Bind | Purpose |
| --- | --- | --- |
| Internal API | `127.0.0.1:${AS2_SUBMITTER_PORT:-9090}` | `/health`, `/internal/status`, `/submit`, `/callbacks/ack`. Called only by `web-server` and operators. |
| AS2 receiver | `0.0.0.0:${AS2_RECEIVER_PORT:-4080}` path `${AS2_RECEIVER_PATH:-/as2/receive}` | Inbound AS2 from the authority: asynchronous MDNs and ACK files. |

The receiver defaults to port 4080 because FDA accepts only 443 or 4080 for the
industry gateway URL. TLS is served in-process when
`AS2_RECEIVER_TLS_KEYSTORE_PATH` is set, otherwise the listener is plaintext,
which is what local and cross-implementation testing uses.

`POST /callbacks/ack` survives on the internal API as a manual injection path
for operators and tests. Once the receiver exists it is no longer how real
acknowledgements arrive, and it stays bound to loopback.

## Outbound Flow

### FDA routing

`AS2_FDA_ROUTING_MODE` selects one of the three documented options, and
`AS2_FDA_ENV` (`prod` or `test`) selects the environment.

| Mode | `AS2-To` | Additional headers | Encryption certificate |
| --- | --- | --- | --- |
| `routing_id` (default) | `FDA_AERS` | none | ZZFDA |
| `metadata_headers` | `ZZFDA` / `ZZFDATST` | `X-Cyclone-Metadata-FdaCenter: CDER`, `X-Cyclone-Metadata-FdaSubmissionType: AERS` | ZZFDA / ZZFDATST |
| `true_receiver` | `ZZFDA` / `ZZFDATST` | `X-Cyclone-True-Receiver: FDA_AERS` | ZZFDA |

`routing_id` is the default because it is the simplest correct configuration for
AERS and needs no custom headers at all.

The endpoint derives from `AS2_FDA_ENV`
(`https://upload-api-esgng.fda.gov:4080/as2/receive` for prod,
`.../as2/receive/test` for test) and is overridable through
`AS2_FDA_ENDPOINT_URL`, which local and EC2 testing requires.

The three header sets are mutually exclusive. Each mode's "do not use" headers
are rejected at startup rather than silently ignored, because sending the wrong
combination produces a routing failure at FDA that is hard to diagnose from our
side.

FDA's `ZZFDA` and `ZZFDATST` public certificates are self-signed RSA-2048 with
`sha256WithRSAEncryption`; both are already extracted from the Gateway
Configuration Information form.

### Headers and crypto

Beyond routing, an outbound submission carries `AS2-Version: 1.2`, `AS2-From`
(our industry routing ID), a `Message-ID` we generate,
`Disposition-Notification-To`, and — when asynchronous MDN is configured —
`Receipt-Delivery-Option` pointing at our receiver URL.

`Disposition-Notification-Options` changes from `optional` to
**`signed-receipt-protocol=required, pkcs7-signature; signed-receipt-micalg=required, sha-256`**.
Under `optional` a partner that never signs its receipts still passes, which
defeats the point of requesting a signed receipt.

The payload is CMS-signed with our private key, then CMS-enveloped to the
authority's public certificate using AES-256-CBC. AES-256-GCM is deliberately
not offered: FDA does not support it.

## Inbound Flow

```
parse headers
  → identify partner by AS2-From
  → decrypt (enveloped-data)
  → verify signature (signed-data / multipart/signed)
  → compute MIC
  → is it an MDN?
      yes → ACK level 1
      no  → file delivery; classify by filename extension
  → correlate to a submission
  → forward to backend (existing retry queue)
  → return a signed MDN
```

### Errors are MDN dispositions, not HTTP statuses

The receiver answers HTTP 200 whenever it can produce an MDN at all. Failures
are reported in the MDN's `Disposition` field (`processed/Error:` or
`failed/Failure:`) per RFC 4130.

This is both the protocol-correct behavior and an explicit FDA operational
requirement: the troubleshooting table names "ACKs are being delivered to
industry AS2 gateway but rejected with a 400 or 500 error" as a cause of lost
acknowledgements. An unmatched or malformed inbound message must never become a
5xx.

### MDN generation

A returned MDN is a `multipart/report; report-type=disposition-notification`
carrying a human-readable `text/plain` part and a
`message/disposition-notification` part with `Reporting-UA`, `Final-Recipient`,
`Original-Message-ID`, `Received-Content-MIC: <base64>, sha-256` and
`Disposition`. When the sender requested a signed receipt the report is wrapped
in `multipart/signed; protocol="application/pkcs7-signature"; micalg=sha-256`.

## ACK Classification and Correlation

### Classification

An inbound message whose content type is `multipart/report` or which contains a
`message/disposition-notification` part is an **MDN**, reported as ACK level 1.
Its `Disposition` decides success: `processed` without an error or failure
token succeeds; `failed` or `error` does not.

Anything else is a file delivery. The level comes from the filename in
`Content-Disposition`, driven by a configurable extension map so a wrong guess
is a config change rather than a code change:

| Extension | Level | Note |
| --- | --- | --- |
| `.ack` | 3 | The AERS acknowledgement |
| `.txt` | 2 | Not produced on the AERS path |
| other | 3 | Conservative default |

An ACK1a virus exception is reported as level 1 with `success=false`.

### Correlation

First match wins:

1. **MDN** — `Original-Message-ID` against the stored outbound `Message-ID`.
   Exact.
2. **File** — an E2B(R3) identifier echoed by the acknowledgement, against the
   identifiers parsed out of `xmlPayload` at submit time and stored on the
   record. See below.
3. **File** — `caseId` appearing in the delivered filename.
4. **No match** — persisted as an *orphan ACK*, surfaced on `/internal/status`,
   and still answered with a `processed` MDN. Nothing is dropped and nothing
   returns an error status.

E2B(R3) is HL7 v3, not the R2 `<ichicsr>` structure, so the correlation keys are
attributes rather than elements. Two are captured at submit time from
`xmlPayload`:

| Field | XPath |
| --- | --- |
| N.1.2 Batch Number | `/MCCI_IN200100UV01/id/@extension` |
| N.2.r.1 Message Identifier | `/MCCI_IN200100UV01/PORR_IN049016UV/id/@extension` |

Rather than hard-coding a path into the acknowledgement message — whose exact
schema we cannot verify from an official source — matching collects **every**
`extension` attribute of every `id` element in the inbound acknowledgement and
succeeds if any equals a stored batch number or message identifier. This is
robust to variations in where the acknowledgement echoes the identifier.

FDA's Core ID (`ci<timestamp>.<GUID>`) is not used as a correlation key: it is
delivered in ACK2, which the AERS path never receives.

## Backend Contract

Unchanged. `web-server` keeps its existing request and callback shapes, so no
Rust changes are required by this work.

Submit stays `POST /submit` with `{caseId, authority, xmlPayload, callbackUrl}`
and the `x-api-token` / `x-callback-token` / `Authorization: Bearer` headers.
Callbacks stay `POST` to `callbackUrl` with header `x-callback-token` and a
snake_case body matching `GatewayAckCallbackInput`:
`{remote_submission_id, ack_level, success, ack_code, ack_message}`.

On the AERS path the backend therefore observes level 1 followed by level 3, and
never level 2. `SubmissionStatus` already models all four levels, so this is a
sequence the existing state machine handles without modification.

## Corrections to Existing Behavior

### MIC computation

`dispatchToAuthority` computes `sha256(xmlPayload_utf8)`. That is only correct
for an unsigned, unencrypted payload — which is precisely the configuration the
March 2026 test runs used, so the defect never surfaced.

For a signed and encrypted message the MIC is computed over the signed MIME
entity, including its MIME headers, after decryption and with the
canonicalization implied by `micalg`. The mock's partnership sets
`prevent_canonicalization_for_mic=false`, so cross-implementation testing will
detect any remaining disagreement.

Canonicalization is the classic source of AS2 MIC mismatches, because the two
sides can disagree about whether it applies to a given media type. That
ambiguity is removed by normalizing line endings to CRLF **once, before the MIME
entity is built**. What we sign and what we hash are then byte-identical and
already canonical, so a partner that canonicalizes and a partner that does not
compute the same MIC.

### S/MIME structure

`buildSignedEncryptedPayload` emits raw CMS `SignedData` wrapped in CMS
`EnvelopedData`. That is not S/MIME: there is no MIME entity, so a standards
conforming partner cannot find the content type of what it just decrypted.
This is the most likely reason the crypto path was never seen working against
OpenAS2.

The outbound message becomes proper S/MIME — a `MimeBodyPart` payload, signed
into `multipart/signed; protocol="application/pkcs7-signature"; micalg=sha-256`,
then enveloped into `application/pkcs7-mime; smime-type=enveloped-data`. This
requires adding `bcmail-jdk18on` and a Jakarta Mail implementation to the build,
which also supplies the MIME parsing the receiver needs for inbound
`multipart/signed` MDNs.

### `remote_submission_id`

Currently preferred from the response's `AS2-Message-ID` header, which is the
MDN's own identifier rather than a submission identifier. It becomes **our
outbound `Message-ID`**: stable, generated before the request, and echoed back
in every MDN as `Original-Message-ID`, which makes correlation fall out of the
protocol instead of requiring a side channel.

### ACK level model

Levels stop being uniform. The AERS path is explicitly ACK1 (MDN) followed by
ACK3 (`.ack` file), with no ACK2.

## Testing

| Level | Scope |
| --- | --- |
| 1. Unit | CMS sign/verify and encrypt/decrypt round trips, tamper detection; MIC against known vectors including canonicalization; MDN build then parse, disposition parsing, `Original-Message-ID` extraction; each routing mode's exact header set including absence of "do not use" headers; ACK classification and each correlation strategy. |
| 2. Loopback | Our sender against our receiver on localhost: signed and encrypted submission, signed MDN returned, MIC matches. A stub backend asserts the callback body matches `GatewayAckCallbackInput` exactly. |
| 3. Cross-implementation, local | Our submitter against OpenAS2 acting as FDA, both in Docker Compose. Outbound proves OpenAS2 accepts our signed and encrypted message and returns a MIC-matching signed MDN. Inbound proves our receiver decrypts an OpenAS2-sent `.ack`, verifies it, and returns an MDN OpenAS2 accepts. |
| 4. Local to EC2 | Level 3 with OpenAS2 on EC2: real network, real TLS, and asynchronous MDN via `Receipt-Delivery-Option`, which a same-host run cannot meaningfully exercise. |

Level 3 is the load-bearing level. Because the two sides are independent
implementations, agreement on MIC, canonicalization, certificate handling and
MDN structure is evidence rather than self-confirmation.

`mock-mfds-openas2` is reconfigured as an FDA counterparty with the `FDA_AERS`
partner identity. Test certificates are generated by script as self-signed
RSA-2048 / SHA-256 pairs, matching the profile of FDA's own certificates.

## Implementation Outcome

Built and merged to `main` in `github.com/donihyun/as2-submitter` as of
2026-07-29. `Main.java` went from 1083 lines to 73 lines of wiring across 22
files. 135 unit and integration tests pass, plus a nine-check
cross-implementation run against OpenAS2 4.9.0 acting as `FDA_AERS`, covering
both directions and the asynchronous MDN path. The backend contract is unchanged
and no Rust code was touched.

Three defects surfaced that this design did not anticipate:

- **BouncyCastle was never registered on the startup path.** `ReceiverConfig`
  asks JCA for provider `"BC"` before anything loads `Cms`, whose static
  initializer installs it. Every real boot with `AS2_RECEIVER_PARTNER_CERTS` set
  died with `no such provider: BC`. Unit tests missed it because they load `Cms`
  first; running the service against the mock found it immediately.
- **Filename preservation needs the MIME entity, not the HTTP header.** A
  partner replaces the outer headers with the decrypted entity's, so a filename
  living only on the HTTP request is lost. FDA's Appendix F requires filename
  preservation. Confirmed against OpenAS2, which stored a random name before the
  fix and the correct name after.
- **`bcmail-jdk18on` cannot be used.** It is compiled against the legacy
  `javax.mail` namespace and fails at runtime alongside Jakarta Mail 2.x. The
  Jakarta-namespaced artifact is `bcjmail-jdk18on`, same package, same version.

The design's own corrections held up: the MIC computation, the S/MIME structure
and the `remote_submission_id` change were all necessary, and the first two are
what the cross-implementation run actually exercises.

## Open Items

Level 4 (local to EC2) is documented in the submitter repository at
`docs/interop-ec2.md` but has not been executed. The asynchronous MDN *code
path* is verified locally; what remains untested is real network behavior, TLS
on our receiver, and an inbound connection from an address we do not control.

MFDS transport remains unspecified. The MFDS E2B(R3) guideline and regional data
elements are published, but the 의약품안전나라 연계보고 transport specification —
whether it is AS2 at all, and if so its routing identifiers, endpoint and
certificates — has not been obtained. The MFDS path is left as it stands until
that document is available.
