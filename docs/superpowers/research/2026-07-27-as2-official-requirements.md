# AS2 submitter — official requirements research (2026-07-27)

Sources: FDA *ESG NextGen AS2 Guide for Industry Users* v2.1 (Jan 2026, local copy) and v2.2
(Jul 2026, fda.gov/media/193542), *ESG NextGen Gateway Configuration Information.docx*,
fda.gov ESG NextGen AS2 Account Set-Up Steps and Submission Acknowledgements pages.

## 1. What our codebase already assumes

`crates/services/web-server/src/submission/gateway.rs` treats the AS2 submitter as an
**external HTTP service**. It is the only consumer of the contract below; the service itself
does not exist anywhere in the repo.

Outbound (web-server → submitter), `POST {AS2_SUBMITTER_URL}/submit`:

```json
{ "caseId": "<uuid>", "authority": "fda|mfds", "xmlPayload": "<e2b r3 xml>",
  "callbackUrl": "<AS2_ACK_CALLBACK_URL or null>" }
```

Headers: `x-api-token`, `x-callback-token`, `Authorization: Bearer <token>` — all three carry
the same value (`AS2_SUBMITTER_TOKEN`, falling back to `AS2_CALLBACK_TOKEN`).

Expected response (camelCase or snake_case both accepted by serde aliases in `rows.rs`):

```json
{ "remoteSubmissionId": "...", "status": "...", "authority": "..." }
```

Inbound (submitter → web-server), `POST /internal/submissions/callbacks/ack` with header
`x-callback-token`, body **snake_case**:

```json
{ "remote_submission_id": "...", "ack_level": 1, "success": true,
  "ack_code": "...", "ack_message": "..." }
```

Retry/backoff, dispatch state and reconciliation already live in the web-server
(`gateway.rs`, `reconcile.rs`), so the submitter does not need its own retry ladder for the
`/submit` hop.

## 2. FDA ESG NextGen — AS2 facts

### Endpoints

| Context | URL |
| --- | --- |
| Production | `https://upload-api-esgng.fda.gov:4080/as2/receive` |
| Test | `https://upload-api-esgng.fda.gov:4080/as2/receive/test` |

Our own receiving gateway URL must be HTTPS on **port 443 or 4080 only** — FDA pushes
async MDNs, ACK2 and ACK3 back to it. It is registered out-of-band via the Gateway
Configuration Information form emailed to ESGNGSupport@fda.hhs.gov. FDA does not pin our SSL
cert and does not need our IPs whitelisted.

### Routing for adverse events (our path: CDER/AERS)

Three mutually exclusive options; **Option 1 is the simplest for AERS**:

| | Option 1 | Option 2 | Option 3 |
| --- | --- | --- | --- |
| `AS2-To` (submission) | `FDA_AERS` | `ZZFDA` (prod) / `ZZFDATST` (test) | `ZZFDA` / `ZZFDATST` |
| `X-Cyclone-True-Receiver` | do not use | do not use | `FDA_AERS` |
| `X-Cyclone-Metadata-FdaCenter` | do not use | `CDER` | do not use |
| `X-Cyclone-Metadata-FdaSubmissionType` | do not use | `AERS` | do not use |
| Encryption cert | `ZZFDA` (both envs) | `ZZFDA` prod / `ZZFDATST` test | `ZZFDA` (both envs) |
| `AS2-From` on returned ACKs | `FDA_AERS` | `ZZFDA` / `ZZFDATST` | `FDA_AERS` |

Note the v2.1 tables spell the header `X-Cyclone-Metadata-True-Receiver`; v2.2 removed
"Metadata" from it, so **`X-Cyclone-True-Receiver`** is current. Guide v2.2 also dropped the
ZZFDATST certificate, leaving `ZZFDA` as the single encryption cert for both environments.

`AS2-From` on our outbound submission is our own industry routing ID, assigned by FDA.

### Crypto

- Sign with our private key; encrypt with FDA's public cert (encryption optional but expected).
- Encryption: AES-128 / AES-192 / **AES-256 (preferred)**, plus legacy RC2, 3DES, Cast5, Idea.
  **AES-256-GCM is not supported** — CBC only.
- FDA's `ZZFDA` and `ZZFDATST` certs (extracted from the config docx) are self-signed
  **RSA-2048, sha256WithRSAEncryption**. Self-signed is normal here; there is no chain to
  validate, only a pinned public key.
- One signing/encryption certificate per industry routing ID, shared across test and prod.
  Different certs per environment requires a second routing ID.

### Message flow and acknowledgements

1. Prepare payload. Multi-file submissions must be a zip or gzip archive.
2. Sign + encrypt, POST over AS2. Our software assigns the `Message-ID` (RFC 4130 form).
3. FDA decrypts, verifies the signature, returns an **MDN**. Sync MDN comes back on the same
   HTTP response; async MDN arrives from `150.148.0.0/16`. The MDN is ACK1.
4. FDA unpacks, assigns a Core ID: `ci<timestamp>.<GUID>`, 47 chars.
5. Antivirus scan; a virus produces an exception **ACK1a**.
6. **For CDER/AERS and CBER/AERS specifically: ACK2 is not sent (N). ACK3 is sent as a `.ack`
   file.** ACKs are not generated for invalid file extensions or improperly encoded files.
7. All ACKs return **over AS2** to our gateway URL — so the submitter must be an AS2
   *receiver* as well as a sender.

Outbound (industry → FDA) source IPs: `15.205.247.22`, `3.31.183.245`.
Inbound (FDA → industry) source range: `150.148.0.0/16`.

Mapping to our `SubmissionStatus`: MDN → ack level 1, AERS `.ack` file → ack level 3.
Level 2 never arrives on the AERS path.

## 3. MFDS — open

Not yet established from an official source whether MFDS accepts E2B(R3) over AS2 and, if so,
what routing IDs, endpoint and certificates apply. MFDS publishes the E2B(R3) guideline and
regional data elements, but the transport spec for 의약품안전나라 (nedrug) 연계보고 was not
located. **Needs the user's own MFDS gateway documentation before the MFDS path is built.**
