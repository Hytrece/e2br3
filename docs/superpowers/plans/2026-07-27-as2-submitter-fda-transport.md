# AS2 Submitter FDA Transport Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give `~/projects/java/as2-submitter` a working AS2 receiver and correct FDA ESG NextGen routing, so an E2B(R3) ICSR can complete a full round trip: signed+encrypted submission out, MDN back as ACK1, and FDA's `.ack` file back over AS2 as ACK3.

**Architecture:** The submitter owns the AS2 protocol in both directions. `Main.java` (1083 lines) is decomposed into `config`, `api`, `as2`, `ack` and `state` packages. Two listeners: the existing internal JSON API on loopback `9090`, and a new public AS2 receiver on `4080`. The `openas2` transport mode is deleted; OpenAS2 becomes the independent counterparty in cross-implementation tests.

**Tech Stack:** Java 21 (toolchain runs JDK 25), Maven 3.9, JUnit 5.11, BouncyCastle 1.78.1 (`bcprov` + `bcpkix`, **adding `bcmail`**), Jakarta Mail (Angus) for MIME, Jackson 2.18, `com.sun.net.httpserver`, Docker Compose + OpenAS2 4.8.0 for interop testing.

**Spec:** [`docs/superpowers/specs/2026-07-27-as2-submitter-fda-design.md`](../specs/2026-07-27-as2-submitter-fda-design.md) in the `e2br3` repo. Requirements provenance: [`docs/superpowers/research/2026-07-27-as2-official-requirements.md`](../research/2026-07-27-as2-official-requirements.md).

**Working directory for every task:** `/Users/hyundonghoon/projects/java/as2-submitter` unless a task says otherwise.

---

## File Structure

Final layout under `src/main/java/com/example/as2/`. Test files mirror the same paths under `src/test/java/`.

| File | Responsibility |
| --- | --- |
| `Main.java` | Wiring only. Validate config, start both listeners, schedule background ACK forwarding. |
| `config/Env.java` | `normalize`, `firstNonBlank`, `envTruthy`, `requireEnv`, `intEnv`. No other class reads `System.getenv` directly. |
| `config/AuthorityProfile.java` | Immutable per-authority transport settings: endpoint, `AS2-From`, `AS2-To`, extra routing headers, encryption certificate path, TLS stores. |
| `config/FdaRouting.java` | The three FDA routing modes and the exact header set each produces. |
| `api/ApiServer.java` | Loopback listener and its four handlers. |
| `as2/As2Headers.java` | AS2 header name constants and small header helpers. |
| `as2/Mic.java` | CRLF canonicalization and MIC computation over a MIME entity. |
| `as2/Cms.java` | S/MIME sign, encrypt, decrypt, verify. Pure functions over `MimeBodyPart`. |
| `as2/Mdn.java` | Build an MDN, parse an inbound MDN, verify its signature, extract disposition and MIC. |
| `as2/InboundMessage.java` | Parsed inbound AS2 message value type. |
| `as2/As2Sender.java` | Build headers, sign+encrypt, POST, interpret the response. |
| `as2/As2Receiver.java` | Public listener. Decrypt, verify, classify, correlate, forward, reply with MDN. |
| `ack/AckClassifier.java` | Inbound message → `AckDecision(level, success, code, message)`. |
| `ack/AckCorrelator.java` | Inbound message + decision → `SubmissionRecord` or orphan. |
| `ack/AckForwarder.java` | POST to the backend `callbackUrl`; retry queue drain. |
| `state/Store.java` | Submissions, idempotency keys, pending ACKs, orphan ACKs, persistence. |
| `state/SubmissionRecord.java` | One submission's durable state. |
| `state/AckForwardTask.java` | One queued backend callback. |

---

## Task 1: Land the existing working tree

The repository has uncommitted changes and untracked infrastructure files. Nothing else in this plan is reviewable until that baseline is committed.

**Files:**
- Modify: `.gitignore`

- [ ] **Step 1: Inspect what is uncommitted**

```bash
cd /Users/hyundonghoon/projects/java/as2-submitter
git status --short
```

Expected: `M` on `.gitignore`, `README.md`, `pom.xml`, `src/main/java/com/example/as2/Main.java`, `src/test/java/com/example/as2/MainTest.java`; `??` on `.env.docker.example`, `Dockerfile`, `docker-compose.openas2.yml`, `scripts/`.

- [ ] **Step 2: Add runtime artefacts to `.gitignore`**

Replace the whole file with:

```gitignore
/target/
/.idea/
*.iml
.DS_Store

.env.docker
as2-state.json
as2-state.json.tmp
/openas2/
/certs/
```

- [ ] **Step 3: Verify the build is green before committing**

```bash
mvn -q -o test
```

Expected: BUILD SUCCESS, 7 tests in `MainTest`, 0 failures.

- [ ] **Step 4: Commit the baseline**

```bash
git add -A
git commit -m "chore: land docker, compose and interop scripts

Commits the previously untracked Dockerfile, OpenAS2 compose file,
env example and verification scripts, plus the accumulated edits to
Main, README and pom, so subsequent AS2 work has a clean baseline."
```

- [ ] **Step 5: Confirm a clean tree**

```bash
git status --short
```

Expected: no output.

---

## Task 2: Add S/MIME dependencies

`buildSignedEncryptedPayload` currently emits raw CMS, not S/MIME, so no conforming partner can interpret it. Proper S/MIME needs `bcmail` and a Jakarta Mail implementation. The same libraries give the receiver its inbound MIME parsing.

**Files:**
- Modify: `pom.xml:12-44`
- Test: `src/test/java/com/example/as2/as2/SmimeDependencyTest.java` (create)

- [ ] **Step 1: Write the failing test**

Create `src/test/java/com/example/as2/as2/SmimeDependencyTest.java`:

```java
package com.example.as2.as2;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertEquals;

import jakarta.mail.internet.MimeBodyPart;
import java.nio.charset.StandardCharsets;
import org.bouncycastle.mail.smime.SMIMESignedGenerator;
import org.junit.jupiter.api.Test;

class SmimeDependencyTest {
    @Test
    void jakartaMailIsOnTheClasspath() throws Exception {
        MimeBodyPart part = new MimeBodyPart();
        part.setContent("<ichicsr/>", "application/xml");
        part.setHeader("Content-Type", "application/xml");
        assertEquals("application/xml", part.getHeader("Content-Type")[0]);
    }

    @Test
    void bcmailIsOnTheClasspath() {
        assertDoesNotThrow(() -> new SMIMESignedGenerator());
    }

    @Test
    void mimeBodyPartRoundTripsBytes() throws Exception {
        MimeBodyPart part = new MimeBodyPart();
        part.setContent("hello".getBytes(StandardCharsets.UTF_8), "application/octet-stream");
        java.io.ByteArrayOutputStream out = new java.io.ByteArrayOutputStream();
        part.writeTo(out);
        String written = out.toString(StandardCharsets.UTF_8);
        assertEquals(true, written.contains("hello"));
    }
}
```

- [ ] **Step 2: Run it to verify it fails**

```bash
mvn -q -o test -Dtest=SmimeDependencyTest
```

Expected: COMPILATION ERROR — `package jakarta.mail.internet does not exist`, `package org.bouncycastle.mail.smime does not exist`.

- [ ] **Step 3: Add the dependencies**

In `pom.xml`, add to `<properties>` after the `bouncycastle.version` line:

```xml
    <angus.version>2.0.3</angus.version>
    <jakarta.mail.version>2.1.3</jakarta.mail.version>
```

Add to `<dependencies>` after the `bcpkix-jdk18on` block:

```xml
    <dependency>
      <groupId>org.bouncycastle</groupId>
      <artifactId>bcmail-jdk18on</artifactId>
      <version>${bouncycastle.version}</version>
    </dependency>
    <dependency>
      <groupId>jakarta.mail</groupId>
      <artifactId>jakarta.mail-api</artifactId>
      <version>${jakarta.mail.version}</version>
    </dependency>
    <dependency>
      <groupId>org.eclipse.angus</groupId>
      <artifactId>angus-mail</artifactId>
      <version>${angus.version}</version>
    </dependency>
```

- [ ] **Step 4: Run the test to verify it passes**

```bash
mvn -o test -Dtest=SmimeDependencyTest
```

Expected: `Tests run: 3, Failures: 0, Errors: 0`. This downloads artifacts, so drop `-o` if the local repository lacks them.

- [ ] **Step 5: Verify the shaded jar still builds**

```bash
mvn -q -o package -DskipTests && ls -la target/as2-submitter.jar
```

Expected: the jar exists. Jakarta Mail ships `META-INF/services` provider files; if the jar fails to start later with `no object DCH for MIME type`, that is a shade-plugin services-merge problem — fix it by adding `<transformer implementation="org.apache.maven.plugins.shade.resource.ServicesResourceTransformer"/>` inside the existing `<transformers>` block in `pom.xml`.

- [ ] **Step 6: Commit**

```bash
git add pom.xml src/test/java/com/example/as2/as2/SmimeDependencyTest.java
git commit -m "build: add bcmail and Jakarta Mail for S/MIME

Raw CMS is not S/MIME: a decrypting partner has no MIME entity and so
no content type for what it just unwrapped. Proper AS2 packaging needs
bcmail, and the receiver needs Jakarta Mail to parse inbound
multipart/signed MDNs."
```

---

## Task 3: Test certificate generation

Every crypto test needs key material. Generate it with a script rather than checking keys into git.

**Files:**
- Create: `scripts/gen_test_certs.sh`
- Create: `src/test/java/com/example/as2/testsupport/TestCerts.java`

- [ ] **Step 1: Write the generator script**

Create `scripts/gen_test_certs.sh`:

```bash
#!/usr/bin/env bash
# Generates self-signed RSA-2048/SHA-256 key pairs for AS2 testing.
# Profile matches FDA's own ZZFDA certificate (self-signed, RSA-2048, sha256WithRSAEncryption).
set -euo pipefail

OUT_DIR="${1:-./certs}"
PASSWORD="${AS2_TEST_CERT_PASSWORD:-changeit}"

mkdir -p "$OUT_DIR"

gen() {
  local name="$1" cn="$2"
  openssl req -x509 -newkey rsa:2048 -sha256 -days 3650 -nodes \
    -keyout "$OUT_DIR/$name.key" \
    -out "$OUT_DIR/$name.crt" \
    -subj "/CN=$cn" 2>/dev/null
  openssl pkcs12 -export \
    -inkey "$OUT_DIR/$name.key" \
    -in "$OUT_DIR/$name.crt" \
    -name "$name" \
    -out "$OUT_DIR/$name.p12" \
    -passout "pass:$PASSWORD"
  echo "  $name -> $OUT_DIR/$name.{key,crt,p12}"
}

echo "Generating AS2 test certificates in $OUT_DIR (password: $PASSWORD)"
gen submitter E2BR3-SUBMITTER
gen partner    FDA_AERS

echo "Done."
```

- [ ] **Step 2: Make it executable and run it**

```bash
chmod +x scripts/gen_test_certs.sh
./scripts/gen_test_certs.sh
openssl x509 -in certs/submitter.crt -noout -subject
openssl x509 -in certs/partner.crt -noout -subject -text | grep -E "Public-Key|Signature Algorithm" | head -2
```

Expected: `subject=CN=E2BR3-SUBMITTER`, then `subject=CN=FDA_AERS`, `Public-Key: (2048 bit)`, `Signature Algorithm: sha256WithRSAEncryption`.

- [ ] **Step 3: Write the test-support loader**

Create `src/test/java/com/example/as2/testsupport/TestCerts.java`:

```java
package com.example.as2.testsupport;

import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.KeyStore;
import java.security.PrivateKey;
import java.security.cert.CertificateFactory;
import java.security.cert.X509Certificate;

/** Loads the key material produced by {@code scripts/gen_test_certs.sh}. */
public final class TestCerts {
    private static final Path DIR = Path.of("certs");
    private static final char[] PASSWORD = "changeit".toCharArray();

    private TestCerts() {}

    /** Skips the calling test with a clear message when the script has not been run. */
    public static void requireGenerated() {
        if (!Files.exists(DIR.resolve("submitter.p12"))) {
            throw new org.opentest4j.TestAbortedException(
                    "Run ./scripts/gen_test_certs.sh first (missing certs/submitter.p12)");
        }
    }

    public static PrivateKey privateKey(String name) throws Exception {
        KeyStore ks = keyStore(name);
        return (PrivateKey) ks.getKey(name, PASSWORD);
    }

    public static X509Certificate certificate(String name) throws Exception {
        try (InputStream in = Files.newInputStream(DIR.resolve(name + ".crt"))) {
            return (X509Certificate) CertificateFactory.getInstance("X.509").generateCertificate(in);
        }
    }

    private static KeyStore keyStore(String name) throws Exception {
        KeyStore ks = KeyStore.getInstance("PKCS12");
        try (InputStream in = Files.newInputStream(DIR.resolve(name + ".p12"))) {
            ks.load(in, PASSWORD);
        }
        return ks;
    }
}
```

- [ ] **Step 4: Verify it compiles and loads**

Create `src/test/java/com/example/as2/testsupport/TestCertsTest.java`:

```java
package com.example.as2.testsupport;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;

import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;

class TestCertsTest {
    @BeforeAll
    static void haveCerts() {
        TestCerts.requireGenerated();
    }

    @Test
    void loadsSubmitterKeyAndCertificate() throws Exception {
        assertNotNull(TestCerts.privateKey("submitter"));
        assertEquals("CN=E2BR3-SUBMITTER", TestCerts.certificate("submitter").getSubjectX500Principal().getName());
    }

    @Test
    void loadsPartnerCertificate() throws Exception {
        assertEquals("CN=FDA_AERS", TestCerts.certificate("partner").getSubjectX500Principal().getName());
    }
}
```

```bash
mvn -q -o test -Dtest=TestCertsTest
```

Expected: `Tests run: 2, Failures: 0, Errors: 0`.

- [ ] **Step 5: Commit**

```bash
git add scripts/gen_test_certs.sh src/test/java/com/example/as2/testsupport/
git commit -m "test: generate self-signed AS2 test certificates

RSA-2048/SHA-256 self-signed, matching the profile of FDA's own ZZFDA
certificate. Key material stays out of git; certs/ is ignored."
```

---

## Task 4: Extract `config/Env`

A pure move, done first so later tasks have one place to read configuration.

**Files:**
- Create: `src/main/java/com/example/as2/config/Env.java`
- Modify: `src/main/java/com/example/as2/Main.java` (delete the private helpers, import the new class)
- Test: `src/test/java/com/example/as2/config/EnvTest.java` (create)

- [ ] **Step 1: Write the failing test**

Create `src/test/java/com/example/as2/config/EnvTest.java`:

```java
package com.example.as2.config;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

class EnvTest {
    @Test
    void normalizeTrimsAndNullsEmpty() {
        assertNull(Env.normalize(null));
        assertNull(Env.normalize("   "));
        assertEquals("value", Env.normalize("  value  "));
    }

    @Test
    void firstNonBlankPicksTheFirstUsableValue() {
        assertEquals("b", Env.firstNonBlank(null, "  ", "b", "c"));
        assertNull(Env.firstNonBlank(null, "", "   "));
    }

    @Test
    void truthyAcceptsTheDocumentedSpellings() {
        assertTrue(Env.truthy("1"));
        assertTrue(Env.truthy("true"));
        assertTrue(Env.truthy("TRUE"));
        assertTrue(Env.truthy("yes"));
        assertTrue(Env.truthy("on"));
        assertFalse(Env.truthy("0"));
        assertFalse(Env.truthy("false"));
        assertFalse(Env.truthy(null));
        assertFalse(Env.truthy(""));
    }

    @Test
    void intOrDefaultFallsBackOnGarbage() {
        assertEquals(30, Env.intOrDefault("30", 20));
        assertEquals(20, Env.intOrDefault("abc", 20));
        assertEquals(20, Env.intOrDefault(null, 20));
    }
}
```

- [ ] **Step 2: Run it to verify it fails**

```bash
mvn -q -o test -Dtest=EnvTest
```

Expected: COMPILATION ERROR — `package com.example.as2.config does not exist`.

- [ ] **Step 3: Write the implementation**

Create `src/main/java/com/example/as2/config/Env.java`:

```java
package com.example.as2.config;

import java.util.Locale;

/** Single entry point for reading environment configuration. */
public final class Env {
    private Env() {}

    /** Trims a value, returning null for null, empty or whitespace-only input. */
    public static String normalize(String value) {
        if (value == null) {
            return null;
        }
        String v = value.trim();
        return v.isEmpty() ? null : v;
    }

    public static String firstNonBlank(String... values) {
        for (String v : values) {
            String n = normalize(v);
            if (n != null) {
                return n;
            }
        }
        return null;
    }

    public static String get(String key) {
        return normalize(System.getenv(key));
    }

    public static String getOrDefault(String key, String fallback) {
        String value = get(key);
        return value == null ? fallback : value;
    }

    public static boolean truthy(String value) {
        String v = normalize(value);
        if (v == null) {
            return false;
        }
        String lower = v.toLowerCase(Locale.ROOT);
        return lower.equals("1") || lower.equals("true") || lower.equals("yes") || lower.equals("on");
    }

    public static boolean envTruthy(String key) {
        return truthy(System.getenv(key));
    }

    public static int intOrDefault(String value, int fallback) {
        String v = normalize(value);
        if (v == null) {
            return fallback;
        }
        try {
            return Integer.parseInt(v);
        } catch (NumberFormatException ex) {
            return fallback;
        }
    }

    public static int envInt(String key, int fallback) {
        return intOrDefault(System.getenv(key), fallback);
    }

    public static long longOrDefault(String value, long fallback) {
        String v = normalize(value);
        if (v == null) {
            return fallback;
        }
        try {
            return Long.parseLong(v);
        } catch (NumberFormatException ex) {
            return fallback;
        }
    }

    public static long envLong(String key, long fallback) {
        return longOrDefault(System.getenv(key), fallback);
    }

    public static String require(String key) {
        String value = get(key);
        if (value == null) {
            throw new IllegalStateException("Missing required env var: " + key);
        }
        return value;
    }
}
```

- [ ] **Step 4: Run the test to verify it passes**

```bash
mvn -q -o test -Dtest=EnvTest
```

Expected: `Tests run: 4, Failures: 0, Errors: 0`.

- [ ] **Step 5: Delete the duplicated helpers from `Main.java`**

In `src/main/java/com/example/as2/Main.java`, delete the private methods `envTruthy` (lines 917-924), `normalize` (926-932), `firstNonBlank` (934-942) and `requireEnv` (150-154). Add `import com.example.as2.config.Env;` and replace every call site: `envTruthy(x)` → `Env.envTruthy(x)`, `normalize(x)` → `Env.normalize(x)`, `firstNonBlank(...)` → `Env.firstNonBlank(...)`, `requireEnv(x)` → `Env.require(x)`.

Replace the numeric parses too: `Integer.parseInt(System.getenv().getOrDefault("AS2_AUTHORITY_TIMEOUT_SECS", "20"))` → `Env.envInt("AS2_AUTHORITY_TIMEOUT_SECS", 20)`, and the same shape for `AS2_ACK_FORWARD_MAX_ATTEMPTS` (default 10), `AS2_ACK_FORWARD_BASE_MS` (default 1000), `AS2_ACK_FORWARD_MAX_MS` (default 60000) and `AS2_SUBMITTER_PORT` (default 9090).

- [ ] **Step 6: Verify the whole suite still passes**

```bash
mvn -q -o test
```

Expected: BUILD SUCCESS. `MainTest` still has 7 passing tests — its two public methods `isAuthorizedToken` and `validateTlsConfigForAuthority` are untouched by this task.

- [ ] **Step 7: Commit**

```bash
git add src/main/java/com/example/as2/config/Env.java \
        src/main/java/com/example/as2/Main.java \
        src/test/java/com/example/as2/config/EnvTest.java
git commit -m "refactor: extract config.Env from Main

One place reads the environment. Adds typed int/long accessors so
call sites stop repeating parse-with-default."
```

---

## Task 5: `as2/Mic` — canonicalization and MIC

The MIC is the single most common cause of AS2 interop failure. The current code
computes `sha256(xmlPayload_utf8)`, which is only right for an unsigned,
unencrypted payload.

The strategy is to remove the canonicalization ambiguity rather than negotiate
it: normalize line endings to CRLF **once**, before building the MIME entity.
What we sign and what we hash are then byte-identical and already canonical, so
a partner that canonicalizes and a partner that does not agree on the answer.

**Files:**
- Create: `src/main/java/com/example/as2/as2/Mic.java`
- Test: `src/test/java/com/example/as2/as2/MicTest.java` (create)

- [ ] **Step 1: Write the failing test**

Create `src/test/java/com/example/as2/as2/MicTest.java`:

```java
package com.example.as2.as2;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import jakarta.mail.internet.MimeBodyPart;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.util.Base64;
import org.junit.jupiter.api.Test;

class MicTest {
    @Test
    void canonicalizeConvertsLoneLfToCrlf() {
        assertArrayEquals(
                "a\r\nb\r\n".getBytes(StandardCharsets.UTF_8),
                Mic.canonicalize("a\nb\n".getBytes(StandardCharsets.UTF_8)));
    }

    @Test
    void canonicalizeLeavesExistingCrlfAlone() {
        byte[] already = "a\r\nb\r\n".getBytes(StandardCharsets.UTF_8);
        assertArrayEquals(already, Mic.canonicalize(already));
    }

    @Test
    void canonicalizeConvertsLoneCrToCrlf() {
        assertArrayEquals(
                "a\r\nb".getBytes(StandardCharsets.UTF_8),
                Mic.canonicalize("a\rb".getBytes(StandardCharsets.UTF_8)));
    }

    @Test
    void canonicalizeIsIdempotent() {
        byte[] once = Mic.canonicalize("a\nb\rc\r\nd".getBytes(StandardCharsets.UTF_8));
        assertArrayEquals(once, Mic.canonicalize(once));
    }

    @Test
    void computeMatchesSha256OfTheEntityBytes() throws Exception {
        MimeBodyPart part = new MimeBodyPart();
        part.setContent("<ichicsr/>", "application/xml");
        part.setHeader("Content-Type", "application/xml");

        java.io.ByteArrayOutputStream out = new java.io.ByteArrayOutputStream();
        part.writeTo(out);
        byte[] expectedDigest = MessageDigest.getInstance("SHA-256")
                .digest(Mic.canonicalize(out.toByteArray()));

        assertEquals(Base64.getEncoder().encodeToString(expectedDigest), Mic.compute(part, "sha-256"));
    }

    @Test
    void computeIncludesMimeHeadersNotJustTheBody() throws Exception {
        MimeBodyPart xml = new MimeBodyPart();
        xml.setContent("payload", "application/xml");
        xml.setHeader("Content-Type", "application/xml");

        MimeBodyPart text = new MimeBodyPart();
        text.setContent("payload", "text/plain");
        text.setHeader("Content-Type", "text/plain");

        assertNotEquals(Mic.compute(xml, "sha-256"), Mic.compute(text, "sha-256"));
    }

    @Test
    void computeSupportsSha1ForLegacyPartners() throws Exception {
        MimeBodyPart part = new MimeBodyPart();
        part.setContent("x", "text/plain");
        assertEquals(28, Mic.compute(part, "sha-1").length());
    }

    @Test
    void computeRejectsUnknownAlgorithms() throws Exception {
        MimeBodyPart part = new MimeBodyPart();
        part.setContent("x", "text/plain");
        assertThrows(IllegalArgumentException.class, () -> Mic.compute(part, "md5-ish"));
    }

    @Test
    void micHeaderValueCarriesTheAlgorithm() throws Exception {
        MimeBodyPart part = new MimeBodyPart();
        part.setContent("x", "text/plain");
        assertEquals(Mic.compute(part, "sha-256") + ", sha-256", Mic.headerValue(part, "sha-256"));
    }

    @Test
    void parseHeaderValueSplitsDigestFromAlgorithm() {
        Mic.Parsed parsed = Mic.parseHeaderValue("abc123=, sha-256");
        assertEquals("abc123=", parsed.digest());
        assertEquals("sha-256", parsed.algorithm());
    }

    @Test
    void parseHeaderValueToleratesAMissingAlgorithm() {
        Mic.Parsed parsed = Mic.parseHeaderValue("abc123=");
        assertEquals("abc123=", parsed.digest());
        assertEquals("sha-256", parsed.algorithm());
    }
}
```

- [ ] **Step 2: Run it to verify it fails**

```bash
mvn -q -o test -Dtest=MicTest
```

Expected: COMPILATION ERROR — `cannot find symbol: class Mic`.

- [ ] **Step 3: Write the implementation**

Create `src/main/java/com/example/as2/as2/Mic.java`:

```java
package com.example.as2.as2;

import jakarta.mail.internet.MimeBodyPart;
import java.io.ByteArrayOutputStream;
import java.security.MessageDigest;
import java.util.Base64;
import java.util.Locale;

/**
 * AS2 Message Integrity Check.
 *
 * <p>The MIC is a base64 digest over the MIME entity that was signed, headers
 * included. RFC 4130 leaves room for disagreement about whether canonicalization
 * applies to a given media type; we sidestep that by canonicalizing line endings
 * to CRLF before the entity is ever built, so both interpretations agree.
 */
public final class Mic {
    private Mic() {}

    /** A parsed {@code Received-Content-MIC} header value. */
    public record Parsed(String digest, String algorithm) {}

    /** Rewrites lone CR and lone LF to CRLF. Already-CRLF input is returned unchanged. */
    public static byte[] canonicalize(byte[] input) {
        ByteArrayOutputStream out = new ByteArrayOutputStream(input.length + 16);
        for (int i = 0; i < input.length; i++) {
            byte b = input[i];
            if (b == '\r') {
                out.write('\r');
                out.write('\n');
                if (i + 1 < input.length && input[i + 1] == '\n') {
                    i++;
                }
            } else if (b == '\n') {
                out.write('\r');
                out.write('\n');
            } else {
                out.write(b);
            }
        }
        return out.toByteArray();
    }

    /** Base64 digest of the canonicalized serialized MIME entity. */
    public static String compute(MimeBodyPart part, String algorithm) throws Exception {
        ByteArrayOutputStream out = new ByteArrayOutputStream();
        part.writeTo(out);
        byte[] canonical = canonicalize(out.toByteArray());
        MessageDigest digest = MessageDigest.getInstance(javaDigestName(algorithm));
        return Base64.getEncoder().encodeToString(digest.digest(canonical));
    }

    /** The value for a {@code Received-Content-MIC} header: {@code <digest>, <algorithm>}. */
    public static String headerValue(MimeBodyPart part, String algorithm) throws Exception {
        return compute(part, algorithm) + ", " + algorithm.toLowerCase(Locale.ROOT);
    }

    /** Splits {@code <digest>, <algorithm>}. Defaults to sha-256 when the algorithm is absent. */
    public static Parsed parseHeaderValue(String raw) {
        String value = raw == null ? "" : raw.trim();
        int comma = value.indexOf(',');
        if (comma < 0) {
            return new Parsed(value, "sha-256");
        }
        return new Parsed(
                value.substring(0, comma).trim(),
                value.substring(comma + 1).trim().toLowerCase(Locale.ROOT));
    }

    private static String javaDigestName(String algorithm) {
        String a = algorithm == null ? "" : algorithm.trim().toLowerCase(Locale.ROOT);
        return switch (a) {
            case "sha-256", "sha256" -> "SHA-256";
            case "sha-384", "sha384" -> "SHA-384";
            case "sha-512", "sha512" -> "SHA-512";
            case "sha-1", "sha1" -> "SHA-1";
            default -> throw new IllegalArgumentException("unsupported MIC algorithm: " + algorithm);
        };
    }
}
```

- [ ] **Step 4: Run the test to verify it passes**

```bash
mvn -q -o test -Dtest=MicTest
```

Expected: `Tests run: 11, Failures: 0, Errors: 0`.

- [ ] **Step 5: Commit**

```bash
git add src/main/java/com/example/as2/as2/Mic.java src/test/java/com/example/as2/as2/MicTest.java
git commit -m "feat: correct AS2 MIC computation

Digest the serialized MIME entity including its headers, not the bare
payload bytes. Canonicalize to CRLF up front so partners that do and do
not canonicalize reach the same MIC."
```

---

## Task 6: `as2/Cms` — S/MIME sign, encrypt, decrypt, verify

**Files:**
- Create: `src/main/java/com/example/as2/as2/Cms.java`
- Test: `src/test/java/com/example/as2/as2/CmsTest.java` (create)

- [ ] **Step 1: Write the failing test**

Create `src/test/java/com/example/as2/as2/CmsTest.java`:

```java
package com.example.as2.as2;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.example.as2.testsupport.TestCerts;
import jakarta.mail.internet.MimeBodyPart;
import java.nio.charset.StandardCharsets;
import java.security.PrivateKey;
import java.security.cert.X509Certificate;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;

class CmsTest {
    private static PrivateKey submitterKey;
    private static X509Certificate submitterCert;
    private static PrivateKey partnerKey;
    private static X509Certificate partnerCert;

    @BeforeAll
    static void load() throws Exception {
        TestCerts.requireGenerated();
        submitterKey = TestCerts.privateKey("submitter");
        submitterCert = TestCerts.certificate("submitter");
        partnerKey = TestCerts.privateKey("partner");
        partnerCert = TestCerts.certificate("partner");
    }

    private static MimeBodyPart payload() throws Exception {
        return Cms.buildPayload("<MCCI_IN200100UV01/>".getBytes(StandardCharsets.UTF_8), "application/xml");
    }

    @Test
    void buildPayloadCanonicalizesContent() throws Exception {
        MimeBodyPart part = Cms.buildPayload("a\nb".getBytes(StandardCharsets.UTF_8), "application/xml");
        java.io.ByteArrayOutputStream out = new java.io.ByteArrayOutputStream();
        part.writeTo(out);
        assertTrue(out.toString(StandardCharsets.UTF_8).contains("a\r\nb"));
    }

    @Test
    void signProducesMultipartSigned() throws Exception {
        MimeBodyPart signed = Cms.sign(payload(), submitterKey, submitterCert, "sha-256");
        assertTrue(signed.getContentType().toLowerCase().startsWith("multipart/signed"));
        assertTrue(signed.getContentType().toLowerCase().contains("micalg=sha-256"));
        assertTrue(signed.getContentType().contains("application/pkcs7-signature"));
    }

    @Test
    void verifyReturnsTheOriginalContent() throws Exception {
        MimeBodyPart signed = Cms.sign(payload(), submitterKey, submitterCert, "sha-256");
        MimeBodyPart recovered = Cms.verify(signed, submitterCert);
        assertArrayEquals(Cms.contentBytes(payload()), Cms.contentBytes(recovered));
    }

    @Test
    void verifyRejectsTheWrongSignerCertificate() throws Exception {
        MimeBodyPart signed = Cms.sign(payload(), submitterKey, submitterCert, "sha-256");
        assertThrows(SecurityException.class, () -> Cms.verify(signed, partnerCert));
    }

    @Test
    void encryptProducesEnvelopedPkcs7Mime() throws Exception {
        MimeBodyPart encrypted = Cms.encrypt(payload(), partnerCert, "aes-256-cbc");
        String type = encrypted.getContentType().toLowerCase();
        assertTrue(type.startsWith("application/pkcs7-mime"));
        assertTrue(type.contains("smime-type=enveloped-data"));
    }

    @Test
    void decryptRecoversTheEncryptedPart() throws Exception {
        MimeBodyPart encrypted = Cms.encrypt(payload(), partnerCert, "aes-256-cbc");
        MimeBodyPart recovered = Cms.decrypt(encrypted, partnerKey, partnerCert);
        assertArrayEquals(Cms.contentBytes(payload()), Cms.contentBytes(recovered));
    }

    @Test
    void signThenEncryptRoundTripsAndPreservesTheMic() throws Exception {
        MimeBodyPart signed = Cms.sign(payload(), submitterKey, submitterCert, "sha-256");
        String micBeforeSending = Mic.compute(payload(), "sha-256");

        MimeBodyPart encrypted = Cms.encrypt(signed, partnerCert, "aes-256-cbc");
        MimeBodyPart decrypted = Cms.decrypt(encrypted, partnerKey, partnerCert);
        MimeBodyPart content = Cms.verify(decrypted, submitterCert);

        assertEquals(micBeforeSending, Mic.compute(content, "sha-256"));
    }

    @Test
    void decryptRejectsTheWrongRecipientKey() throws Exception {
        MimeBodyPart encrypted = Cms.encrypt(payload(), partnerCert, "aes-256-cbc");
        assertThrows(SecurityException.class, () -> Cms.decrypt(encrypted, submitterKey, submitterCert));
    }

    @Test
    void encryptRejectsAesGcmWhichFdaDoesNotSupport() throws Exception {
        assertThrows(IllegalArgumentException.class, () -> Cms.encrypt(payload(), partnerCert, "aes-256-gcm"));
    }

    @Test
    void encryptSupportsTheDocumentedFdaAlgorithms() throws Exception {
        for (String algorithm : new String[] {"aes-128-cbc", "aes-192-cbc", "aes-256-cbc", "3des"}) {
            MimeBodyPart encrypted = Cms.encrypt(payload(), partnerCert, algorithm);
            MimeBodyPart recovered = Cms.decrypt(encrypted, partnerKey, partnerCert);
            assertArrayEquals(Cms.contentBytes(payload()), Cms.contentBytes(recovered), algorithm);
        }
    }
}
```

- [ ] **Step 2: Run it to verify it fails**

```bash
mvn -q -o test -Dtest=CmsTest
```

Expected: COMPILATION ERROR — `cannot find symbol: class Cms`.

- [ ] **Step 3: Write the implementation**

Create `src/main/java/com/example/as2/as2/Cms.java`:

```java
package com.example.as2.as2;

import jakarta.mail.internet.MimeBodyPart;
import jakarta.mail.internet.MimeMultipart;
import jakarta.mail.util.ByteArrayDataSource;
import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.security.PrivateKey;
import java.security.Security;
import java.security.cert.X509Certificate;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.Locale;
import org.bouncycastle.asn1.ASN1ObjectIdentifier;
import org.bouncycastle.asn1.smime.SMIMECapabilitiesAttribute;
import org.bouncycastle.asn1.smime.SMIMECapabilityVector;
import org.bouncycastle.asn1.cms.AttributeTable;
import org.bouncycastle.cms.CMSAlgorithm;
import org.bouncycastle.cms.RecipientInformation;
import org.bouncycastle.cms.SignerInformation;
import org.bouncycastle.cms.jcajce.JcaSimpleSignerInfoVerifierBuilder;
import org.bouncycastle.cms.jcajce.JceCMSContentEncryptorBuilder;
import org.bouncycastle.cms.jcajce.JceKeyTransEnvelopedRecipient;
import org.bouncycastle.cms.jcajce.JceKeyTransRecipientInfoGenerator;
import org.bouncycastle.jce.provider.BouncyCastleProvider;
import org.bouncycastle.mail.smime.SMIMEEnveloped;
import org.bouncycastle.mail.smime.SMIMEEnvelopedGenerator;
import org.bouncycastle.mail.smime.SMIMESigned;
import org.bouncycastle.mail.smime.SMIMESignedGenerator;
import org.bouncycastle.operator.jcajce.JcaContentSignerBuilder;
import org.bouncycastle.cms.jcajce.JcaSignerInfoGeneratorBuilder;
import org.bouncycastle.operator.jcajce.JcaDigestCalculatorProviderBuilder;
import org.bouncycastle.util.Store;

/** S/MIME operations for AS2 payloads and MDNs. */
public final class Cms {
    static {
        if (Security.getProvider("BC") == null) {
            Security.addProvider(new BouncyCastleProvider());
        }
    }

    private Cms() {}

    /**
     * Wraps raw bytes as a MIME entity, canonicalizing line endings so the MIC is
     * stable regardless of the partner's canonicalization behavior.
     */
    public static MimeBodyPart buildPayload(byte[] content, String contentType) throws Exception {
        MimeBodyPart part = new MimeBodyPart();
        part.setDataHandler(new jakarta.activation.DataHandler(
                new ByteArrayDataSource(Mic.canonicalize(content), contentType)));
        part.setHeader("Content-Type", contentType);
        part.setHeader("Content-Transfer-Encoding", "binary");
        return part;
    }

    /** The decoded content bytes of a MIME entity, without its headers. */
    public static byte[] contentBytes(MimeBodyPart part) throws Exception {
        ByteArrayOutputStream out = new ByteArrayOutputStream();
        part.getDataHandler().writeTo(out);
        return out.toByteArray();
    }

    /** The full serialized entity: headers, blank line, content. */
    public static byte[] entityBytes(MimeBodyPart part) throws Exception {
        ByteArrayOutputStream out = new ByteArrayOutputStream();
        part.writeTo(out);
        return out.toByteArray();
    }

    /** Signs into {@code multipart/signed; protocol="application/pkcs7-signature"}. */
    public static MimeBodyPart sign(
            MimeBodyPart content, PrivateKey key, X509Certificate cert, String micalg) throws Exception {
        SMIMESignedGenerator generator = new SMIMESignedGenerator();

        SMIMECapabilityVector capabilities = new SMIMECapabilityVector();
        capabilities.addCapability(org.bouncycastle.asn1.smime.SMIMECapability.aES256_CBC);
        capabilities.addCapability(org.bouncycastle.asn1.smime.SMIMECapability.aES128_CBC);
        capabilities.addCapability(org.bouncycastle.asn1.smime.SMIMECapability.dES_EDE3_CBC);
        org.bouncycastle.asn1.ASN1EncodableVector signedAttrs = new org.bouncycastle.asn1.ASN1EncodableVector();
        signedAttrs.add(new SMIMECapabilitiesAttribute(capabilities));

        generator.addSignerInfoGenerator(
                new JcaSignerInfoGeneratorBuilder(
                        new JcaDigestCalculatorProviderBuilder().setProvider("BC").build())
                        .setSignedAttributeGenerator(new AttributeTable(signedAttrs))
                        .build(new JcaContentSignerBuilder(signatureAlgorithm(micalg)).setProvider("BC").build(key), cert));

        List<X509Certificate> chain = new ArrayList<>();
        chain.add(cert);
        generator.addCertificates(new org.bouncycastle.cert.jcajce.JcaCertStore(chain));

        MimeMultipart multipart = generator.generate(content);
        MimeBodyPart signed = new MimeBodyPart();
        signed.setContent(multipart);
        signed.setHeader("Content-Type", multipart.getContentType());
        return signed;
    }

    /** Verifies a {@code multipart/signed} entity against an expected signer and returns its content. */
    public static MimeBodyPart verify(MimeBodyPart signedPart, X509Certificate expectedSigner) throws Exception {
        SMIMESigned signed;
        try {
            Object content = signedPart.getContent();
            signed = content instanceof MimeMultipart mm
                    ? new SMIMESigned(mm)
                    : new SMIMESigned(signedPart);
        } catch (Exception ex) {
            throw new SecurityException("not a parseable signed entity: " + ex.getMessage(), ex);
        }

        Store<?> certificates = signed.getCertificates();
        boolean verified = false;
        for (SignerInformation signer : signed.getSignerInfos().getSigners()) {
            try {
                if (signer.verify(new JcaSimpleSignerInfoVerifierBuilder()
                        .setProvider("BC")
                        .build(expectedSigner))) {
                    verified = true;
                    break;
                }
            } catch (Exception ignored) {
                // try the next signer
            }
        }
        if (!verified) {
            throw new SecurityException("signature does not verify against the expected certificate");
        }
        return (MimeBodyPart) signed.getContent();
    }

    /** Envelopes into {@code application/pkcs7-mime; smime-type=enveloped-data}. */
    public static MimeBodyPart encrypt(MimeBodyPart content, X509Certificate recipient, String algorithm)
            throws Exception {
        SMIMEEnvelopedGenerator generator = new SMIMEEnvelopedGenerator();
        generator.addRecipientInfoGenerator(
                new JceKeyTransRecipientInfoGenerator(recipient).setProvider("BC"));
        return generator.generate(
                content,
                new JceCMSContentEncryptorBuilder(encryptionOid(algorithm)).setProvider("BC").build());
    }

    /** Unwraps an enveloped entity with our own key. */
    public static MimeBodyPart decrypt(MimeBodyPart encrypted, PrivateKey key, X509Certificate cert)
            throws Exception {
        SMIMEEnveloped enveloped;
        try {
            enveloped = new SMIMEEnveloped(encrypted);
        } catch (Exception ex) {
            throw new SecurityException("not a parseable enveloped entity: " + ex.getMessage(), ex);
        }
        Iterator<RecipientInformation> recipients =
                enveloped.getRecipientInfos().getRecipients().iterator();
        while (recipients.hasNext()) {
            RecipientInformation recipient = recipients.next();
            try {
                byte[] decrypted = recipient.getContent(
                        new JceKeyTransEnvelopedRecipient(key).setProvider("BC"));
                return new MimeBodyPart(new ByteArrayInputStream(decrypted));
            } catch (Exception ignored) {
                // try the next recipient info
            }
        }
        throw new SecurityException("no recipient info could be decrypted with the supplied key");
    }

    private static String signatureAlgorithm(String micalg) {
        String a = micalg == null ? "sha-256" : micalg.trim().toLowerCase(Locale.ROOT);
        return switch (a) {
            case "sha-256", "sha256" -> "SHA256withRSA";
            case "sha-384", "sha384" -> "SHA384withRSA";
            case "sha-512", "sha512" -> "SHA512withRSA";
            case "sha-1", "sha1" -> "SHA1withRSA";
            default -> throw new IllegalArgumentException("unsupported signing algorithm: " + micalg);
        };
    }

    /**
     * FDA supports AES-128/192/256 CBC, RC2, 3DES, Cast5 and Idea, and explicitly
     * does not support AES-256-GCM.
     */
    private static ASN1ObjectIdentifier encryptionOid(String algorithm) {
        String a = algorithm == null ? "" : algorithm.trim().toLowerCase(Locale.ROOT);
        return switch (a) {
            case "aes-128-cbc", "aes128" -> CMSAlgorithm.AES128_CBC;
            case "aes-192-cbc", "aes192" -> CMSAlgorithm.AES192_CBC;
            case "aes-256-cbc", "aes256" -> CMSAlgorithm.AES256_CBC;
            case "3des", "des-ede3-cbc" -> CMSAlgorithm.DES_EDE3_CBC;
            case "cast5" -> CMSAlgorithm.CAST5_CBC;
            case "idea" -> CMSAlgorithm.IDEA_CBC;
            case "rc2" -> CMSAlgorithm.RC2_CBC;
            default -> throw new IllegalArgumentException(
                    "unsupported encryption algorithm for FDA ESG: " + algorithm);
        };
    }
}
```

- [ ] **Step 4: Run the test to verify it passes**

```bash
mvn -q -o test -Dtest=CmsTest
```

Expected: `Tests run: 10, Failures: 0, Errors: 0`.

If `signThenEncryptRoundTripsAndPreservesTheMic` fails on a MIC mismatch, the
cause is Jakarta Mail re-encoding the content on the round trip. Confirm that
`buildPayload` sets `Content-Transfer-Encoding: binary` and that
`SMIMESignedGenerator` is not re-wrapping — do not "fix" it by loosening the
assertion, because that assertion is what protects real interop.

- [ ] **Step 5: Commit**

```bash
git add src/main/java/com/example/as2/as2/Cms.java src/test/java/com/example/as2/as2/CmsTest.java
git commit -m "feat: real S/MIME sign, encrypt, decrypt and verify

Replaces raw CMS with multipart/signed plus enveloped pkcs7-mime, which
is what a conforming AS2 partner can actually parse. Rejects AES-256-GCM,
which FDA ESG does not support."
```

---

## Task 7: Extract `state` package

Moves the durable state out of `Main`, and adds the two fields correlation will
need (the E2B identifiers) plus orphan ACK storage.

**Files:**
- Create: `src/main/java/com/example/as2/state/SubmissionRecord.java`
- Create: `src/main/java/com/example/as2/state/AckForwardTask.java`
- Create: `src/main/java/com/example/as2/state/OrphanAck.java`
- Create: `src/main/java/com/example/as2/state/Store.java`
- Modify: `src/main/java/com/example/as2/Main.java` (delete the moved inner classes and the static maps)
- Test: `src/test/java/com/example/as2/state/StoreTest.java` (create)

- [ ] **Step 1: Write the failing test**

Create `src/test/java/com/example/as2/state/StoreTest.java`:

```java
package com.example.as2.state;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.file.Files;
import java.nio.file.Path;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

class StoreTest {
    private static SubmissionRecord record(String id, String remoteId) {
        SubmissionRecord r = new SubmissionRecord();
        r.submissionId = id;
        r.remoteSubmissionId = remoteId;
        r.authority = "fda";
        r.caseId = "case-1";
        r.status = "submitted_ack1_pending";
        r.callbackUrl = "http://127.0.0.1:8080/internal/submissions/callbacks/ack";
        return r;
    }

    @Test
    void putAndGetRoundTrip(@TempDir Path dir) {
        Store store = new Store(dir.resolve("state.json"));
        store.putSubmission(record("s1", "r1"));
        assertEquals("r1", store.getSubmission("s1").remoteSubmissionId);
    }

    @Test
    void findByRemoteSubmissionId(@TempDir Path dir) {
        Store store = new Store(dir.resolve("state.json"));
        store.putSubmission(record("s1", "r1"));
        assertEquals("s1", store.findByRemoteSubmissionId("r1").submissionId);
        assertNull(store.findByRemoteSubmissionId("nope"));
    }

    @Test
    void findByAs2MessageIdMatchesExactly(@TempDir Path dir) {
        Store store = new Store(dir.resolve("state.json"));
        SubmissionRecord r = record("s1", "r1");
        r.as2MessageId = "<abc@as2-submitter>";
        store.putSubmission(r);
        assertEquals("s1", store.findByAs2MessageId("<abc@as2-submitter>").submissionId);
        assertNull(store.findByAs2MessageId("<other@as2-submitter>"));
    }

    @Test
    void findByE2bIdentifierMatchesBatchOrMessageId(@TempDir Path dir) {
        Store store = new Store(dir.resolve("state.json"));
        SubmissionRecord r = record("s1", "r1");
        r.e2bBatchNumber = "BATCH-001";
        r.e2bMessageIdentifier = "MSG-001";
        store.putSubmission(r);

        assertEquals("s1", store.findByE2bIdentifier(java.util.Set.of("MSG-001")).submissionId);
        assertEquals("s1", store.findByE2bIdentifier(java.util.Set.of("x", "BATCH-001")).submissionId);
        assertNull(store.findByE2bIdentifier(java.util.Set.of("unrelated")));
    }

    @Test
    void findByCaseIdInFilename(@TempDir Path dir) {
        Store store = new Store(dir.resolve("state.json"));
        store.putSubmission(record("s1", "r1"));
        assertEquals("s1", store.findByCaseIdInFilename("ACK_case-1_20260727.ack").submissionId);
        assertNull(store.findByCaseIdInFilename("ACK_other_20260727.ack"));
    }

    @Test
    void idempotencyKeysResolveToASubmission(@TempDir Path dir) {
        Store store = new Store(dir.resolve("state.json"));
        store.putSubmission(record("s1", "r1"));
        store.putIdempotency("fda:case-1:key-1", "s1");
        assertEquals("s1", store.resolveIdempotency("fda:case-1:key-1"));
        assertNull(store.resolveIdempotency("fda:case-1:key-2"));
    }

    @Test
    void orphanAcksAreRetained(@TempDir Path dir) {
        Store store = new Store(dir.resolve("state.json"));
        OrphanAck orphan = new OrphanAck();
        orphan.id = "o1";
        orphan.as2From = "FDA_AERS";
        orphan.filename = "unknown.ack";
        orphan.ackLevel = 3;
        store.putOrphanAck(orphan);
        assertEquals(1, store.orphanAckCount());
        assertEquals("unknown.ack", store.orphanAcks().get(0).filename);
    }

    @Test
    void statePersistsAcrossInstances(@TempDir Path dir) throws Exception {
        Path file = dir.resolve("state.json");
        Store first = new Store(file);
        first.putSubmission(record("s1", "r1"));
        first.putIdempotency("fda:case-1:key-1", "s1");
        first.save();
        assertTrue(Files.exists(file));

        Store second = new Store(file);
        second.load();
        assertNotNull(second.getSubmission("s1"));
        assertEquals("s1", second.resolveIdempotency("fda:case-1:key-1"));
    }

    @Test
    void loadOnAMissingFileIsANoOp(@TempDir Path dir) {
        Store store = new Store(dir.resolve("absent.json"));
        store.load();
        assertEquals(0, store.submissionCount());
    }
}
```

- [ ] **Step 2: Run it to verify it fails**

```bash
mvn -q -o test -Dtest=StoreTest
```

Expected: COMPILATION ERROR — `package com.example.as2.state does not exist`.

- [ ] **Step 3: Write the record types**

Create `src/main/java/com/example/as2/state/SubmissionRecord.java`. This is the
existing inner class from `Main.java:973-990` plus four new fields —
`e2bBatchNumber`, `e2bMessageIdentifier`, `outboundFilename` and `updatedAt`
already present:

```java
package com.example.as2.state;

/** Durable state for one submission. Public fields keep Jackson mapping trivial. */
public class SubmissionRecord {
    public String submissionId;
    public String remoteSubmissionId;
    public String authority;
    public String caseId;
    public String status;
    public String updatedAt;
    public String callbackUrl;
    public String idempotencyKey;
    public String as2MessageId;
    public boolean mdnReceived;
    public String mdnDisposition;
    public String expectedMic;
    public String receivedMic;
    public Boolean mdnMicMatch;

    /** N.1.2 Batch Number, parsed from the payload at submit time. */
    public String e2bBatchNumber;
    /** N.2.r.1 Message Identifier, parsed from the payload at submit time. */
    public String e2bMessageIdentifier;
    /** The filename we sent, used as a last-resort correlation key. */
    public String outboundFilename;

    public SubmissionRecord() {}
}
```

Create `src/main/java/com/example/as2/state/AckForwardTask.java` — an exact move
of the inner class at `Main.java:992-1006`:

```java
package com.example.as2.state;

/** One queued backend callback awaiting delivery. */
public class AckForwardTask {
    public String id;
    public String submissionId;
    public String remoteSubmissionId;
    public String callbackUrl;
    public int ackLevel;
    public boolean success;
    public String ackCode;
    public String ackMessage;
    public int attemptCount;
    public String nextAttemptAt;
    public String lastError;

    public AckForwardTask() {}
}
```

Create `src/main/java/com/example/as2/state/OrphanAck.java`:

```java
package com.example.as2.state;

/** An inbound acknowledgement that matched no submission. Retained, never dropped. */
public class OrphanAck {
    public String id;
    public String receivedAt;
    public String as2From;
    public String as2MessageId;
    public String filename;
    public int ackLevel;
    public boolean success;
    public String ackCode;
    public String ackMessage;
    /** Base64 of the decrypted payload, so an operator can inspect what arrived. */
    public String payloadBase64;

    public OrphanAck() {}
}
```

- [ ] **Step 4: Write the store**

Create `src/main/java/com/example/as2/state/Store.java`:

```java
package com.example.as2.state;

import com.fasterxml.jackson.databind.ObjectMapper;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;

/** In-memory state with atomic file persistence. */
public class Store {
    private static final ObjectMapper MAPPER = new ObjectMapper();

    private final Map<String, SubmissionRecord> submissions = new ConcurrentHashMap<>();
    private final Map<String, String> idempotency = new ConcurrentHashMap<>();
    private final Map<String, AckForwardTask> pendingAcks = new ConcurrentHashMap<>();
    private final Map<String, OrphanAck> orphanAcks = new ConcurrentHashMap<>();
    private final Path file;
    private final Object lock = new Object();

    public Store(Path file) {
        this.file = file;
    }

    public static class Snapshot {
        public Map<String, SubmissionRecord> submissions;
        public Map<String, String> idempotency;
        public Map<String, AckForwardTask> pendingAcks;
        public Map<String, OrphanAck> orphanAcks;

        public Snapshot() {}
    }

    public void putSubmission(SubmissionRecord record) {
        submissions.put(record.submissionId, record);
    }

    public SubmissionRecord getSubmission(String submissionId) {
        return submissionId == null ? null : submissions.get(submissionId);
    }

    public int submissionCount() {
        return submissions.size();
    }

    public SubmissionRecord findByRemoteSubmissionId(String remoteSubmissionId) {
        if (remoteSubmissionId == null || remoteSubmissionId.isBlank()) {
            return null;
        }
        return submissions.values().stream()
                .filter(s -> remoteSubmissionId.equals(s.remoteSubmissionId))
                .findFirst()
                .orElse(null);
    }

    public SubmissionRecord findByAs2MessageId(String as2MessageId) {
        if (as2MessageId == null || as2MessageId.isBlank()) {
            return null;
        }
        return submissions.values().stream()
                .filter(s -> as2MessageId.equals(s.as2MessageId))
                .findFirst()
                .orElse(null);
    }

    /** Matches when any supplied identifier equals a stored batch number or message identifier. */
    public SubmissionRecord findByE2bIdentifier(Set<String> identifiers) {
        if (identifiers == null || identifiers.isEmpty()) {
            return null;
        }
        return submissions.values().stream()
                .filter(s -> (s.e2bMessageIdentifier != null && identifiers.contains(s.e2bMessageIdentifier))
                        || (s.e2bBatchNumber != null && identifiers.contains(s.e2bBatchNumber)))
                .findFirst()
                .orElse(null);
    }

    public SubmissionRecord findByCaseIdInFilename(String filename) {
        if (filename == null || filename.isBlank()) {
            return null;
        }
        String lower = filename.toLowerCase(Locale.ROOT);
        return submissions.values().stream()
                .filter(s -> s.caseId != null && !s.caseId.isBlank())
                .filter(s -> lower.contains(s.caseId.toLowerCase(Locale.ROOT)))
                .findFirst()
                .orElse(null);
    }

    public void putIdempotency(String key, String submissionId) {
        idempotency.put(key, submissionId);
    }

    public String resolveIdempotency(String key) {
        return key == null ? null : idempotency.get(key);
    }

    public int idempotencyCount() {
        return idempotency.size();
    }

    public void putPendingAck(AckForwardTask task) {
        pendingAcks.put(task.id, task);
    }

    public void removePendingAck(String id) {
        pendingAcks.remove(id);
    }

    public List<AckForwardTask> pendingAcks() {
        return new ArrayList<>(pendingAcks.values());
    }

    public int pendingAckCount() {
        return pendingAcks.size();
    }

    public void putOrphanAck(OrphanAck orphan) {
        orphanAcks.put(orphan.id, orphan);
    }

    public List<OrphanAck> orphanAcks() {
        return new ArrayList<>(orphanAcks.values());
    }

    public int orphanAckCount() {
        return orphanAcks.size();
    }

    public void load() {
        if (!Files.exists(file)) {
            return;
        }
        try {
            byte[] bytes = Files.readAllBytes(file);
            if (bytes.length == 0) {
                return;
            }
            Snapshot snapshot = MAPPER.readValue(bytes, Snapshot.class);
            submissions.clear();
            idempotency.clear();
            pendingAcks.clear();
            orphanAcks.clear();
            if (snapshot.submissions != null) {
                submissions.putAll(snapshot.submissions);
            }
            if (snapshot.idempotency != null) {
                idempotency.putAll(snapshot.idempotency);
            }
            if (snapshot.pendingAcks != null) {
                pendingAcks.putAll(snapshot.pendingAcks);
            }
            if (snapshot.orphanAcks != null) {
                orphanAcks.putAll(snapshot.orphanAcks);
            }
        } catch (Exception ex) {
            System.err.printf("failed loading state: %s%n", ex.getMessage());
        }
    }

    public void save() {
        synchronized (lock) {
            try {
                Snapshot snapshot = new Snapshot();
                snapshot.submissions = new HashMap<>(submissions);
                snapshot.idempotency = new HashMap<>(idempotency);
                snapshot.pendingAcks = new HashMap<>(pendingAcks);
                snapshot.orphanAcks = new HashMap<>(orphanAcks);

                byte[] data = MAPPER.writerWithDefaultPrettyPrinter().writeValueAsBytes(snapshot);
                Path parent = file.toAbsolutePath().getParent();
                if (parent != null) {
                    Files.createDirectories(parent);
                }
                Path tmp = file.resolveSibling(file.getFileName() + ".tmp");
                Files.write(tmp, data);
                Files.move(tmp, file, StandardCopyOption.REPLACE_EXISTING);
            } catch (Exception ex) {
                System.err.printf("failed saving state: %s%n", ex.getMessage());
            }
        }
    }
}
```

- [ ] **Step 5: Run the test to verify it passes**

```bash
mvn -q -o test -Dtest=StoreTest
```

Expected: `Tests run: 9, Failures: 0, Errors: 0`.

- [ ] **Step 6: Point `Main` at the new store**

In `src/main/java/com/example/as2/Main.java`: delete the inner classes
`SubmissionRecord`, `AckForwardTask` and `StateSnapshot`; delete the static
fields `SUBMISSIONS`, `IDEMPOTENCY`, `PENDING_ACKS` and `STATE_FILE`; delete the
methods `loadState` and `saveStateUnsafe`. Add:

```java
    private static final Store STORE = new Store(Path.of(
            Env.getOrDefault("AS2_STATE_FILE", "./as2-state.json")));
```

with `import com.example.as2.state.Store;`, `import com.example.as2.state.SubmissionRecord;`,
`import com.example.as2.state.AckForwardTask;`. Replace every `SUBMISSIONS.get(x)`
with `STORE.getSubmission(x)`, `SUBMISSIONS.put(r.submissionId, r)` with
`STORE.putSubmission(r)`, `IDEMPOTENCY.get`/`put` with
`STORE.resolveIdempotency`/`putIdempotency`, `PENDING_ACKS.put(t.id, t)` with
`STORE.putPendingAck(t)`, `PENDING_ACKS.remove(id)` with
`STORE.removePendingAck(id)`, `PENDING_ACKS.values()` with `STORE.pendingAcks()`,
`PENDING_ACKS.size()` with `STORE.pendingAckCount()`, `saveStateUnsafe()` with
`STORE.save()` and `loadState()` with `STORE.load()`. The `synchronized (LOCK)`
blocks around store mutations can go — `Store` locks internally on save and uses
concurrent maps for the rest.

Update `handleInternalStatus` to report `STORE.submissionCount()`,
`STORE.idempotencyCount()`, `STORE.pendingAckCount()` and add
`"orphan_acks", STORE.orphanAckCount()`.

- [ ] **Step 7: Verify the whole suite passes**

```bash
mvn -q -o test
```

Expected: BUILD SUCCESS.

- [ ] **Step 8: Commit**

```bash
git add src/main/java/com/example/as2/state/ \
        src/main/java/com/example/as2/Main.java \
        src/test/java/com/example/as2/state/StoreTest.java
git commit -m "refactor: extract state package from Main

Adds the lookups correlation needs (AS2 Message-ID, E2B batch number and
message identifier, filename) and orphan ACK retention, so an inbound
acknowledgement that matches nothing is kept rather than dropped."
```

---

## Task 8: `as2/Mdn` — build, parse, verify

**Files:**
- Create: `src/main/java/com/example/as2/as2/Mdn.java`
- Test: `src/test/java/com/example/as2/as2/MdnTest.java` (create)

- [ ] **Step 1: Write the failing test**

Create `src/test/java/com/example/as2/as2/MdnTest.java`:

```java
package com.example.as2.as2;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.example.as2.testsupport.TestCerts;
import jakarta.mail.internet.MimeBodyPart;
import java.nio.charset.StandardCharsets;
import java.security.PrivateKey;
import java.security.cert.X509Certificate;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;

class MdnTest {
    private static PrivateKey key;
    private static X509Certificate cert;

    @BeforeAll
    static void load() throws Exception {
        TestCerts.requireGenerated();
        key = TestCerts.privateKey("partner");
        cert = TestCerts.certificate("partner");
    }

    private static Mdn.Request request() {
        return new Mdn.Request(
                "FDA_AERS",
                "E2BR3-SUBMITTER",
                "<original@as2-submitter>",
                "abc123digest=",
                "sha-256",
                Mdn.Disposition.PROCESSED,
                null,
                "as2-submitter/1.0");
    }

    @Test
    void buildProducesAMultipartReport() throws Exception {
        MimeBodyPart mdn = Mdn.build(request());
        String type = mdn.getContentType().toLowerCase();
        assertTrue(type.startsWith("multipart/report"));
        assertTrue(type.contains("report-type=disposition-notification"));
    }

    @Test
    void buildEmbedsTheOriginalMessageIdAndMic() throws Exception {
        String serialized = new String(Cms.entityBytes(Mdn.build(request())), StandardCharsets.UTF_8);
        assertTrue(serialized.contains("Original-Message-ID: <original@as2-submitter>"));
        assertTrue(serialized.contains("Received-Content-MIC: abc123digest=, sha-256"));
        assertTrue(serialized.contains("Final-Recipient: rfc822; FDA_AERS"));
    }

    @Test
    void buildMarksAutomaticProcessedDisposition() throws Exception {
        String serialized = new String(Cms.entityBytes(Mdn.build(request())), StandardCharsets.UTF_8);
        assertTrue(serialized.contains(
                "Disposition: automatic-action/MDN-sent-automatically; processed"));
    }

    @Test
    void buildRendersAFailureDisposition() throws Exception {
        Mdn.Request failed = new Mdn.Request(
                "FDA_AERS", "E2BR3-SUBMITTER", "<original@as2-submitter>", null, "sha-256",
                Mdn.Disposition.FAILED, "decryption-failed", "as2-submitter/1.0");
        String serialized = new String(Cms.entityBytes(Mdn.build(failed)), StandardCharsets.UTF_8);
        assertTrue(serialized.contains(
                "Disposition: automatic-action/MDN-sent-automatically; failed/Failure: decryption-failed"));
    }

    @Test
    void parseReadsAnUnsignedMdnWeJustBuilt() throws Exception {
        MimeBodyPart mdn = Mdn.build(request());
        Mdn.Parsed parsed = Mdn.parse(mdn.getContentType(), Cms.entityBytes(mdn));
        assertTrue(parsed.isMdn());
        assertTrue(parsed.success());
        assertEquals("<original@as2-submitter>", parsed.originalMessageId());
        assertEquals("abc123digest=", parsed.receivedMic());
        assertEquals("sha-256", parsed.micAlgorithm());
    }

    @Test
    void parseTreatsFailedDispositionAsUnsuccessful() throws Exception {
        Mdn.Request failed = new Mdn.Request(
                "FDA_AERS", "E2BR3-SUBMITTER", "<m@x>", null, "sha-256",
                Mdn.Disposition.FAILED, "virus-detected", "ua");
        MimeBodyPart mdn = Mdn.build(failed);
        Mdn.Parsed parsed = Mdn.parse(mdn.getContentType(), Cms.entityBytes(mdn));
        assertTrue(parsed.isMdn());
        assertFalse(parsed.success());
        assertTrue(parsed.disposition().contains("virus-detected"));
    }

    @Test
    void parseTreatsProcessedErrorAsUnsuccessful() throws Exception {
        Mdn.Request errored = new Mdn.Request(
                "FDA_AERS", "E2BR3-SUBMITTER", "<m@x>", null, "sha-256",
                Mdn.Disposition.PROCESSED_ERROR, "authentication-failed", "ua");
        Mdn.Parsed parsed = Mdn.parse(
                Mdn.build(errored).getContentType(), Cms.entityBytes(Mdn.build(errored)));
        assertFalse(parsed.success());
    }

    @Test
    void parseRejectsSomethingThatIsNotAnMdn() throws Exception {
        Mdn.Parsed parsed = Mdn.parse("application/xml", "<MCCI_IN200100UV01/>".getBytes(StandardCharsets.UTF_8));
        assertFalse(parsed.isMdn());
        assertNull(parsed.originalMessageId());
    }

    @Test
    void signedMdnRoundTripsThroughVerification() throws Exception {
        MimeBodyPart signed = Cms.sign(Mdn.build(request()), key, cert, "sha-256");
        MimeBodyPart content = Cms.verify(signed, cert);
        Mdn.Parsed parsed = Mdn.parse(content.getContentType(), Cms.entityBytes(content));
        assertTrue(parsed.isMdn());
        assertEquals("<original@as2-submitter>", parsed.originalMessageId());
    }

    @Test
    void micMatchesWhenTheDigestsAgree() {
        assertTrue(Mdn.micMatches("abc=", "abc="));
        assertTrue(Mdn.micMatches("ABC=", "abc="));
        assertFalse(Mdn.micMatches("abc=", "def="));
        assertFalse(Mdn.micMatches(null, "abc="));
        assertFalse(Mdn.micMatches("abc=", null));
    }
}
```

- [ ] **Step 2: Run it to verify it fails**

```bash
mvn -q -o test -Dtest=MdnTest
```

Expected: COMPILATION ERROR — `cannot find symbol: class Mdn`.

- [ ] **Step 3: Write the implementation**

Create `src/main/java/com/example/as2/as2/Mdn.java`:

```java
package com.example.as2.as2;

import jakarta.mail.internet.InternetHeaders;
import jakarta.mail.internet.MimeBodyPart;
import jakarta.mail.internet.MimeMultipart;
import jakarta.mail.util.ByteArrayDataSource;
import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.nio.charset.StandardCharsets;
import java.util.Locale;

/**
 * Message Disposition Notification handling per RFC 4130 and RFC 3798.
 *
 * <p>An MDN is the AS2 receipt and, for FDA ESG, is ACK1.
 */
public final class Mdn {
    private Mdn() {}

    /** The disposition outcomes we emit. */
    public enum Disposition {
        PROCESSED,
        PROCESSED_WARNING,
        PROCESSED_ERROR,
        FAILED
    }

    /** Everything needed to render an MDN. */
    public record Request(
            String reportingAs2Id,
            String recipientAs2Id,
            String originalMessageId,
            String receivedMic,
            String micAlgorithm,
            Disposition disposition,
            String detail,
            String userAgent) {}

    /** The outcome of reading an inbound entity that may or may not be an MDN. */
    public record Parsed(
            boolean isMdn,
            boolean success,
            String disposition,
            String originalMessageId,
            String receivedMic,
            String micAlgorithm) {

        static Parsed notAnMdn() {
            return new Parsed(false, false, null, null, null, null);
        }
    }

    public static MimeBodyPart build(Request request) throws Exception {
        MimeMultipart report = new MimeMultipart("report; report-type=disposition-notification");

        MimeBodyPart human = new MimeBodyPart();
        human.setText(humanText(request), "UTF-8");
        human.setHeader("Content-Type", "text/plain; charset=UTF-8");
        report.addBodyPart(human);

        MimeBodyPart machine = new MimeBodyPart();
        machine.setDataHandler(new jakarta.activation.DataHandler(new ByteArrayDataSource(
                machineText(request).getBytes(StandardCharsets.UTF_8),
                "message/disposition-notification")));
        machine.setHeader("Content-Type", "message/disposition-notification");
        report.addBodyPart(machine);

        MimeBodyPart mdn = new MimeBodyPart();
        mdn.setContent(report);
        mdn.setHeader("Content-Type", report.getContentType());
        return mdn;
    }

    private static String humanText(Request request) {
        StringBuilder sb = new StringBuilder();
        sb.append("The message sent to ").append(request.reportingAs2Id())
                .append(" with Message-ID ").append(request.originalMessageId());
        if (request.disposition() == Disposition.PROCESSED) {
            sb.append(" has been received and processed.\r\n");
        } else {
            sb.append(" could not be processed: ")
                    .append(request.detail() == null ? "unspecified-error" : request.detail())
                    .append("\r\n");
        }
        return sb.toString();
    }

    private static String machineText(Request request) {
        StringBuilder sb = new StringBuilder();
        sb.append("Reporting-UA: ").append(request.userAgent()).append("\r\n");
        sb.append("Original-Recipient: rfc822; ").append(request.reportingAs2Id()).append("\r\n");
        sb.append("Final-Recipient: rfc822; ").append(request.reportingAs2Id()).append("\r\n");
        sb.append("Original-Message-ID: ").append(request.originalMessageId()).append("\r\n");
        sb.append("Disposition: automatic-action/MDN-sent-automatically; ")
                .append(dispositionText(request)).append("\r\n");
        if (request.receivedMic() != null && !request.receivedMic().isBlank()) {
            sb.append("Received-Content-MIC: ").append(request.receivedMic())
                    .append(", ").append(request.micAlgorithm()).append("\r\n");
        }
        return sb.toString();
    }

    private static String dispositionText(Request request) {
        String detail = request.detail() == null ? "unspecified-error" : request.detail();
        return switch (request.disposition()) {
            case PROCESSED -> "processed";
            case PROCESSED_WARNING -> "processed/Warning: " + detail;
            case PROCESSED_ERROR -> "processed/Error: " + detail;
            case FAILED -> "failed/Failure: " + detail;
        };
    }

    /** Reads an entity that may be an MDN. Never throws on malformed input. */
    public static Parsed parse(String contentType, byte[] body) {
        String type = contentType == null ? "" : contentType.toLowerCase(Locale.ROOT);
        String text = new String(body, StandardCharsets.UTF_8);
        boolean looksLikeMdn = type.contains("multipart/report")
                || type.contains("message/disposition-notification")
                || text.toLowerCase(Locale.ROOT).contains("disposition-notification");
        if (!looksLikeMdn) {
            return Parsed.notAnMdn();
        }

        String machine = extractMachinePart(contentType, body);
        String disposition = headerValue(machine, "disposition");
        String originalMessageId = headerValue(machine, "original-message-id");
        String rawMic = headerValue(machine, "received-content-mic");

        if (disposition == null && originalMessageId == null) {
            return Parsed.notAnMdn();
        }

        Mic.Parsed mic = rawMic == null ? null : Mic.parseHeaderValue(rawMic);
        return new Parsed(
                true,
                isSuccess(disposition),
                disposition,
                originalMessageId,
                mic == null ? null : mic.digest(),
                mic == null ? null : mic.algorithm());
    }

    private static boolean isSuccess(String disposition) {
        if (disposition == null) {
            return false;
        }
        String d = disposition.toLowerCase(Locale.ROOT);
        if (d.contains("failed") || d.contains("/error") || d.contains("error:")) {
            return false;
        }
        return d.contains("processed");
    }

    /**
     * Pulls the message/disposition-notification part out of a multipart/report,
     * falling back to treating the whole body as the machine part.
     */
    private static String extractMachinePart(String contentType, byte[] body) {
        try {
            if (contentType != null && contentType.toLowerCase(Locale.ROOT).contains("multipart/")) {
                MimeMultipart multipart = new MimeMultipart(new ByteArrayDataSource(body, contentType));
                for (int i = 0; i < multipart.getCount(); i++) {
                    jakarta.mail.BodyPart part = multipart.getBodyPart(i);
                    String partType = part.getContentType() == null
                            ? "" : part.getContentType().toLowerCase(Locale.ROOT);
                    if (partType.contains("disposition-notification")) {
                        ByteArrayOutputStream out = new ByteArrayOutputStream();
                        part.getDataHandler().writeTo(out);
                        return out.toString(StandardCharsets.UTF_8);
                    }
                }
            }
        } catch (Exception ignored) {
            // fall through to the raw-body path
        }
        return new String(body, StandardCharsets.UTF_8);
    }

    private static String headerValue(String text, String headerName) {
        try {
            InternetHeaders headers = new InternetHeaders(
                    new ByteArrayInputStream(text.getBytes(StandardCharsets.UTF_8)));
            String[] values = headers.getHeader(headerName);
            if (values != null && values.length > 0) {
                return values[0].trim();
            }
        } catch (Exception ignored) {
            // fall through to the line scan
        }
        String needle = headerName.toLowerCase(Locale.ROOT) + ":";
        for (String line : text.split("\\r?\\n")) {
            if (line.toLowerCase(Locale.ROOT).startsWith(needle)) {
                return line.substring(needle.length()).trim();
            }
        }
        return null;
    }

    /** Case-insensitive comparison of two base64 MIC digests. */
    public static boolean micMatches(String expected, String received) {
        return expected != null && received != null && expected.equalsIgnoreCase(received);
    }
}
```

- [ ] **Step 4: Run the test to verify it passes**

```bash
mvn -q -o test -Dtest=MdnTest
```

Expected: `Tests run: 10, Failures: 0, Errors: 0`.

- [ ] **Step 5: Commit**

```bash
git add src/main/java/com/example/as2/as2/Mdn.java src/test/java/com/example/as2/as2/MdnTest.java
git commit -m "feat: build, parse and verify MDNs

Replaces lowercase string matching over the response body with real
multipart/report handling, so a disposition of processed/Error is no
longer read as success."
```

---

## Task 9: `config/FdaRouting` — the three routing options

**Files:**
- Create: `src/main/java/com/example/as2/config/FdaRouting.java`
- Test: `src/test/java/com/example/as2/config/FdaRoutingTest.java` (create)

- [ ] **Step 1: Write the failing test**

Create `src/test/java/com/example/as2/config/FdaRoutingTest.java`:

```java
package com.example.as2.config;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.Map;
import org.junit.jupiter.api.Test;

class FdaRoutingTest {
    @Test
    void routingIdModeUsesFdaAersAndNoCycloneHeaders() {
        FdaRouting r = FdaRouting.resolve("routing_id", "test");
        assertEquals("FDA_AERS", r.as2To());
        assertTrue(r.extraHeaders().isEmpty());
        assertEquals("ZZFDA", r.encryptionCertificateName());
        assertEquals("FDA_AERS", r.expectedAckAs2From());
    }

    @Test
    void routingIdModeIsIdenticalInProduction() {
        FdaRouting prod = FdaRouting.resolve("routing_id", "prod");
        assertEquals("FDA_AERS", prod.as2To());
        assertEquals("ZZFDA", prod.encryptionCertificateName());
    }

    @Test
    void metadataHeadersModeCarriesCenterAndSubmissionType() {
        FdaRouting r = FdaRouting.resolve("metadata_headers", "prod");
        assertEquals("ZZFDA", r.as2To());
        Map<String, String> headers = r.extraHeaders();
        assertEquals("CDER", headers.get("X-Cyclone-Metadata-FdaCenter"));
        assertEquals("AERS", headers.get("X-Cyclone-Metadata-FdaSubmissionType"));
        assertFalse(headers.containsKey("X-Cyclone-True-Receiver"));
        assertEquals("ZZFDA", r.encryptionCertificateName());
        assertEquals("ZZFDA", r.expectedAckAs2From());
    }

    @Test
    void metadataHeadersModeSwitchesToZzfdatstForTest() {
        FdaRouting r = FdaRouting.resolve("metadata_headers", "test");
        assertEquals("ZZFDATST", r.as2To());
        assertEquals("ZZFDATST", r.encryptionCertificateName());
        assertEquals("ZZFDATST", r.expectedAckAs2From());
    }

    @Test
    void trueReceiverModeCarriesOnlyTheTrueReceiverHeader() {
        FdaRouting r = FdaRouting.resolve("true_receiver", "test");
        assertEquals("ZZFDATST", r.as2To());
        Map<String, String> headers = r.extraHeaders();
        assertEquals("FDA_AERS", headers.get("X-Cyclone-True-Receiver"));
        assertFalse(headers.containsKey("X-Cyclone-Metadata-FdaCenter"));
        assertFalse(headers.containsKey("X-Cyclone-Metadata-FdaSubmissionType"));
        assertEquals("ZZFDA", r.encryptionCertificateName());
        assertEquals("FDA_AERS", r.expectedAckAs2From());
    }

    @Test
    void endpointFollowsTheEnvironment() {
        assertEquals(
                "https://upload-api-esgng.fda.gov:4080/as2/receive",
                FdaRouting.resolve("routing_id", "prod").defaultEndpointUrl());
        assertEquals(
                "https://upload-api-esgng.fda.gov:4080/as2/receive/test",
                FdaRouting.resolve("routing_id", "test").defaultEndpointUrl());
    }

    @Test
    void modeDefaultsToRoutingId() {
        assertEquals("FDA_AERS", FdaRouting.resolve(null, "test").as2To());
        assertEquals("FDA_AERS", FdaRouting.resolve("  ", "test").as2To());
    }

    @Test
    void environmentDefaultsToTest() {
        assertEquals("ZZFDATST", FdaRouting.resolve("metadata_headers", null).as2To());
    }

    @Test
    void unknownModeIsRejected() {
        IllegalStateException ex = assertThrows(
                IllegalStateException.class, () -> FdaRouting.resolve("cyclone_magic", "test"));
        assertTrue(ex.getMessage().contains("AS2_FDA_ROUTING_MODE"));
    }

    @Test
    void unknownEnvironmentIsRejected() {
        assertThrows(IllegalStateException.class, () -> FdaRouting.resolve("routing_id", "staging"));
    }
}
```

- [ ] **Step 2: Run it to verify it fails**

```bash
mvn -q -o test -Dtest=FdaRoutingTest
```

Expected: COMPILATION ERROR — `cannot find symbol: class FdaRouting`.

- [ ] **Step 3: Write the implementation**

Create `src/main/java/com/example/as2/config/FdaRouting.java`:

```java
package com.example.as2.config;

import java.util.LinkedHashMap;
import java.util.Locale;
import java.util.Map;

/**
 * The three routing options FDA documents for AS2 submitters, resolved for the
 * CDER/AERS submission type.
 *
 * <p>Source: FDA ESG NextGen AS2 Guide for Industry Users v2.1 section 2.4. The
 * options are mutually exclusive — the guide marks the unused headers of each
 * option "DO NOT USE" — so each mode emits exactly one header set and nothing
 * else. v2.2 dropped "Metadata" from the true-receiver header name, so the
 * current spelling {@code X-Cyclone-True-Receiver} is used here.
 */
public record FdaRouting(
        Mode mode,
        Environment environment,
        String as2To,
        Map<String, String> extraHeaders,
        String encryptionCertificateName,
        String expectedAckAs2From) {

    public enum Mode {
        ROUTING_ID,
        METADATA_HEADERS,
        TRUE_RECEIVER
    }

    public enum Environment {
        PROD,
        TEST
    }

    private static final String PROD_ENDPOINT = "https://upload-api-esgng.fda.gov:4080/as2/receive";
    private static final String TEST_ENDPOINT = "https://upload-api-esgng.fda.gov:4080/as2/receive/test";

    public static FdaRouting resolve(String rawMode, String rawEnvironment) {
        Mode mode = parseMode(rawMode);
        Environment environment = parseEnvironment(rawEnvironment);
        boolean prod = environment == Environment.PROD;

        return switch (mode) {
            case ROUTING_ID -> new FdaRouting(
                    mode, environment, "FDA_AERS", Map.of(), "ZZFDA", "FDA_AERS");
            case METADATA_HEADERS -> {
                String gateway = prod ? "ZZFDA" : "ZZFDATST";
                Map<String, String> headers = new LinkedHashMap<>();
                headers.put("X-Cyclone-Metadata-FdaCenter", "CDER");
                headers.put("X-Cyclone-Metadata-FdaSubmissionType", "AERS");
                yield new FdaRouting(mode, environment, gateway, Map.copyOf(headers), gateway, gateway);
            }
            case TRUE_RECEIVER -> {
                String gateway = prod ? "ZZFDA" : "ZZFDATST";
                yield new FdaRouting(
                        mode,
                        environment,
                        gateway,
                        Map.of("X-Cyclone-True-Receiver", "FDA_AERS"),
                        "ZZFDA",
                        "FDA_AERS");
            }
        };
    }

    public String defaultEndpointUrl() {
        return environment == Environment.PROD ? PROD_ENDPOINT : TEST_ENDPOINT;
    }

    private static Mode parseMode(String raw) {
        String value = Env.normalize(raw);
        if (value == null) {
            return Mode.ROUTING_ID;
        }
        return switch (value.toLowerCase(Locale.ROOT)) {
            case "routing_id" -> Mode.ROUTING_ID;
            case "metadata_headers" -> Mode.METADATA_HEADERS;
            case "true_receiver" -> Mode.TRUE_RECEIVER;
            default -> throw new IllegalStateException(
                    "AS2_FDA_ROUTING_MODE must be routing_id, metadata_headers or true_receiver, got: " + raw);
        };
    }

    private static Environment parseEnvironment(String raw) {
        String value = Env.normalize(raw);
        if (value == null) {
            return Environment.TEST;
        }
        return switch (value.toLowerCase(Locale.ROOT)) {
            case "prod", "production" -> Environment.PROD;
            case "test", "preprod" -> Environment.TEST;
            default -> throw new IllegalStateException(
                    "AS2_FDA_ENV must be prod or test, got: " + raw);
        };
    }
}
```

- [ ] **Step 4: Run the test to verify it passes**

```bash
mvn -q -o test -Dtest=FdaRoutingTest
```

Expected: `Tests run: 10, Failures: 0, Errors: 0`.

- [ ] **Step 5: Commit**

```bash
git add src/main/java/com/example/as2/config/FdaRouting.java \
        src/test/java/com/example/as2/config/FdaRoutingTest.java
git commit -m "feat: FDA AS2 routing modes for CDER/AERS

Encodes the three mutually exclusive routing options from the ESG
NextGen AS2 guide. Each mode emits exactly one header set, so the
combinations the guide marks DO NOT USE cannot be produced."
```

---

## Task 10: `as2/E2bIdentifiers` — correlation keys from the payload

E2B(R3) is HL7 v3, not the R2 `<ichicsr>` structure, so the identifiers are
attributes on `id` elements rather than elements of their own.

| Field | XPath |
| --- | --- |
| N.1.2 Batch Number | `/MCCI_IN200100UV01/id/@extension` |
| N.2.r.1 Message Identifier | `/MCCI_IN200100UV01/PORR_IN049016UV/id/@extension` |

Reading an acknowledgement takes the opposite approach: rather than hard-coding
a path into a schema we cannot verify from an official source, collect **every**
`extension` attribute of every `id` element and let the correlator look for any
match.

**Files:**
- Create: `src/main/java/com/example/as2/as2/E2bIdentifiers.java`
- Test: `src/test/java/com/example/as2/as2/E2bIdentifiersTest.java` (create)

- [ ] **Step 1: Write the failing test**

Create `src/test/java/com/example/as2/as2/E2bIdentifiersTest.java`:

```java
package com.example.as2.as2;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.util.Set;
import org.junit.jupiter.api.Test;

class E2bIdentifiersTest {
    private static final String SUBMISSION =
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>"
            + "<MCCI_IN200100UV01 xmlns=\"urn:hl7-org:v3\">"
            + "  <id extension=\"BATCH-2026-0001\" root=\"2.16.840.1.113883.3.989.2.1.3.22\"/>"
            + "  <creationTime value=\"20260727101010-0500\"/>"
            + "  <PORR_IN049016UV>"
            + "    <id extension=\"MSG-2026-0001\" root=\"2.16.840.1.113883.3.989.2.1.3.1\"/>"
            + "    <interactionId extension=\"PORR_IN049016UV\" root=\"2.16.840.1.113883.1.6\"/>"
            + "  </PORR_IN049016UV>"
            + "</MCCI_IN200100UV01>";

    @Test
    void extractsBatchNumberFromTheRootId() {
        assertEquals("BATCH-2026-0001", E2bIdentifiers.fromSubmission(SUBMISSION).batchNumber());
    }

    @Test
    void extractsMessageIdentifierFromTheNestedId() {
        assertEquals("MSG-2026-0001", E2bIdentifiers.fromSubmission(SUBMISSION).messageIdentifier());
    }

    @Test
    void ignoresInteractionIdWhichIsNotAnIdElement() {
        assertEquals("MSG-2026-0001", E2bIdentifiers.fromSubmission(SUBMISSION).messageIdentifier());
    }

    @Test
    void returnsEmptyForNonXmlWithoutThrowing() {
        E2bIdentifiers ids = E2bIdentifiers.fromSubmission("not xml at all");
        assertNull(ids.batchNumber());
        assertNull(ids.messageIdentifier());
    }

    @Test
    void returnsEmptyForNullWithoutThrowing() {
        assertNull(E2bIdentifiers.fromSubmission(null).batchNumber());
    }

    @Test
    void collectsEveryIdExtensionFromAnAcknowledgement() {
        String ack =
                "<MCCI_IN200101UV01 xmlns=\"urn:hl7-org:v3\">"
                + "  <id extension=\"ACK-9999\" root=\"2.16.840.1.113883.3.989.2.1.3.22\"/>"
                + "  <PORR_IN049016UV>"
                + "    <id extension=\"BATCH-2026-0001\" root=\"x\"/>"
                + "    <acknowledgement>"
                + "      <targetMessage><id extension=\"MSG-2026-0001\" root=\"y\"/></targetMessage>"
                + "    </acknowledgement>"
                + "  </PORR_IN049016UV>"
                + "</MCCI_IN200101UV01>";
        Set<String> found = E2bIdentifiers.allIdExtensions(ack.getBytes(StandardCharsets.UTF_8));
        assertTrue(found.contains("ACK-9999"));
        assertTrue(found.contains("BATCH-2026-0001"));
        assertTrue(found.contains("MSG-2026-0001"));
        assertEquals(3, found.size());
    }

    @Test
    void allIdExtensionsReturnsEmptyForBinaryGarbage() {
        assertTrue(E2bIdentifiers.allIdExtensions(new byte[] {0, 1, 2}).isEmpty());
    }

    @Test
    void allIdExtensionsIgnoresNamespacePrefixes() {
        String ack =
                "<hl7:MCCI_IN200101UV01 xmlns:hl7=\"urn:hl7-org:v3\">"
                + "  <hl7:id extension=\"PREFIXED-1\" root=\"x\"/>"
                + "</hl7:MCCI_IN200101UV01>";
        assertTrue(E2bIdentifiers.allIdExtensions(ack.getBytes(StandardCharsets.UTF_8))
                .contains("PREFIXED-1"));
    }
}
```

- [ ] **Step 2: Run it to verify it fails**

```bash
mvn -q -o test -Dtest=E2bIdentifiersTest
```

Expected: COMPILATION ERROR — `cannot find symbol: class E2bIdentifiers`.

- [ ] **Step 3: Write the implementation**

Create `src/main/java/com/example/as2/as2/E2bIdentifiers.java`:

```java
package com.example.as2.as2;

import java.io.ByteArrayInputStream;
import java.nio.charset.StandardCharsets;
import java.util.LinkedHashSet;
import java.util.Set;
import javax.xml.XMLConstants;
import javax.xml.parsers.DocumentBuilder;
import javax.xml.parsers.DocumentBuilderFactory;
import org.w3c.dom.Document;
import org.w3c.dom.Element;
import org.w3c.dom.Node;
import org.w3c.dom.NodeList;

/**
 * Correlation keys read out of E2B(R3) HL7 v3 messages.
 *
 * <p>Parsing never throws: a payload we cannot read simply yields no keys, and
 * correlation falls through to the next strategy.
 */
public record E2bIdentifiers(String batchNumber, String messageIdentifier) {

    private static final E2bIdentifiers EMPTY = new E2bIdentifiers(null, null);

    /** Reads N.1.2 Batch Number and N.2.r.1 Message Identifier from an outbound submission. */
    public static E2bIdentifiers fromSubmission(String xml) {
        if (xml == null || xml.isBlank()) {
            return EMPTY;
        }
        try {
            Element root = parse(xml.getBytes(StandardCharsets.UTF_8)).getDocumentElement();
            if (root == null) {
                return EMPTY;
            }
            String batchNumber = extensionOfFirstChildId(root);
            String messageIdentifier = null;
            NodeList children = root.getChildNodes();
            for (int i = 0; i < children.getLength(); i++) {
                if (children.item(i) instanceof Element element
                        && localName(element).startsWith("PORR_IN")) {
                    messageIdentifier = extensionOfFirstChildId(element);
                    break;
                }
            }
            return new E2bIdentifiers(batchNumber, messageIdentifier);
        } catch (Exception ex) {
            return EMPTY;
        }
    }

    /** Every {@code extension} attribute on every {@code id} element, at any depth. */
    public static Set<String> allIdExtensions(byte[] xml) {
        Set<String> found = new LinkedHashSet<>();
        if (xml == null || xml.length == 0) {
            return found;
        }
        try {
            collect(parse(xml).getDocumentElement(), found);
        } catch (Exception ignored) {
            // not XML; no keys
        }
        return found;
    }

    private static void collect(Node node, Set<String> found) {
        if (!(node instanceof Element element)) {
            return;
        }
        if (localName(element).equals("id")) {
            String extension = element.getAttribute("extension");
            if (extension != null && !extension.isBlank()) {
                found.add(extension.trim());
            }
        }
        NodeList children = element.getChildNodes();
        for (int i = 0; i < children.getLength(); i++) {
            collect(children.item(i), found);
        }
    }

    private static String extensionOfFirstChildId(Element parent) {
        NodeList children = parent.getChildNodes();
        for (int i = 0; i < children.getLength(); i++) {
            if (children.item(i) instanceof Element element && localName(element).equals("id")) {
                String extension = element.getAttribute("extension");
                return extension == null || extension.isBlank() ? null : extension.trim();
            }
        }
        return null;
    }

    private static String localName(Element element) {
        String local = element.getLocalName();
        if (local != null) {
            return local;
        }
        String name = element.getNodeName();
        int colon = name.indexOf(':');
        return colon < 0 ? name : name.substring(colon + 1);
    }

    private static Document parse(byte[] xml) throws Exception {
        DocumentBuilderFactory factory = DocumentBuilderFactory.newInstance();
        factory.setNamespaceAware(true);
        factory.setAttribute(XMLConstants.ACCESS_EXTERNAL_DTD, "");
        factory.setAttribute(XMLConstants.ACCESS_EXTERNAL_SCHEMA, "");
        factory.setFeature("http://apache.org/xml/features/disallow-doctype-decl", true);
        factory.setExpandEntityReferences(false);
        DocumentBuilder builder = factory.newDocumentBuilder();
        return builder.parse(new ByteArrayInputStream(xml));
    }
}
```

The parser is locked down — no DOCTYPE, no external entities — because inbound
acknowledgement bytes come from the network.

- [ ] **Step 4: Run the test to verify it passes**

```bash
mvn -q -o test -Dtest=E2bIdentifiersTest
```

Expected: `Tests run: 8, Failures: 0, Errors: 0`.

- [ ] **Step 5: Commit**

```bash
git add src/main/java/com/example/as2/as2/E2bIdentifiers.java \
        src/test/java/com/example/as2/as2/E2bIdentifiersTest.java
git commit -m "feat: read E2B(R3) correlation keys from HL7 v3 payloads

Batch number and message identifier are attributes on id elements, not
R2-style elements. Acknowledgement matching collects every id extension
rather than assuming a path into an unverified ack schema."
```

---

## Task 11: `ack/AckClassifier`

**Files:**
- Create: `src/main/java/com/example/as2/as2/InboundMessage.java`
- Create: `src/main/java/com/example/as2/ack/AckDecision.java`
- Create: `src/main/java/com/example/as2/ack/AckClassifier.java`
- Test: `src/test/java/com/example/as2/ack/AckClassifierTest.java` (create)

- [ ] **Step 1: Write the failing test**

Create `src/test/java/com/example/as2/ack/AckClassifierTest.java`:

```java
package com.example.as2.ack;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.example.as2.as2.InboundMessage;
import com.example.as2.as2.Mdn;
import java.nio.charset.StandardCharsets;
import java.util.Map;
import org.junit.jupiter.api.Test;

class AckClassifierTest {
    private static InboundMessage file(String filename, String body) {
        return new InboundMessage(
                Map.of("as2-from", "FDA_AERS"),
                "FDA_AERS",
                "E2BR3-SUBMITTER",
                "<inbound@fda>",
                "application/octet-stream",
                filename,
                body.getBytes(StandardCharsets.UTF_8),
                null);
    }

    private static InboundMessage mdn(Mdn.Parsed parsed) {
        return new InboundMessage(
                Map.of("as2-from", "FDA_AERS"),
                "FDA_AERS",
                "E2BR3-SUBMITTER",
                "<mdn@fda>",
                "multipart/report; report-type=disposition-notification",
                null,
                new byte[0],
                parsed);
    }

    @Test
    void anMdnIsAckLevelOne() {
        AckDecision decision = AckClassifier.classify(
                mdn(new Mdn.Parsed(true, true, "processed", "<orig@x>", "abc=", "sha-256")));
        assertEquals(1, decision.level());
        assertTrue(decision.success());
        assertEquals("ACK1_MDN_PROCESSED", decision.code());
    }

    @Test
    void aFailedMdnIsAckOneUnsuccessful() {
        AckDecision decision = AckClassifier.classify(
                mdn(new Mdn.Parsed(true, false, "failed/Failure: virus-detected", "<orig@x>", null, null)));
        assertEquals(1, decision.level());
        assertFalse(decision.success());
        assertEquals("ACK1_MDN_FAILED", decision.code());
        assertTrue(decision.message().contains("virus-detected"));
    }

    @Test
    void anAckExtensionIsLevelThree() {
        AckDecision decision = AckClassifier.classify(file("CDER_AERS_20260727.ack", "<MCCI_IN200101UV01/>"));
        assertEquals(3, decision.level());
        assertTrue(decision.success());
    }

    @Test
    void aTxtExtensionIsLevelTwo() {
        assertEquals(2, AckClassifier.classify(file("receipt.txt", "ok")).level());
    }

    @Test
    void anUnknownExtensionFallsBackToLevelThree() {
        assertEquals(3, AckClassifier.classify(file("something.bin", "data")).level());
    }

    @Test
    void aMissingFilenameFallsBackToLevelThree() {
        assertEquals(3, AckClassifier.classify(file(null, "data")).level());
    }

    @Test
    void extensionMatchingIsCaseInsensitive() {
        assertEquals(3, AckClassifier.classify(file("UPPER.ACK", "x")).level());
        assertEquals(2, AckClassifier.classify(file("UPPER.TXT", "x")).level());
    }

    @Test
    void aVirusExceptionIsAckOneUnsuccessful() {
        AckDecision decision = AckClassifier.classify(
                file("exception.ack", "ACK1a virus detected in submission"));
        assertEquals(1, decision.level());
        assertFalse(decision.success());
        assertEquals("ACK1A_VIRUS_DETECTED", decision.code());
    }

    @Test
    void anAcknowledgementReportingRejectionIsUnsuccessful() {
        AckDecision decision = AckClassifier.classify(file(
                "reject.ack",
                "<MCCI_IN200101UV01><acknowledgement typeCode=\"CE\"/></MCCI_IN200101UV01>"));
        assertEquals(3, decision.level());
        assertFalse(decision.success());
    }

    @Test
    void anAcknowledgementReportingAcceptanceIsSuccessful() {
        AckDecision decision = AckClassifier.classify(file(
                "accept.ack",
                "<MCCI_IN200101UV01><acknowledgement typeCode=\"CA\"/></MCCI_IN200101UV01>"));
        assertTrue(decision.success());
    }
}
```

- [ ] **Step 2: Run it to verify it fails**

```bash
mvn -q -o test -Dtest=AckClassifierTest
```

Expected: COMPILATION ERROR — `package com.example.as2.ack does not exist`.

- [ ] **Step 3: Write the inbound message and decision types**

Create `src/main/java/com/example/as2/as2/InboundMessage.java`:

```java
package com.example.as2.as2;

import java.util.Map;

/**
 * A fully unwrapped inbound AS2 message: decrypted, signature-verified, and with
 * its MDN parsed if it was one.
 *
 * @param headers lowercase header names to values
 * @param payload the decrypted, verified content bytes
 * @param mdn the parsed MDN, or null when this is a file delivery
 */
public record InboundMessage(
        Map<String, String> headers,
        String as2From,
        String as2To,
        String messageId,
        String contentType,
        String filename,
        byte[] payload,
        Mdn.Parsed mdn) {

    public boolean isMdn() {
        return mdn != null && mdn.isMdn();
    }
}
```

Create `src/main/java/com/example/as2/ack/AckDecision.java`:

```java
package com.example.as2.ack;

/** What an inbound message means to the backend. */
public record AckDecision(int level, boolean success, String code, String message) {}
```

- [ ] **Step 4: Write the classifier**

Create `src/main/java/com/example/as2/ack/AckClassifier.java`:

```java
package com.example.as2.ack;

import com.example.as2.as2.InboundMessage;
import com.example.as2.config.Env;
import java.nio.charset.StandardCharsets;
import java.util.HashMap;
import java.util.Locale;
import java.util.Map;

/**
 * Turns an inbound AS2 message into an ACK level and outcome.
 *
 * <p>On the CDER/AERS path FDA sends the MDN as ACK1 and a {@code .ack} file as
 * ACK3. ACK2 is never produced for AERS, so the mapping below exists mainly so
 * other submission types do not need a code change.
 */
public final class AckClassifier {
    private static final int DEFAULT_FILE_LEVEL = 3;

    private AckClassifier() {}

    public static AckDecision classify(InboundMessage message) {
        return message.isMdn() ? fromMdn(message) : fromFile(message);
    }

    private static AckDecision fromMdn(InboundMessage message) {
        var mdn = message.mdn();
        return mdn.success()
                ? new AckDecision(1, true, "ACK1_MDN_PROCESSED", mdn.disposition())
                : new AckDecision(1, false, "ACK1_MDN_FAILED", mdn.disposition());
    }

    private static AckDecision fromFile(InboundMessage message) {
        String body = new String(message.payload(), StandardCharsets.UTF_8);
        String lowerBody = body.toLowerCase(Locale.ROOT);

        if (lowerBody.contains("ack1a") || lowerBody.contains("virus")) {
            return new AckDecision(1, false, "ACK1A_VIRUS_DETECTED", firstLine(body));
        }

        int level = levelForFilename(message.filename());
        boolean success = !indicatesRejection(lowerBody);
        String code = "ACK" + level + (success ? "_ACCEPTED" : "_REJECTED");
        return new AckDecision(level, success, code, firstLine(body));
    }

    /**
     * {@code AS2_ACK_LEVEL_BY_EXT} overrides the built-in map, formatted as
     * {@code ack=3,txt=2}. A wrong guess is then a config change, not a release.
     */
    static int levelForFilename(String filename) {
        if (filename == null || filename.isBlank()) {
            return DEFAULT_FILE_LEVEL;
        }
        int dot = filename.lastIndexOf('.');
        if (dot < 0 || dot == filename.length() - 1) {
            return DEFAULT_FILE_LEVEL;
        }
        String extension = filename.substring(dot + 1).toLowerCase(Locale.ROOT);
        return extensionMap().getOrDefault(extension, DEFAULT_FILE_LEVEL);
    }

    private static Map<String, Integer> extensionMap() {
        Map<String, Integer> map = new HashMap<>();
        map.put("ack", 3);
        map.put("txt", 2);

        String configured = Env.get("AS2_ACK_LEVEL_BY_EXT");
        if (configured == null) {
            return map;
        }
        for (String entry : configured.split(",")) {
            String[] pair = entry.split("=", 2);
            if (pair.length != 2) {
                continue;
            }
            String extension = pair[0].trim().toLowerCase(Locale.ROOT).replaceFirst("^[.]", "");
            int level = Env.intOrDefault(pair[1], -1);
            if (!extension.isEmpty() && level >= 1 && level <= 4) {
                map.put(extension, level);
            }
        }
        return map;
    }

    /** HL7 v3 acknowledgement type codes: CA accepts; CE, AE, CR and AR do not. */
    private static boolean indicatesRejection(String lowerBody) {
        return lowerBody.contains("typecode=\"ce\"")
                || lowerBody.contains("typecode=\"ae\"")
                || lowerBody.contains("typecode=\"cr\"")
                || lowerBody.contains("typecode=\"ar\"");
    }

    private static String firstLine(String body) {
        String trimmed = body.strip();
        int newline = trimmed.indexOf('\n');
        String line = newline < 0 ? trimmed : trimmed.substring(0, newline);
        return line.length() > 500 ? line.substring(0, 500) : line;
    }
}
```

- [ ] **Step 5: Run the test to verify it passes**

```bash
mvn -q -o test -Dtest=AckClassifierTest
```

Expected: `Tests run: 10, Failures: 0, Errors: 0`.

- [ ] **Step 6: Commit**

```bash
git add src/main/java/com/example/as2/ack/AckClassifier.java \
        src/main/java/com/example/as2/ack/AckDecision.java \
        src/main/java/com/example/as2/as2/InboundMessage.java \
        src/test/java/com/example/as2/ack/AckClassifierTest.java
git commit -m "feat: classify inbound AS2 messages into ACK levels

MDN maps to ACK1 and .ack to ACK3 on the AERS path. The extension map is
configurable so a wrong assumption about another submission type is a
config change rather than a release."
```

---

## Task 12: `ack/AckCorrelator`

**Files:**
- Create: `src/main/java/com/example/as2/ack/AckCorrelator.java`
- Test: `src/test/java/com/example/as2/ack/AckCorrelatorTest.java` (create)

- [ ] **Step 1: Write the failing test**

Create `src/test/java/com/example/as2/ack/AckCorrelatorTest.java`:

```java
package com.example.as2.ack;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

import com.example.as2.as2.InboundMessage;
import com.example.as2.as2.Mdn;
import com.example.as2.state.Store;
import com.example.as2.state.SubmissionRecord;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.util.Map;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

class AckCorrelatorTest {
    private static Store storeWithSubmission(Path dir) {
        Store store = new Store(dir.resolve("state.json"));
        SubmissionRecord record = new SubmissionRecord();
        record.submissionId = "s1";
        record.remoteSubmissionId = "<outbound@as2-submitter>";
        record.as2MessageId = "<outbound@as2-submitter>";
        record.caseId = "case-abc";
        record.authority = "fda";
        record.e2bBatchNumber = "BATCH-2026-0001";
        record.e2bMessageIdentifier = "MSG-2026-0001";
        store.putSubmission(record);
        return store;
    }

    private static InboundMessage mdnFor(String originalMessageId) {
        return new InboundMessage(
                Map.of(), "FDA_AERS", "E2BR3-SUBMITTER", "<mdn@fda>",
                "multipart/report", null, new byte[0],
                new Mdn.Parsed(true, true, "processed", originalMessageId, "abc=", "sha-256"));
    }

    private static InboundMessage fileFor(String filename, String body) {
        return new InboundMessage(
                Map.of(), "FDA_AERS", "E2BR3-SUBMITTER", "<file@fda>",
                "application/octet-stream", filename,
                body.getBytes(StandardCharsets.UTF_8), null);
    }

    private static String ackWith(String extension) {
        return "<MCCI_IN200101UV01 xmlns=\"urn:hl7-org:v3\"><id extension=\""
                + extension + "\" root=\"x\"/></MCCI_IN200101UV01>";
    }

    @Test
    void mdnCorrelatesByOriginalMessageId(@TempDir Path dir) {
        Store store = storeWithSubmission(dir);
        assertEquals("s1", AckCorrelator.correlate(store, mdnFor("<outbound@as2-submitter>")).submissionId);
    }

    @Test
    void mdnWithAnUnknownOriginalMessageIdDoesNotCorrelate(@TempDir Path dir) {
        assertNull(AckCorrelator.correlate(storeWithSubmission(dir), mdnFor("<someone-else@elsewhere>")));
    }

    @Test
    void fileCorrelatesByMessageIdentifier(@TempDir Path dir) {
        Store store = storeWithSubmission(dir);
        assertEquals("s1",
                AckCorrelator.correlate(store, fileFor("x.ack", ackWith("MSG-2026-0001"))).submissionId);
    }

    @Test
    void fileCorrelatesByBatchNumber(@TempDir Path dir) {
        Store store = storeWithSubmission(dir);
        assertEquals("s1",
                AckCorrelator.correlate(store, fileFor("x.ack", ackWith("BATCH-2026-0001"))).submissionId);
    }

    @Test
    void fileFallsBackToCaseIdInTheFilename(@TempDir Path dir) {
        Store store = storeWithSubmission(dir);
        assertEquals("s1",
                AckCorrelator.correlate(store, fileFor("ACK_case-abc_20260727.ack", "opaque")).submissionId);
    }

    @Test
    void identifierMatchWinsOverFilename(@TempDir Path dir) {
        Store store = storeWithSubmission(dir);
        SubmissionRecord other = new SubmissionRecord();
        other.submissionId = "s2";
        other.caseId = "case-abc";
        store.putSubmission(other);

        assertEquals("s1",
                AckCorrelator.correlate(store, fileFor("ACK_case-abc.ack", ackWith("MSG-2026-0001")))
                        .submissionId);
    }

    @Test
    void unmatchableFileReturnsNull(@TempDir Path dir) {
        assertNull(AckCorrelator.correlate(
                storeWithSubmission(dir), fileFor("mystery.ack", "nothing familiar here")));
    }

    @Test
    void nonXmlFileStillTriesTheFilename(@TempDir Path dir) {
        Store store = storeWithSubmission(dir);
        assertEquals("s1", AckCorrelator.correlate(store, fileFor("case-abc.ack", " binary")).submissionId);
    }
}
```

- [ ] **Step 2: Run it to verify it fails**

```bash
mvn -q -o test -Dtest=AckCorrelatorTest
```

Expected: COMPILATION ERROR — `cannot find symbol: class AckCorrelator`.

- [ ] **Step 3: Write the implementation**

Create `src/main/java/com/example/as2/ack/AckCorrelator.java`:

```java
package com.example.as2.ack;

import com.example.as2.as2.E2bIdentifiers;
import com.example.as2.as2.InboundMessage;
import com.example.as2.state.Store;
import com.example.as2.state.SubmissionRecord;
import java.util.Set;

/**
 * Matches an inbound acknowledgement to the submission it belongs to.
 *
 * <p>Strategies are tried in descending order of confidence. Returning null is a
 * normal outcome: the caller records an orphan and still answers with an MDN,
 * because an HTTP error would make FDA treat the acknowledgement as undelivered.
 */
public final class AckCorrelator {
    private AckCorrelator() {}

    public static SubmissionRecord correlate(Store store, InboundMessage message) {
        if (message.isMdn()) {
            return store.findByAs2MessageId(message.mdn().originalMessageId());
        }

        Set<String> identifiers = E2bIdentifiers.allIdExtensions(message.payload());
        SubmissionRecord byIdentifier = store.findByE2bIdentifier(identifiers);
        if (byIdentifier != null) {
            return byIdentifier;
        }

        return store.findByCaseIdInFilename(message.filename());
    }
}
```

- [ ] **Step 4: Run the test to verify it passes**

```bash
mvn -q -o test -Dtest=AckCorrelatorTest
```

Expected: `Tests run: 8, Failures: 0, Errors: 0`.

- [ ] **Step 5: Commit**

```bash
git add src/main/java/com/example/as2/ack/AckCorrelator.java \
        src/test/java/com/example/as2/ack/AckCorrelatorTest.java
git commit -m "feat: correlate inbound acknowledgements to submissions

Original-Message-ID for MDNs, then E2B identifiers, then the case id in
the filename. No match is a normal outcome that produces an orphan, not
an error response."
```

---

## Task 13: `config/AuthorityProfile`

One immutable object holds everything the sender needs for an authority, so no
transport code reads the environment directly. It takes an environment lookup
function rather than calling `System.getenv`, which makes every combination
testable without mutating process state.

**Files:**
- Create: `src/main/java/com/example/as2/config/AuthorityProfile.java`
- Test: `src/test/java/com/example/as2/config/AuthorityProfileTest.java` (create)

**Environment variables consumed:**

| Variable | Default | Meaning |
| --- | --- | --- |
| `AS2_FROM_ID` | required | Our industry routing ID, sent as `AS2-From`. |
| `AS2_FDA_ROUTING_MODE` | `routing_id` | One of the three FDA routing options. |
| `AS2_FDA_ENV` | `test` | `prod` or `test`; picks the endpoint and gateway ID. |
| `AS2_FDA_ENDPOINT_URL` | derived from `AS2_FDA_ENV` | Override, required for local and EC2 testing. |
| `AS2_FDA_ENCRYPT_CERT_PEM_PATH` | required when crypto is on | FDA's public certificate. |
| `AS2_MFDS_TO_ID` | required for MFDS | MFDS `AS2-To`. |
| `AS2_MFDS_ENDPOINT_URL` | required for MFDS | MFDS endpoint. |
| `AS2_MFDS_ENCRYPT_CERT_PEM_PATH` | required when crypto is on | MFDS public certificate. |
| `AS2_ENABLE_CRYPTO` | off | Sign and encrypt outbound messages. |
| `AS2_SIGNING_PKCS12_PATH` / `_PASSWORD` / `AS2_SIGNING_KEY_ALIAS` | required when crypto is on | Our key material. |
| `AS2_ENCRYPTION_ALGORITHM` | `aes-256-cbc` | FDA prefers AES; AES-256-GCM is unsupported. |
| `AS2_MIC_ALGORITHM` | `sha-256` | Signing digest and requested MIC algorithm. |
| `AS2_ASYNC_MDN_URL` | unset | When set, requests an asynchronous MDN to this URL. |
| `AS2_AUTHORITY_TIMEOUT_SECS` | `20` | Outbound request timeout. |

- [ ] **Step 1: Write the failing test**

Create `src/test/java/com/example/as2/config/AuthorityProfileTest.java`:

```java
package com.example.as2.config;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.HashMap;
import java.util.Map;
import org.junit.jupiter.api.Test;

class AuthorityProfileTest {
    private static Map<String, String> baseEnv() {
        Map<String, String> env = new HashMap<>();
        env.put("AS2_FROM_ID", "E2BR3-SUBMITTER");
        return env;
    }

    private static AuthorityProfile build(String authority, Map<String, String> env) {
        return AuthorityProfile.build(authority, env::get);
    }

    @Test
    void fdaDefaultsToRoutingIdModeAgainstTheTestEndpoint() {
        AuthorityProfile profile = build("fda", baseEnv());
        assertEquals("E2BR3-SUBMITTER", profile.as2From());
        assertEquals("FDA_AERS", profile.as2To());
        assertEquals("https://upload-api-esgng.fda.gov:4080/as2/receive/test", profile.endpointUrl());
        assertTrue(profile.extraHeaders().isEmpty());
        assertEquals("FDA_AERS", profile.expectedAckAs2From());
    }

    @Test
    void fdaProdEnvironmentSelectsTheProductionEndpoint() {
        Map<String, String> env = baseEnv();
        env.put("AS2_FDA_ENV", "prod");
        assertEquals("https://upload-api-esgng.fda.gov:4080/as2/receive", build("fda", env).endpointUrl());
    }

    @Test
    void fdaEndpointOverrideWinsOverTheDerivedUrl() {
        Map<String, String> env = baseEnv();
        env.put("AS2_FDA_ENDPOINT_URL", "http://127.0.0.1:5080");
        assertEquals("http://127.0.0.1:5080", build("fda", env).endpointUrl());
    }

    @Test
    void fdaMetadataHeadersModeCarriesTheCycloneHeaders() {
        Map<String, String> env = baseEnv();
        env.put("AS2_FDA_ROUTING_MODE", "metadata_headers");
        env.put("AS2_FDA_ENV", "prod");
        AuthorityProfile profile = build("fda", env);
        assertEquals("ZZFDA", profile.as2To());
        assertEquals("CDER", profile.extraHeaders().get("X-Cyclone-Metadata-FdaCenter"));
        assertEquals("AERS", profile.extraHeaders().get("X-Cyclone-Metadata-FdaSubmissionType"));
    }

    @Test
    void mfdsUsesItsOwnRoutingIdAndEndpoint() {
        Map<String, String> env = baseEnv();
        env.put("AS2_MFDS_TO_ID", "MFDS_GW");
        env.put("AS2_MFDS_ENDPOINT_URL", "https://mfds.example/as2");
        AuthorityProfile profile = build("mfds", env);
        assertEquals("MFDS_GW", profile.as2To());
        assertEquals("https://mfds.example/as2", profile.endpointUrl());
        assertEquals("MFDS_GW", profile.expectedAckAs2From());
        assertTrue(profile.extraHeaders().isEmpty());
    }

    @Test
    void mfdsWithoutARoutingIdIsRejected() {
        Map<String, String> env = baseEnv();
        env.put("AS2_MFDS_ENDPOINT_URL", "https://mfds.example/as2");
        IllegalStateException ex = assertThrows(IllegalStateException.class, () -> build("mfds", env));
        assertTrue(ex.getMessage().contains("AS2_MFDS_TO_ID"));
    }

    @Test
    void anUnknownAuthorityIsRejected() {
        assertThrows(IllegalStateException.class, () -> build("ema", baseEnv()));
    }

    @Test
    void aMissingFromIdIsRejected() {
        assertThrows(IllegalStateException.class, () -> build("fda", new HashMap<>()));
    }

    @Test
    void cryptoIsOffByDefault() {
        AuthorityProfile profile = build("fda", baseEnv());
        assertFalse(profile.cryptoEnabled());
        assertNull(profile.signingKeystorePath());
    }

    @Test
    void cryptoRequiresKeyMaterialAndAPartnerCertificate() {
        Map<String, String> env = baseEnv();
        env.put("AS2_ENABLE_CRYPTO", "1");
        IllegalStateException ex = assertThrows(IllegalStateException.class, () -> build("fda", env));
        assertTrue(ex.getMessage().contains("AS2_SIGNING_PKCS12_PATH"));

        env.put("AS2_SIGNING_PKCS12_PATH", "certs/submitter.p12");
        env.put("AS2_SIGNING_PKCS12_PASSWORD", "changeit");
        IllegalStateException missingCert =
                assertThrows(IllegalStateException.class, () -> build("fda", env));
        assertTrue(missingCert.getMessage().contains("AS2_FDA_ENCRYPT_CERT_PEM_PATH"));
    }

    @Test
    void cryptoEnabledProfileCarriesTheAlgorithmDefaults() {
        Map<String, String> env = baseEnv();
        env.put("AS2_ENABLE_CRYPTO", "1");
        env.put("AS2_SIGNING_PKCS12_PATH", "certs/submitter.p12");
        env.put("AS2_SIGNING_PKCS12_PASSWORD", "changeit");
        env.put("AS2_FDA_ENCRYPT_CERT_PEM_PATH", "certs/partner.crt");
        AuthorityProfile profile = build("fda", env);
        assertTrue(profile.cryptoEnabled());
        assertEquals("aes-256-cbc", profile.encryptionAlgorithm());
        assertEquals("sha-256", profile.micAlgorithm());
        assertEquals("certs/partner.crt", profile.encryptionCertPath());
    }

    @Test
    void asyncMdnUrlIsCarriedWhenConfigured() {
        Map<String, String> env = baseEnv();
        env.put("AS2_ASYNC_MDN_URL", "http://gw.example:4080/as2/receive");
        assertEquals("http://gw.example:4080/as2/receive", build("fda", env).asyncMdnUrl());
        assertNull(build("fda", baseEnv()).asyncMdnUrl());
    }

    @Test
    void timeoutDefaultsToTwentySeconds() {
        assertEquals(20, build("fda", baseEnv()).timeoutSecs());
        Map<String, String> env = baseEnv();
        env.put("AS2_AUTHORITY_TIMEOUT_SECS", "45");
        assertEquals(45, build("fda", env).timeoutSecs());
    }
}
```

- [ ] **Step 2: Run it to verify it fails**

```bash
mvn -q -o test -Dtest=AuthorityProfileTest
```

Expected: COMPILATION ERROR — `cannot find symbol: class AuthorityProfile`.

- [ ] **Step 3: Write the implementation**

Create `src/main/java/com/example/as2/config/AuthorityProfile.java`:

```java
package com.example.as2.config;

import java.util.Locale;
import java.util.Map;
import java.util.function.Function;

/** Everything the sender needs to reach one authority. */
public record AuthorityProfile(
        String authority,
        String endpointUrl,
        String as2From,
        String as2To,
        Map<String, String> extraHeaders,
        String expectedAckAs2From,
        boolean cryptoEnabled,
        String signingKeystorePath,
        String signingKeystorePassword,
        String signingKeyAlias,
        String encryptionCertPath,
        String encryptionAlgorithm,
        String micAlgorithm,
        String asyncMdnUrl,
        int timeoutSecs) {

    public static AuthorityProfile fromEnvironment(String authority) {
        return build(authority, System::getenv);
    }

    public static AuthorityProfile build(String authority, Function<String, String> env) {
        String normalized = Env.normalize(authority);
        if (normalized == null) {
            throw new IllegalStateException("authority is required");
        }
        String key = normalized.toLowerCase(Locale.ROOT);

        String as2From = require(env, "AS2_FROM_ID");
        boolean crypto = Env.truthy(env.apply("AS2_ENABLE_CRYPTO"));

        String signingPath = null;
        String signingPassword = null;
        if (crypto) {
            signingPath = require(env, "AS2_SIGNING_PKCS12_PATH");
            signingPassword = require(env, "AS2_SIGNING_PKCS12_PASSWORD");
        }

        String endpointUrl;
        String as2To;
        Map<String, String> extraHeaders;
        String expectedAckAs2From;
        String certVar;

        switch (key) {
            case "fda" -> {
                FdaRouting routing = FdaRouting.resolve(
                        env.apply("AS2_FDA_ROUTING_MODE"), env.apply("AS2_FDA_ENV"));
                endpointUrl = Env.firstNonBlank(
                        env.apply("AS2_FDA_ENDPOINT_URL"), routing.defaultEndpointUrl());
                as2To = routing.as2To();
                extraHeaders = routing.extraHeaders();
                expectedAckAs2From = routing.expectedAckAs2From();
                certVar = "AS2_FDA_ENCRYPT_CERT_PEM_PATH";
            }
            case "mfds" -> {
                as2To = require(env, "AS2_MFDS_TO_ID");
                endpointUrl = require(env, "AS2_MFDS_ENDPOINT_URL");
                extraHeaders = Map.of();
                expectedAckAs2From = as2To;
                certVar = "AS2_MFDS_ENCRYPT_CERT_PEM_PATH";
            }
            default -> throw new IllegalStateException(
                    "authority must be fda or mfds, got: " + authority);
        }

        String certPath = crypto ? require(env, certVar) : Env.normalize(env.apply(certVar));

        return new AuthorityProfile(
                key,
                endpointUrl,
                as2From,
                as2To,
                extraHeaders,
                expectedAckAs2From,
                crypto,
                signingPath,
                signingPassword,
                Env.normalize(env.apply("AS2_SIGNING_KEY_ALIAS")),
                certPath,
                Env.firstNonBlank(env.apply("AS2_ENCRYPTION_ALGORITHM"), "aes-256-cbc"),
                Env.firstNonBlank(env.apply("AS2_MIC_ALGORITHM"), "sha-256"),
                Env.normalize(env.apply("AS2_ASYNC_MDN_URL")),
                Env.intOrDefault(env.apply("AS2_AUTHORITY_TIMEOUT_SECS"), 20));
    }

    /** True when we requested the MDN be delivered on a separate connection. */
    public boolean asyncMdnRequested() {
        return asyncMdnUrl != null && !asyncMdnUrl.isBlank();
    }

    private static String require(Function<String, String> env, String key) {
        String value = Env.normalize(env.apply(key));
        if (value == null) {
            throw new IllegalStateException("Missing required env var: " + key);
        }
        return value;
    }
}
```

- [ ] **Step 4: Run the test to verify it passes**

```bash
mvn -q -o test -Dtest=AuthorityProfileTest
```

Expected: `Tests run: 13, Failures: 0, Errors: 0`.

- [ ] **Step 5: Commit**

```bash
git add src/main/java/com/example/as2/config/AuthorityProfile.java \
        src/test/java/com/example/as2/config/AuthorityProfileTest.java
git commit -m "feat: per-authority transport profile

Takes an environment lookup function rather than reading System.getenv,
so every routing and crypto combination is testable without mutating
process state."
```

---

## Task 14: `as2/As2Sender`

Header construction is separated from I/O so the exact wire headers can be
asserted without a network.

**Files:**
- Create: `src/main/java/com/example/as2/as2/As2Headers.java`
- Create: `src/main/java/com/example/as2/as2/As2Sender.java`
- Test: `src/test/java/com/example/as2/as2/As2SenderHeadersTest.java` (create)

- [ ] **Step 1: Write the failing test**

Create `src/test/java/com/example/as2/as2/As2SenderHeadersTest.java`:

```java
package com.example.as2.as2;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.example.as2.config.AuthorityProfile;
import java.util.HashMap;
import java.util.Map;
import org.junit.jupiter.api.Test;

class As2SenderHeadersTest {
    private static AuthorityProfile profile(Map<String, String> overrides) {
        Map<String, String> env = new HashMap<>();
        env.put("AS2_FROM_ID", "E2BR3-SUBMITTER");
        env.putAll(overrides);
        return AuthorityProfile.build("fda", env::get);
    }

    private static Map<String, String> headers(AuthorityProfile p) {
        return As2Sender.buildHeaders(
                p, "<msg-1@as2-submitter>", "E2B(R3) FDA case case-abc", "case-abc.xml", "application/xml");
    }

    @Test
    void carriesTheCoreAs2Headers() {
        Map<String, String> h = headers(profile(Map.of()));
        assertEquals("1.2", h.get("AS2-Version"));
        assertEquals("E2BR3-SUBMITTER", h.get("AS2-From"));
        assertEquals("FDA_AERS", h.get("AS2-To"));
        assertEquals("<msg-1@as2-submitter>", h.get("Message-ID"));
        assertEquals("E2B(R3) FDA case case-abc", h.get("Subject"));
        assertEquals("application/xml", h.get("Content-Type"));
    }

    @Test
    void requestsASignedReceiptAsRequiredNotOptional() {
        String options = headers(profile(Map.of())).get("Disposition-Notification-Options");
        assertTrue(options.contains("signed-receipt-protocol=required, pkcs7-signature"));
        assertTrue(options.contains("signed-receipt-micalg=required, sha-256"));
        assertFalse(options.contains("optional"));
    }

    @Test
    void addressesTheReceiptToOurOwnRoutingId() {
        assertEquals("E2BR3-SUBMITTER", headers(profile(Map.of())).get("Disposition-Notification-To"));
    }

    @Test
    void preservesTheFilename() {
        assertEquals(
                "attachment; filename=\"case-abc.xml\"",
                headers(profile(Map.of())).get("Content-Disposition"));
    }

    @Test
    void omitsReceiptDeliveryOptionWhenMdnIsSynchronous() {
        assertFalse(headers(profile(Map.of())).containsKey("Receipt-Delivery-Option"));
    }

    @Test
    void addsReceiptDeliveryOptionWhenMdnIsAsynchronous() {
        Map<String, String> h = headers(profile(
                Map.of("AS2_ASYNC_MDN_URL", "http://gw.example:4080/as2/receive")));
        assertEquals("http://gw.example:4080/as2/receive", h.get("Receipt-Delivery-Option"));
    }

    @Test
    void routingIdModeEmitsNoCycloneHeaders() {
        Map<String, String> h = headers(profile(Map.of()));
        assertFalse(h.containsKey("X-Cyclone-True-Receiver"));
        assertFalse(h.containsKey("X-Cyclone-Metadata-FdaCenter"));
        assertFalse(h.containsKey("X-Cyclone-Metadata-FdaSubmissionType"));
    }

    @Test
    void trueReceiverModeEmitsOnlyTheTrueReceiverHeader() {
        Map<String, String> h = headers(profile(Map.of("AS2_FDA_ROUTING_MODE", "true_receiver")));
        assertEquals("ZZFDATST", h.get("AS2-To"));
        assertEquals("FDA_AERS", h.get("X-Cyclone-True-Receiver"));
        assertFalse(h.containsKey("X-Cyclone-Metadata-FdaCenter"));
    }

    @Test
    void metadataHeadersModeEmitsCenterAndSubmissionType() {
        Map<String, String> h = headers(profile(Map.of("AS2_FDA_ROUTING_MODE", "metadata_headers")));
        assertEquals("CDER", h.get("X-Cyclone-Metadata-FdaCenter"));
        assertEquals("AERS", h.get("X-Cyclone-Metadata-FdaSubmissionType"));
        assertFalse(h.containsKey("X-Cyclone-True-Receiver"));
    }

    @Test
    void filenamesAreSanitized() {
        assertEquals("a_b_c.xml", As2Sender.outboundFilename("a/b c"));
        assertEquals("case.xml", As2Sender.outboundFilename(null));
    }

    @Test
    void messageIdsFollowRfc4130Shape() {
        String id = As2Sender.newMessageId();
        assertTrue(id.startsWith("<"));
        assertTrue(id.endsWith("@as2-submitter>"));
    }
}
```

- [ ] **Step 2: Run it to verify it fails**

```bash
mvn -q -o test -Dtest=As2SenderHeadersTest
```

Expected: COMPILATION ERROR — `cannot find symbol: class As2Sender`.

- [ ] **Step 3: Write the header constants**

Create `src/main/java/com/example/as2/as2/As2Headers.java`:

```java
package com.example.as2.as2;

/** AS2 header names, spelled as they go on the wire. */
public final class As2Headers {
    public static final String VERSION = "AS2-Version";
    public static final String FROM = "AS2-From";
    public static final String TO = "AS2-To";
    public static final String MESSAGE_ID = "Message-ID";
    public static final String SUBJECT = "Subject";
    public static final String CONTENT_TYPE = "Content-Type";
    public static final String CONTENT_DISPOSITION = "Content-Disposition";
    public static final String DISPOSITION_NOTIFICATION_TO = "Disposition-Notification-To";
    public static final String DISPOSITION_NOTIFICATION_OPTIONS = "Disposition-Notification-Options";
    public static final String RECEIPT_DELIVERY_OPTION = "Receipt-Delivery-Option";

    public static final String VERSION_VALUE = "1.2";
    public static final String USER_AGENT = "as2-submitter/1.0";

    private As2Headers() {}

    /**
     * Asks for a signed receipt as {@code required}. Under {@code optional} a
     * partner that never signs its receipts still passes, which defeats the
     * point of requesting one.
     */
    public static String signedReceiptOptions(String micAlgorithm) {
        return "signed-receipt-protocol=required, pkcs7-signature; "
                + "signed-receipt-micalg=required, " + micAlgorithm;
    }
}
```

- [ ] **Step 4: Write the sender**

Create `src/main/java/com/example/as2/as2/As2Sender.java`:

```java
package com.example.as2.as2;

import com.example.as2.config.AuthorityProfile;
import jakarta.mail.internet.MimeBodyPart;
import java.io.InputStream;
import java.io.StringReader;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.KeyStore;
import java.security.PrivateKey;
import java.security.cert.X509Certificate;
import java.time.Duration;
import java.util.LinkedHashMap;
import java.util.Locale;
import java.util.Map;
import java.util.UUID;
import org.bouncycastle.cert.X509CertificateHolder;
import org.bouncycastle.cert.jcajce.JcaX509CertificateConverter;
import org.bouncycastle.openssl.PEMParser;

/** Builds and sends an outbound AS2 submission. */
public final class As2Sender {

    /** What a dispatch produced, whether or not an MDN came back on the same connection. */
    public record Result(
            String as2MessageId,
            String outboundFilename,
            String expectedMic,
            boolean mdnReceived,
            String mdnDisposition,
            String receivedMic,
            Boolean micMatch,
            boolean mdnSuccess) {}

    private final HttpClient httpClient;

    public As2Sender(HttpClient httpClient) {
        this.httpClient = httpClient;
    }

    public static String newMessageId() {
        return "<" + UUID.randomUUID() + "@as2-submitter>";
    }

    public static String outboundFilename(String caseId) {
        if (caseId == null || caseId.isBlank()) {
            return "case.xml";
        }
        return caseId.trim().replaceAll("[^A-Za-z0-9._-]", "_") + ".xml";
    }

    /** The exact header set for one outbound message. Pure, so tests can assert it. */
    public static Map<String, String> buildHeaders(
            AuthorityProfile profile,
            String messageId,
            String subject,
            String filename,
            String contentType) {
        Map<String, String> headers = new LinkedHashMap<>();
        headers.put(As2Headers.CONTENT_TYPE, contentType);
        headers.put(As2Headers.VERSION, As2Headers.VERSION_VALUE);
        headers.put(As2Headers.FROM, profile.as2From());
        headers.put(As2Headers.TO, profile.as2To());
        headers.put(As2Headers.MESSAGE_ID, messageId);
        headers.put(As2Headers.SUBJECT, subject);
        headers.put(As2Headers.CONTENT_DISPOSITION, "attachment; filename=\"" + filename + "\"");
        headers.put(As2Headers.DISPOSITION_NOTIFICATION_TO, profile.as2From());
        headers.put(
                As2Headers.DISPOSITION_NOTIFICATION_OPTIONS,
                As2Headers.signedReceiptOptions(profile.micAlgorithm()));
        if (profile.asyncMdnRequested()) {
            headers.put(As2Headers.RECEIPT_DELIVERY_OPTION, profile.asyncMdnUrl());
        }
        headers.putAll(profile.extraHeaders());
        return headers;
    }

    public Result send(AuthorityProfile profile, String caseId, String xmlPayload) throws Exception {
        String messageId = newMessageId();
        String filename = outboundFilename(caseId);
        String subject = "E2B(R3) " + profile.authority().toUpperCase(Locale.ROOT) + " case " + caseId;

        MimeBodyPart payload =
                Cms.buildPayload(xmlPayload.getBytes(StandardCharsets.UTF_8), "application/xml");
        String expectedMic = Mic.compute(payload, profile.micAlgorithm());

        MimeBodyPart outbound = payload;
        if (profile.cryptoEnabled()) {
            SigningMaterial signing = loadSigningMaterial(profile);
            outbound = Cms.sign(payload, signing.privateKey(), signing.certificate(), profile.micAlgorithm());
            expectedMic = Mic.compute(payload, profile.micAlgorithm());
            outbound = Cms.encrypt(
                    outbound, loadPartnerCertificate(profile), profile.encryptionAlgorithm());
        }

        byte[] body = Cms.contentBytes(outbound);
        Map<String, String> headers =
                buildHeaders(profile, messageId, subject, filename, outbound.getContentType());

        HttpRequest.Builder request = HttpRequest.newBuilder()
                .uri(URI.create(profile.endpointUrl()))
                .timeout(Duration.ofSeconds(profile.timeoutSecs()))
                .POST(HttpRequest.BodyPublishers.ofByteArray(body));
        headers.forEach(request::header);

        HttpResponse<byte[]> response =
                httpClient.send(request.build(), HttpResponse.BodyHandlers.ofByteArray());
        if (response.statusCode() < 200 || response.statusCode() >= 300) {
            throw new IllegalStateException(
                    "authority endpoint rejected request: status=" + response.statusCode());
        }

        String responseContentType = response.headers().firstValue("content-type").orElse("");
        Mdn.Parsed mdn = Mdn.parse(responseContentType, response.body());
        Boolean micMatch = null;
        if (mdn.isMdn() && mdn.receivedMic() != null) {
            micMatch = Mdn.micMatches(expectedMic, mdn.receivedMic());
        }

        return new Result(
                messageId,
                filename,
                expectedMic,
                mdn.isMdn(),
                mdn.disposition(),
                mdn.receivedMic(),
                micMatch,
                mdn.success());
    }

    private record SigningMaterial(PrivateKey privateKey, X509Certificate certificate) {}

    private static SigningMaterial loadSigningMaterial(AuthorityProfile profile) throws Exception {
        KeyStore keyStore = KeyStore.getInstance("PKCS12");
        char[] password = profile.signingKeystorePassword().toCharArray();
        try (InputStream in = Files.newInputStream(Path.of(profile.signingKeystorePath()))) {
            keyStore.load(in, password);
        }

        String alias = profile.signingKeyAlias();
        if (alias == null) {
            var aliases = keyStore.aliases();
            while (aliases.hasMoreElements()) {
                String candidate = aliases.nextElement();
                if (keyStore.isKeyEntry(candidate)) {
                    alias = candidate;
                    break;
                }
            }
        }
        if (alias == null) {
            throw new IllegalStateException("no private key alias found in " + profile.signingKeystorePath());
        }

        PrivateKey key = (PrivateKey) keyStore.getKey(alias, password);
        java.security.cert.Certificate certificate = keyStore.getCertificate(alias);
        if (key == null || !(certificate instanceof X509Certificate x509)) {
            throw new IllegalStateException("invalid signing key material for alias " + alias);
        }
        return new SigningMaterial(key, x509);
    }

    static X509Certificate loadPartnerCertificate(AuthorityProfile profile) throws Exception {
        String pem = Files.readString(Path.of(profile.encryptionCertPath()), StandardCharsets.UTF_8);
        try (PEMParser parser = new PEMParser(new StringReader(pem))) {
            Object object = parser.readObject();
            if (!(object instanceof X509CertificateHolder holder)) {
                throw new IllegalStateException("not a PEM certificate: " + profile.encryptionCertPath());
            }
            return new JcaX509CertificateConverter().setProvider("BC").getCertificate(holder);
        }
    }
}
```

- [ ] **Step 5: Run the test to verify it passes**

```bash
mvn -q -o test -Dtest=As2SenderHeadersTest
```

Expected: `Tests run: 11, Failures: 0, Errors: 0`.

- [ ] **Step 6: Run the whole suite**

```bash
mvn -q -o test
```

Expected: BUILD SUCCESS.

- [ ] **Step 7: Commit**

```bash
git add src/main/java/com/example/as2/as2/As2Headers.java \
        src/main/java/com/example/as2/as2/As2Sender.java \
        src/test/java/com/example/as2/as2/As2SenderHeadersTest.java
git commit -m "feat: AS2 sender with FDA routing and required signed receipts

Header construction is a pure function so the exact wire headers are
asserted without a network. Signed receipts move from optional to
required, and Receipt-Delivery-Option enables asynchronous MDNs."
```

---

## Task 15: `ack/AckForwarder`

Moves the backend callback and its retry queue out of `Main`. The body shape is
the contract with `web-server`, so it gets an explicit test: the Rust side
deserializes `GatewayAckCallbackInput` with **snake_case** field names and
authenticates with `x-callback-token`.

**Files:**
- Create: `src/main/java/com/example/as2/ack/AckForwarder.java`
- Modify: `src/main/java/com/example/as2/Main.java` (delete `forwardAckToBackend`, `enqueueAckForward`, `processPendingAckForwards`)
- Test: `src/test/java/com/example/as2/ack/AckForwarderTest.java` (create)

- [ ] **Step 1: Write the failing test**

Create `src/test/java/com/example/as2/ack/AckForwarderTest.java`:

```java
package com.example.as2.ack;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.example.as2.state.Store;
import com.example.as2.state.SubmissionRecord;
import com.sun.net.httpserver.HttpServer;
import java.net.InetSocketAddress;
import java.net.http.HttpClient;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

class AckForwarderTest {
    private static final ObjectMapper MAPPER = new ObjectMapper();

    private HttpServer backend;
    private final List<String> bodies = new ArrayList<>();
    private final Map<String, String> lastHeaders = new ConcurrentHashMap<>();
    private volatile int responseStatus = 200;
    private String backendUrl;

    @BeforeEach
    void startBackend() throws Exception {
        backend = HttpServer.create(new InetSocketAddress("127.0.0.1", 0), 0);
        backend.createContext("/internal/submissions/callbacks/ack", exchange -> {
            byte[] body = exchange.getRequestBody().readAllBytes();
            synchronized (bodies) {
                bodies.add(new String(body, StandardCharsets.UTF_8));
            }
            exchange.getRequestHeaders().forEach((k, v) -> lastHeaders.put(k.toLowerCase(), v.get(0)));
            exchange.sendResponseHeaders(responseStatus, 0);
            exchange.close();
        });
        backend.start();
        backendUrl = "http://127.0.0.1:" + backend.getAddress().getPort()
                + "/internal/submissions/callbacks/ack";
    }

    @AfterEach
    void stopBackend() {
        backend.stop(0);
    }

    private SubmissionRecord record() {
        SubmissionRecord r = new SubmissionRecord();
        r.submissionId = "s1";
        r.remoteSubmissionId = "<outbound@as2-submitter>";
        r.callbackUrl = backendUrl;
        return r;
    }

    @Test
    void forwardsTheExactBackendContract(@TempDir Path dir) throws Exception {
        Store store = new Store(dir.resolve("state.json"));
        AckForwarder forwarder = new AckForwarder(HttpClient.newHttpClient(), store, "secret-token");

        assertTrue(forwarder.forward(record(), new AckDecision(3, true, "ACK3_ACCEPTED", "all good")));

        JsonNode body = MAPPER.readTree(bodies.get(0));
        assertEquals("<outbound@as2-submitter>", body.get("remote_submission_id").asText());
        assertEquals(3, body.get("ack_level").asInt());
        assertTrue(body.get("success").asBoolean());
        assertEquals("ACK3_ACCEPTED", body.get("ack_code").asText());
        assertEquals("all good", body.get("ack_message").asText());
        assertEquals(5, body.size());
    }

    @Test
    void sendsTheCallbackToken(@TempDir Path dir) throws Exception {
        Store store = new Store(dir.resolve("state.json"));
        new AckForwarder(HttpClient.newHttpClient(), store, "secret-token")
                .forward(record(), new AckDecision(1, true, "ACK1_MDN_PROCESSED", "processed"));
        assertEquals("secret-token", lastHeaders.get("x-callback-token"));
        assertEquals("application/json", lastHeaders.get("content-type"));
    }

    @Test
    void aRejectedForwardIsReportedAsFailure(@TempDir Path dir) {
        responseStatus = 500;
        Store store = new Store(dir.resolve("state.json"));
        AckForwarder forwarder = new AckForwarder(HttpClient.newHttpClient(), store, "secret-token");
        assertFalse(forwarder.forward(record(), new AckDecision(3, true, "ACK3_ACCEPTED", "x")));
    }

    @Test
    void enqueueThenDrainDeliversTheCallback(@TempDir Path dir) throws Exception {
        Store store = new Store(dir.resolve("state.json"));
        SubmissionRecord r = record();
        store.putSubmission(r);
        AckForwarder forwarder = new AckForwarder(HttpClient.newHttpClient(), store, "secret-token");

        forwarder.enqueue(r, new AckDecision(3, true, "ACK3_ACCEPTED", "queued"), "initial failure");
        assertEquals(1, store.pendingAckCount());

        forwarder.drain();
        assertEquals(0, store.pendingAckCount());
        assertEquals(1, bodies.size());
    }

    @Test
    void drainKeepsTaskQueuedWhileTheBackendRejects(@TempDir Path dir) {
        responseStatus = 503;
        Store store = new Store(dir.resolve("state.json"));
        SubmissionRecord r = record();
        store.putSubmission(r);
        AckForwarder forwarder = new AckForwarder(HttpClient.newHttpClient(), store, "secret-token");

        forwarder.enqueue(r, new AckDecision(3, true, "ACK3_ACCEPTED", "queued"), "initial failure");
        forwarder.drain();
        assertEquals(1, store.pendingAckCount());
    }

    @Test
    void forwardingIsSkippedWhenNoCallbackUrlIsConfigured(@TempDir Path dir) {
        Store store = new Store(dir.resolve("state.json"));
        SubmissionRecord r = record();
        r.callbackUrl = null;
        assertFalse(new AckForwarder(HttpClient.newHttpClient(), store, "secret-token")
                .forward(r, new AckDecision(1, true, "ACK1_MDN_PROCESSED", "x")));
    }
}
```

- [ ] **Step 2: Run it to verify it fails**

```bash
mvn -q -o test -Dtest=AckForwarderTest
```

Expected: COMPILATION ERROR — `cannot find symbol: class AckForwarder`.

- [ ] **Step 3: Write the implementation**

Create `src/main/java/com/example/as2/ack/AckForwarder.java`:

```java
package com.example.as2.ack;

import com.example.as2.config.Env;
import com.example.as2.state.AckForwardTask;
import com.example.as2.state.Store;
import com.example.as2.state.SubmissionRecord;
import com.fasterxml.jackson.databind.ObjectMapper;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.time.Duration;
import java.time.OffsetDateTime;
import java.util.HashMap;
import java.util.Map;
import java.util.UUID;

/**
 * Delivers acknowledgements to the backend.
 *
 * <p>The body is the contract with {@code web-server}: snake_case fields
 * matching {@code GatewayAckCallbackInput}, authenticated with
 * {@code x-callback-token}.
 */
public class AckForwarder {
    private static final ObjectMapper MAPPER = new ObjectMapper();

    private final HttpClient httpClient;
    private final Store store;
    private final String callbackToken;

    public AckForwarder(HttpClient httpClient, Store store, String callbackToken) {
        this.httpClient = httpClient;
        this.store = store;
        this.callbackToken = callbackToken;
    }

    public boolean forward(SubmissionRecord record, AckDecision decision) {
        if (record == null || record.callbackUrl == null || record.callbackUrl.isBlank()) {
            return false;
        }
        if (callbackToken == null || callbackToken.isBlank()) {
            return false;
        }

        Map<String, Object> payload = new HashMap<>();
        payload.put("remote_submission_id", record.remoteSubmissionId);
        payload.put("ack_level", decision.level());
        payload.put("success", decision.success());
        payload.put("ack_code", decision.code());
        payload.put("ack_message", decision.message());

        try {
            HttpRequest request = HttpRequest.newBuilder()
                    .uri(URI.create(record.callbackUrl))
                    .timeout(Duration.ofSeconds(10))
                    .header("content-type", "application/json")
                    .header("x-callback-token", callbackToken)
                    .POST(HttpRequest.BodyPublishers.ofString(MAPPER.writeValueAsString(payload)))
                    .build();
            HttpResponse<String> response =
                    httpClient.send(request, HttpResponse.BodyHandlers.ofString());
            return response.statusCode() >= 200 && response.statusCode() < 300;
        } catch (Exception ex) {
            return false;
        }
    }

    public void enqueue(SubmissionRecord record, AckDecision decision, String reason) {
        AckForwardTask task = new AckForwardTask();
        task.id = UUID.randomUUID().toString();
        task.submissionId = record.submissionId;
        task.remoteSubmissionId = record.remoteSubmissionId;
        task.callbackUrl = record.callbackUrl;
        task.ackLevel = decision.level();
        task.success = decision.success();
        task.ackCode = decision.code();
        task.ackMessage = decision.message();
        task.attemptCount = 0;
        task.nextAttemptAt = OffsetDateTime.now().toString();
        task.lastError = reason;
        store.putPendingAck(task);
        store.save();
    }

    /** One pass over the retry queue. Safe to call on a schedule. */
    public void drain() {
        if (store.pendingAckCount() == 0) {
            return;
        }
        int maxAttempts = Env.envInt("AS2_ACK_FORWARD_MAX_ATTEMPTS", 10);
        long baseMs = Env.envLong("AS2_ACK_FORWARD_BASE_MS", 1000);
        long maxMs = Env.envLong("AS2_ACK_FORWARD_MAX_MS", 60_000);
        String now = OffsetDateTime.now().toString();

        for (AckForwardTask task : store.pendingAcks()) {
            if (task.nextAttemptAt != null && task.nextAttemptAt.compareTo(now) > 0) {
                continue;
            }
            SubmissionRecord record = store.getSubmission(task.submissionId);
            if (record == null || record.callbackUrl == null || record.callbackUrl.isBlank()) {
                store.removePendingAck(task.id);
                continue;
            }

            AckDecision decision =
                    new AckDecision(task.ackLevel, task.success, task.ackCode, task.ackMessage);
            if (forward(record, decision)) {
                store.removePendingAck(task.id);
                continue;
            }

            task.attemptCount++;
            if (task.attemptCount >= maxAttempts) {
                store.removePendingAck(task.id);
                continue;
            }
            long backoff = Math.min(baseMs * (1L << Math.min(task.attemptCount, 16)), maxMs);
            task.nextAttemptAt = OffsetDateTime.now().plusNanos(backoff * 1_000_000).toString();
            task.lastError = "forward failed";
            store.putPendingAck(task);
        }
        store.save();
    }
}
```

- [ ] **Step 4: Run the test to verify it passes**

```bash
mvn -q -o test -Dtest=AckForwarderTest
```

Expected: `Tests run: 6, Failures: 0, Errors: 0`.

- [ ] **Step 5: Remove the superseded code from `Main`**

Delete `forwardAckToBackend` (`Main.java:689-719`), `enqueueAckForward` (515-532)
and `processPendingAckForwards` (457-513). Add a static
`AckForwarder ACK_FORWARDER = new AckForwarder(DEFAULT_HTTP, STORE, Env.get("AS2_CALLBACK_TOKEN"))`
and change the scheduled task registration in `main` to
`BG.scheduleAtFixedRate(ACK_FORWARDER::drain, 2, 2, TimeUnit.SECONDS);`.
In `handleAckCallback`, replace the forward-then-enqueue block with:

```java
        AckDecision decision = new AckDecision(
                req.ackLevel(), req.success(), req.ackCode(), req.ackMessage());
        boolean forwarded = ACK_FORWARDER.forward(existing, decision);
        if (!forwarded) {
            ACK_FORWARDER.enqueue(existing, decision, "initial forward failed");
        }
```

- [ ] **Step 6: Run the whole suite**

```bash
mvn -q -o test
```

Expected: BUILD SUCCESS.

- [ ] **Step 7: Commit**

```bash
git add src/main/java/com/example/as2/ack/AckForwarder.java \
        src/main/java/com/example/as2/Main.java \
        src/test/java/com/example/as2/ack/AckForwarderTest.java
git commit -m "refactor: extract AckForwarder with a contract test

Pins the callback body to exactly the five snake_case fields
GatewayAckCallbackInput deserializes, so a rename on either side fails a
test rather than silently dropping acknowledgements."
```

---

## Task 16: `as2/As2Receiver` — the inbound endpoint

This is the task the whole plan exists for. FDA delivers ACK3 and asynchronous
MDNs to our gateway over AS2; without this, no acknowledgement can ever arrive.

Two rules govern the design:

1. **Never answer 4xx or 5xx when an MDN can be produced.** RFC 4130 reports
   failures in the MDN disposition, and the FDA troubleshooting table names
   400/500 responses as a cause of lost acknowledgements.
2. **Never drop an unmatched acknowledgement.** It becomes an orphan, visible on
   `/internal/status`.

**Files:**
- Create: `src/main/java/com/example/as2/as2/ReceiverConfig.java`
- Create: `src/main/java/com/example/as2/as2/As2Receiver.java`
- Test: `src/test/java/com/example/as2/as2/As2ReceiverTest.java` (create)

- [ ] **Step 1: Write the failing test**

Create `src/test/java/com/example/as2/as2/As2ReceiverTest.java`:

```java
package com.example.as2.as2;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.example.as2.ack.AckForwarder;
import com.example.as2.state.Store;
import com.example.as2.state.SubmissionRecord;
import com.example.as2.testsupport.TestCerts;
import jakarta.mail.internet.MimeBodyPart;
import java.net.http.HttpClient;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.security.PrivateKey;
import java.security.cert.X509Certificate;
import java.util.HashMap;
import java.util.Map;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

class As2ReceiverTest {
    private static PrivateKey ourKey;
    private static X509Certificate ourCert;
    private static PrivateKey partnerKey;
    private static X509Certificate partnerCert;

    private static final String ACK_XML =
            "<MCCI_IN200101UV01 xmlns=\"urn:hl7-org:v3\">"
            + "<id extension=\"MSG-2026-0001\" root=\"x\"/>"
            + "<acknowledgement typeCode=\"CA\"/></MCCI_IN200101UV01>";

    @BeforeAll
    static void load() throws Exception {
        TestCerts.requireGenerated();
        ourKey = TestCerts.privateKey("submitter");
        ourCert = TestCerts.certificate("submitter");
        partnerKey = TestCerts.privateKey("partner");
        partnerCert = TestCerts.certificate("partner");
    }

    private static Store storeWithSubmission(Path dir) {
        Store store = new Store(dir.resolve("state.json"));
        SubmissionRecord record = new SubmissionRecord();
        record.submissionId = "s1";
        record.remoteSubmissionId = "<outbound@as2-submitter>";
        record.as2MessageId = "<outbound@as2-submitter>";
        record.caseId = "case-abc";
        record.e2bMessageIdentifier = "MSG-2026-0001";
        record.callbackUrl = null;
        store.putSubmission(record);
        return store;
    }

    private static ReceiverConfig config() {
        return new ReceiverConfig(
                "E2BR3-SUBMITTER",
                ourKey,
                ourCert,
                Map.of("FDA_AERS", partnerCert),
                "sha-256",
                false);
    }

    private static As2Receiver receiver(Store store) {
        return new As2Receiver(
                config(), store, new AckForwarder(HttpClient.newHttpClient(), store, "token"));
    }

    private static Map<String, String> headers(String filename, String contentType) {
        Map<String, String> h = new HashMap<>();
        h.put("as2-from", "FDA_AERS");
        h.put("as2-to", "E2BR3-SUBMITTER");
        h.put("message-id", "<inbound@fda>");
        h.put("content-type", contentType);
        h.put("disposition-notification-to", "FDA_AERS");
        if (filename != null) {
            h.put("content-disposition", "attachment; filename=\"" + filename + "\"");
        }
        return h;
    }

    @Test
    void acceptsAPlaintextAckAndReturnsAProcessedMdn(@TempDir Path dir) throws Exception {
        Store store = storeWithSubmission(dir);
        As2Receiver.Response response = receiver(store).process(
                headers("case-abc.ack", "application/xml"),
                ACK_XML.getBytes(StandardCharsets.UTF_8));

        assertEquals(200, response.status());
        String body = new String(response.body(), StandardCharsets.UTF_8);
        assertTrue(body.contains("Disposition: automatic-action/MDN-sent-automatically; processed"));
        assertTrue(body.contains("Original-Message-ID: <inbound@fda>"));
        assertEquals("E2BR3-SUBMITTER", response.headers().get("AS2-From"));
        assertEquals("FDA_AERS", response.headers().get("AS2-To"));
    }

    @Test
    void updatesTheCorrelatedSubmissionToAck3(@TempDir Path dir) throws Exception {
        Store store = storeWithSubmission(dir);
        receiver(store).process(
                headers("case-abc.ack", "application/xml"),
                ACK_XML.getBytes(StandardCharsets.UTF_8));
        assertEquals("ack3_received", store.getSubmission("s1").status);
    }

    @Test
    void decryptsAndVerifiesASignedEncryptedAck(@TempDir Path dir) throws Exception {
        Store store = storeWithSubmission(dir);
        MimeBodyPart payload = Cms.buildPayload(ACK_XML.getBytes(StandardCharsets.UTF_8), "application/xml");
        MimeBodyPart signed = Cms.sign(payload, partnerKey, partnerCert, "sha-256");
        MimeBodyPart encrypted = Cms.encrypt(signed, ourCert, "aes-256-cbc");

        As2Receiver.Response response = receiver(store).process(
                headers("case-abc.ack", encrypted.getContentType()),
                Cms.contentBytes(encrypted));

        assertEquals(200, response.status());
        String body = new String(response.body(), StandardCharsets.UTF_8);
        assertTrue(body.contains("processed"));
        assertTrue(body.contains("Received-Content-MIC: " + Mic.compute(payload, "sha-256")));
        assertEquals("ack3_received", store.getSubmission("s1").status);
    }

    @Test
    void anInboundMdnIsAck1(@TempDir Path dir) throws Exception {
        Store store = storeWithSubmission(dir);
        MimeBodyPart mdn = Mdn.build(new Mdn.Request(
                "FDA_AERS", "E2BR3-SUBMITTER", "<outbound@as2-submitter>",
                "abc=", "sha-256", Mdn.Disposition.PROCESSED, null, "openas2"));

        As2Receiver.Response response = receiver(store).process(
                headers(null, mdn.getContentType()), Cms.entityBytes(mdn));

        assertEquals(200, response.status());
        assertEquals("ack1_received", store.getSubmission("s1").status);
    }

    @Test
    void anUnknownPartnerStillGetsAnMdnAndIsNotAnError(@TempDir Path dir) throws Exception {
        Store store = storeWithSubmission(dir);
        Map<String, String> h = headers("case-abc.ack", "application/xml");
        h.put("as2-from", "SOMEONE_ELSE");

        As2Receiver.Response response =
                receiver(store).process(h, ACK_XML.getBytes(StandardCharsets.UTF_8));

        assertEquals(200, response.status());
        String body = new String(response.body(), StandardCharsets.UTF_8);
        assertTrue(body.contains("failed/Failure: unknown-as2-from"));
        assertEquals(1, store.orphanAckCount());
    }

    @Test
    void anUndecryptableBodyStillGetsAnMdnAndIsNotAnError(@TempDir Path dir) throws Exception {
        Store store = storeWithSubmission(dir);
        As2Receiver.Response response = receiver(store).process(
                headers("case-abc.ack", "application/pkcs7-mime; smime-type=enveloped-data"),
                "this is not CMS".getBytes(StandardCharsets.UTF_8));

        assertEquals(200, response.status());
        String body = new String(response.body(), StandardCharsets.UTF_8);
        assertTrue(body.contains("failed/Failure: decryption-failed"));
        assertEquals(1, store.orphanAckCount());
    }

    @Test
    void anUnmatchedAckBecomesAnOrphanAndStillGetsAProcessedMdn(@TempDir Path dir) throws Exception {
        Store store = storeWithSubmission(dir);
        String unrelated = "<MCCI_IN200101UV01 xmlns=\"urn:hl7-org:v3\">"
                + "<id extension=\"NOT-OURS\" root=\"x\"/></MCCI_IN200101UV01>";

        As2Receiver.Response response = receiver(store).process(
                headers("mystery.ack", "application/xml"),
                unrelated.getBytes(StandardCharsets.UTF_8));

        assertEquals(200, response.status());
        assertTrue(new String(response.body(), StandardCharsets.UTF_8).contains("processed"));
        assertEquals(1, store.orphanAckCount());
        assertEquals("mystery.ack", store.orphanAcks().get(0).filename);
        assertNotNull(store.orphanAcks().get(0).payloadBase64);
    }

    @Test
    void rejectsAnUnsignedMessageWhenSignaturesAreRequired(@TempDir Path dir) throws Exception {
        Store store = storeWithSubmission(dir);
        ReceiverConfig strict = new ReceiverConfig(
                "E2BR3-SUBMITTER", ourKey, ourCert, Map.of("FDA_AERS", partnerCert), "sha-256", true);
        As2Receiver strictReceiver = new As2Receiver(
                strict, store, new AckForwarder(HttpClient.newHttpClient(), store, "token"));

        As2Receiver.Response response = strictReceiver.process(
                headers("case-abc.ack", "application/xml"),
                ACK_XML.getBytes(StandardCharsets.UTF_8));

        assertEquals(200, response.status());
        assertTrue(new String(response.body(), StandardCharsets.UTF_8)
                .contains("failed/Failure: signature-required"));
        assertFalse("ack3_received".equals(store.getSubmission("s1").status));
    }

    @Test
    void aRejectingAcknowledgementMarksTheSubmissionRejected(@TempDir Path dir) throws Exception {
        Store store = storeWithSubmission(dir);
        String rejecting = "<MCCI_IN200101UV01 xmlns=\"urn:hl7-org:v3\">"
                + "<id extension=\"MSG-2026-0001\" root=\"x\"/>"
                + "<acknowledgement typeCode=\"CE\"/></MCCI_IN200101UV01>";

        receiver(store).process(
                headers("case-abc.ack", "application/xml"),
                rejecting.getBytes(StandardCharsets.UTF_8));

        assertEquals("rejected", store.getSubmission("s1").status);
    }
}
```

- [ ] **Step 2: Run it to verify it fails**

```bash
mvn -q -o test -Dtest=As2ReceiverTest
```

Expected: COMPILATION ERROR — `cannot find symbol: class ReceiverConfig`.

- [ ] **Step 3: Write the receiver configuration**

Create `src/main/java/com/example/as2/as2/ReceiverConfig.java`:

```java
package com.example.as2.as2;

import com.example.as2.config.Env;
import java.io.InputStream;
import java.io.StringReader;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.KeyStore;
import java.security.PrivateKey;
import java.security.cert.X509Certificate;
import java.util.LinkedHashMap;
import java.util.Map;
import org.bouncycastle.cert.X509CertificateHolder;
import org.bouncycastle.cert.jcajce.JcaX509CertificateConverter;
import org.bouncycastle.openssl.PEMParser;

/**
 * What the receiver needs to unwrap inbound messages.
 *
 * @param partnerCertificates AS2-From identifier to the certificate we expect that partner to sign with
 * @param requireSignature when true, an unsigned inbound message is refused in the MDN
 */
public record ReceiverConfig(
        String ourAs2Id,
        PrivateKey ourPrivateKey,
        X509Certificate ourCertificate,
        Map<String, X509Certificate> partnerCertificates,
        String micAlgorithm,
        boolean requireSignature) {

    /**
     * Builds from the environment. {@code AS2_RECEIVER_PARTNER_CERTS} maps AS2-From
     * identifiers to PEM paths, formatted as {@code FDA_AERS=certs/fda.crt,ZZFDA=certs/zzfda.crt}.
     */
    public static ReceiverConfig fromEnvironment() throws Exception {
        String ourAs2Id = Env.require("AS2_FROM_ID");

        PrivateKey key = null;
        X509Certificate certificate = null;
        String keystorePath = Env.get("AS2_SIGNING_PKCS12_PATH");
        String keystorePassword = Env.get("AS2_SIGNING_PKCS12_PASSWORD");
        if (keystorePath != null && keystorePassword != null) {
            KeyStore keyStore = KeyStore.getInstance("PKCS12");
            char[] password = keystorePassword.toCharArray();
            try (InputStream in = Files.newInputStream(Path.of(keystorePath))) {
                keyStore.load(in, password);
            }
            String alias = Env.get("AS2_SIGNING_KEY_ALIAS");
            if (alias == null) {
                var aliases = keyStore.aliases();
                while (aliases.hasMoreElements()) {
                    String candidate = aliases.nextElement();
                    if (keyStore.isKeyEntry(candidate)) {
                        alias = candidate;
                        break;
                    }
                }
            }
            if (alias != null) {
                key = (PrivateKey) keyStore.getKey(alias, password);
                certificate = (X509Certificate) keyStore.getCertificate(alias);
            }
        }

        Map<String, X509Certificate> partners = new LinkedHashMap<>();
        String configured = Env.get("AS2_RECEIVER_PARTNER_CERTS");
        if (configured != null) {
            for (String entry : configured.split(",")) {
                String[] pair = entry.split("=", 2);
                if (pair.length != 2) {
                    continue;
                }
                partners.put(pair[0].trim(), readPem(pair[1].trim()));
            }
        }

        return new ReceiverConfig(
                ourAs2Id,
                key,
                certificate,
                Map.copyOf(partners),
                Env.getOrDefault("AS2_MIC_ALGORITHM", "sha-256"),
                Env.envTruthy("AS2_RECEIVER_REQUIRE_SIGNATURE"));
    }

    private static X509Certificate readPem(String path) throws Exception {
        String pem = Files.readString(Path.of(path), StandardCharsets.UTF_8);
        try (PEMParser parser = new PEMParser(new StringReader(pem))) {
            Object object = parser.readObject();
            if (!(object instanceof X509CertificateHolder holder)) {
                throw new IllegalStateException("not a PEM certificate: " + path);
            }
            return new JcaX509CertificateConverter().setProvider("BC").getCertificate(holder);
        }
    }
}
```

- [ ] **Step 4: Write the receiver**

Create `src/main/java/com/example/as2/as2/As2Receiver.java`:

```java
package com.example.as2.as2;

import com.example.as2.ack.AckClassifier;
import com.example.as2.ack.AckDecision;
import com.example.as2.ack.AckCorrelator;
import com.example.as2.ack.AckForwarder;
import com.example.as2.state.OrphanAck;
import com.example.as2.state.Store;
import com.example.as2.state.SubmissionRecord;
import com.sun.net.httpserver.HttpExchange;
import com.sun.net.httpserver.HttpServer;
import jakarta.mail.internet.MimeBodyPart;
import java.io.ByteArrayInputStream;
import java.net.InetSocketAddress;
import java.nio.charset.StandardCharsets;
import java.security.cert.X509Certificate;
import java.time.OffsetDateTime;
import java.util.Base64;
import java.util.LinkedHashMap;
import java.util.Locale;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.Executors;

/**
 * The public AS2 endpoint. FDA delivers asynchronous MDNs, ACK2 and ACK3 here.
 *
 * <p>Failures are reported in the MDN disposition, never as an HTTP error
 * status: FDA treats a 400 or 500 as an undelivered acknowledgement.
 */
public class As2Receiver {

    /** An HTTP response, kept separate from the servlet plumbing so it can be asserted directly. */
    public record Response(int status, Map<String, String> headers, byte[] body) {}

    private final ReceiverConfig config;
    private final Store store;
    private final AckForwarder forwarder;

    public As2Receiver(ReceiverConfig config, Store store, AckForwarder forwarder) {
        this.config = config;
        this.store = store;
        this.forwarder = forwarder;
    }

    public HttpServer start(String bindAddress, int port, String path) throws Exception {
        HttpServer server = HttpServer.create(new InetSocketAddress(bindAddress, port), 0);
        server.createContext(path, this::handle);
        server.setExecutor(Executors.newFixedThreadPool(8));
        server.start();
        System.out.printf("AS2 receiver listening on http://%s:%d%s%n", bindAddress, port, path);
        return server;
    }

    private void handle(HttpExchange exchange) throws java.io.IOException {
        if (!"POST".equalsIgnoreCase(exchange.getRequestMethod())) {
            exchange.sendResponseHeaders(405, -1);
            exchange.close();
            return;
        }
        Map<String, String> headers = new LinkedHashMap<>();
        exchange.getRequestHeaders().forEach((name, values) -> {
            if (!values.isEmpty()) {
                headers.put(name.toLowerCase(Locale.ROOT), values.get(0));
            }
        });
        byte[] body = exchange.getRequestBody().readAllBytes();

        Response response = process(headers, body);
        response.headers().forEach((name, value) -> exchange.getResponseHeaders().set(name, value));
        exchange.sendResponseHeaders(response.status(), response.body().length);
        exchange.getResponseBody().write(response.body());
        exchange.close();
    }

    /** Unwrap, classify, correlate, forward, and answer with an MDN. */
    public Response process(Map<String, String> headers, byte[] body) {
        String as2From = headers.getOrDefault("as2-from", "");
        String messageId = headers.getOrDefault("message-id", "<unknown>");
        String contentType = headers.getOrDefault("content-type", "application/octet-stream");
        String filename = filenameFrom(headers.get("content-disposition"));

        X509Certificate partnerCert = config.partnerCertificates().get(as2From);
        if (partnerCert == null && !config.partnerCertificates().isEmpty()) {
            storeOrphan(as2From, messageId, filename, body, new AckDecision(
                    0, false, "AS2_UNKNOWN_PARTNER", "unknown AS2-From: " + as2From));
            return mdnResponse(as2From, messageId, null,
                    Mdn.Disposition.FAILED, "unknown-as2-from");
        }

        Unwrapped unwrapped;
        try {
            unwrapped = unwrap(contentType, body, partnerCert);
        } catch (SecurityException ex) {
            String detail = ex.getMessage() != null && ex.getMessage().contains("enveloped")
                    ? "decryption-failed"
                    : "authentication-failed";
            storeOrphan(as2From, messageId, filename, body,
                    new AckDecision(0, false, "AS2_UNWRAP_FAILED", ex.getMessage()));
            return mdnResponse(as2From, messageId, null, Mdn.Disposition.FAILED, detail);
        } catch (Exception ex) {
            storeOrphan(as2From, messageId, filename, body,
                    new AckDecision(0, false, "AS2_UNWRAP_FAILED", String.valueOf(ex.getMessage())));
            return mdnResponse(as2From, messageId, null, Mdn.Disposition.FAILED, "decryption-failed");
        }

        if (config.requireSignature() && !unwrapped.signed()) {
            storeOrphan(as2From, messageId, filename, body,
                    new AckDecision(0, false, "AS2_UNSIGNED", "signature required"));
            return mdnResponse(as2From, messageId, unwrapped.mic(),
                    Mdn.Disposition.FAILED, "signature-required");
        }

        Mdn.Parsed mdn = Mdn.parse(unwrapped.contentType(), unwrapped.payload());
        InboundMessage message = new InboundMessage(
                headers, as2From, headers.get("as2-to"), messageId,
                unwrapped.contentType(), filename, unwrapped.payload(),
                mdn.isMdn() ? mdn : null);

        AckDecision decision = AckClassifier.classify(message);
        SubmissionRecord record = AckCorrelator.correlate(store, message);

        if (record == null) {
            storeOrphan(as2From, messageId, filename, unwrapped.payload(), decision);
            return mdnResponse(as2From, messageId, unwrapped.mic(), Mdn.Disposition.PROCESSED, null);
        }

        record.status = statusFor(decision);
        record.updatedAt = OffsetDateTime.now().toString();
        if (decision.level() == 1) {
            record.mdnReceived = true;
            record.mdnDisposition = decision.message();
        }
        store.putSubmission(record);

        if (!forwarder.forward(record, decision)) {
            forwarder.enqueue(record, decision, "initial forward failed");
        }
        store.save();

        return mdnResponse(as2From, messageId, unwrapped.mic(), Mdn.Disposition.PROCESSED, null);
    }

    private record Unwrapped(byte[] payload, String contentType, String mic, boolean signed) {}

    private Unwrapped unwrap(String contentType, byte[] body, X509Certificate partnerCert)
            throws Exception {
        String type = contentType.toLowerCase(Locale.ROOT);
        MimeBodyPart part = new MimeBodyPart(new ByteArrayInputStream(
                (headerBlock(contentType) + new String(body, StandardCharsets.ISO_8859_1))
                        .getBytes(StandardCharsets.ISO_8859_1)));

        boolean signed = false;
        if (type.contains("pkcs7-mime") && type.contains("enveloped-data")) {
            if (config.ourPrivateKey() == null) {
                throw new SecurityException("enveloped message but no private key configured");
            }
            part = Cms.decrypt(part, config.ourPrivateKey(), config.ourCertificate());
        }

        String partType = part.getContentType() == null
                ? "" : part.getContentType().toLowerCase(Locale.ROOT);
        if (partType.contains("multipart/signed") || partType.contains("signed-data")) {
            if (partnerCert == null) {
                throw new SecurityException("signed message but no partner certificate configured");
            }
            MimeBodyPart content = Cms.verify(part, partnerCert);
            String mic = Mic.compute(content, config.micAlgorithm());
            return new Unwrapped(Cms.contentBytes(content), content.getContentType(), mic, true);
        }

        String mic = Mic.compute(part, config.micAlgorithm());
        return new Unwrapped(Cms.contentBytes(part), part.getContentType(), mic, signed);
    }

    private static String headerBlock(String contentType) {
        return "Content-Type: " + contentType + "\r\n\r\n";
    }

    private Response mdnResponse(
            String recipientAs2Id,
            String originalMessageId,
            String mic,
            Mdn.Disposition disposition,
            String detail) {
        try {
            Mdn.Request request = new Mdn.Request(
                    config.ourAs2Id(),
                    recipientAs2Id,
                    originalMessageId,
                    mic,
                    config.micAlgorithm(),
                    disposition,
                    detail,
                    As2Headers.USER_AGENT);
            MimeBodyPart mdn = Mdn.build(request);

            MimeBodyPart outbound = mdn;
            if (config.ourPrivateKey() != null && config.ourCertificate() != null) {
                outbound = Cms.sign(
                        mdn, config.ourPrivateKey(), config.ourCertificate(), config.micAlgorithm());
            }

            Map<String, String> headers = new LinkedHashMap<>();
            headers.put(As2Headers.CONTENT_TYPE, outbound.getContentType());
            headers.put(As2Headers.VERSION, As2Headers.VERSION_VALUE);
            headers.put(As2Headers.FROM, config.ourAs2Id());
            headers.put(As2Headers.TO, recipientAs2Id);
            headers.put(As2Headers.MESSAGE_ID, "<" + UUID.randomUUID() + "@as2-submitter>");
            return new Response(200, headers, Cms.entityBytes(outbound));
        } catch (Exception ex) {
            // An MDN we cannot even build is the one case where a status code is all we have.
            return new Response(
                    500,
                    Map.of(As2Headers.CONTENT_TYPE, "text/plain"),
                    ("MDN generation failed: " + ex.getMessage()).getBytes(StandardCharsets.UTF_8));
        }
    }

    private void storeOrphan(
            String as2From, String messageId, String filename, byte[] payload, AckDecision decision) {
        OrphanAck orphan = new OrphanAck();
        orphan.id = UUID.randomUUID().toString();
        orphan.receivedAt = OffsetDateTime.now().toString();
        orphan.as2From = as2From;
        orphan.as2MessageId = messageId;
        orphan.filename = filename;
        orphan.ackLevel = decision.level();
        orphan.success = decision.success();
        orphan.ackCode = decision.code();
        orphan.ackMessage = decision.message();
        orphan.payloadBase64 = Base64.getEncoder().encodeToString(payload);
        store.putOrphanAck(orphan);
        store.save();
    }

    private static String statusFor(AckDecision decision) {
        if (!decision.success()) {
            return "rejected";
        }
        return switch (decision.level()) {
            case 1 -> "ack1_received";
            case 2 -> "ack2_received";
            case 3 -> "ack3_received";
            case 4 -> "ack4_received";
            default -> "submitted_ack1_pending";
        };
    }

    static String filenameFrom(String contentDisposition) {
        if (contentDisposition == null) {
            return null;
        }
        int index = contentDisposition.toLowerCase(Locale.ROOT).indexOf("filename=");
        if (index < 0) {
            return null;
        }
        String value = contentDisposition.substring(index + "filename=".length()).trim();
        if (value.startsWith("\"")) {
            int close = value.indexOf('"', 1);
            return close < 0 ? value.substring(1) : value.substring(1, close);
        }
        int semicolon = value.indexOf(';');
        return semicolon < 0 ? value : value.substring(0, semicolon).trim();
    }
}
```

- [ ] **Step 5: Run the test to verify it passes**

```bash
mvn -q -o test -Dtest=As2ReceiverTest
```

Expected: `Tests run: 9, Failures: 0, Errors: 0`.

If `decryptsAndVerifiesASignedEncryptedAck` fails on the MIC assertion, the
`unwrap` reconstruction of the MIME entity from the raw HTTP body is losing or
adding bytes. Print both the expected and computed MIC and compare the entity
bytes — do not relax the assertion, because a wrong MIC here is exactly what a
real partner would reject.

- [ ] **Step 6: Commit**

```bash
git add src/main/java/com/example/as2/as2/As2Receiver.java \
        src/main/java/com/example/as2/as2/ReceiverConfig.java \
        src/test/java/com/example/as2/as2/As2ReceiverTest.java
git commit -m "feat: AS2 receiver for inbound MDNs and ACK files

FDA delivers ACK3 and asynchronous MDNs to our own gateway over AS2, so
a sender-only service can never observe an acknowledgement. Failures are
reported in the MDN disposition rather than as HTTP errors, which FDA
treats as undelivered, and unmatched acknowledgements become orphans
instead of being dropped."
```

---

## Task 17: Rewire `Main`, extract `api/ApiServer`, delete `openas2` mode

`Main` becomes wiring only. The `openas2` transport is removed: OpenAS2 is now
the counterparty in tests, not our transport.

**Files:**
- Create: `src/main/java/com/example/as2/api/ApiServer.java`
- Modify: `src/main/java/com/example/as2/Main.java` (reduce to wiring)
- Modify: `src/test/java/com/example/as2/MainTest.java` (move the two surviving tests)
- Test: `src/test/java/com/example/as2/api/ApiServerTest.java` (create)

- [ ] **Step 1: Write the failing test**

Create `src/test/java/com/example/as2/api/ApiServerTest.java`:

```java
package com.example.as2.api;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

class ApiServerTest {
    @Test
    void authAllowsWhenNoTokenIsConfigured() {
        assertTrue(ApiServer.isAuthorizedToken(null, null, null));
        assertTrue(ApiServer.isAuthorizedToken("", null, null));
    }

    @Test
    void authAllowsADirectHeaderTokenMatch() {
        assertTrue(ApiServer.isAuthorizedToken("secret-token", "secret-token", null));
    }

    @Test
    void authAllowsABearerTokenMatch() {
        assertTrue(ApiServer.isAuthorizedToken("secret-token", null, "Bearer secret-token"));
        assertTrue(ApiServer.isAuthorizedToken("secret-token", null, "bearer secret-token"));
    }

    @Test
    void authRejectsAMissingOrWrongToken() {
        assertFalse(ApiServer.isAuthorizedToken("secret-token", null, null));
        assertFalse(ApiServer.isAuthorizedToken("secret-token", "wrong", null));
        assertFalse(ApiServer.isAuthorizedToken("secret-token", null, "Bearer wrong"));
    }

    @Test
    void idempotencyKeysAreScopedByAuthorityAndCase() {
        assertEquals("fda:case-1:key-1", ApiServer.idempotencyLookup("fda", "case-1", "key-1"));
        assertEquals("mfds:case-1:key-1", ApiServer.idempotencyLookup("mfds", "case-1", "key-1"));
    }

    @Test
    void submitValidationRequiresTheThreeMandatoryFields() {
        assertEquals("caseId is required", ApiServer.validateSubmit(null, "fda", "<x/>"));
        assertEquals("authority is required (fda|mfds)", ApiServer.validateSubmit("c", null, "<x/>"));
        assertEquals("xmlPayload is required", ApiServer.validateSubmit("c", "fda", "  "));
        assertEquals("authority must be one of: fda, mfds", ApiServer.validateSubmit("c", "ema", "<x/>"));
        assertEquals(null, ApiServer.validateSubmit("c", "FDA", "<x/>"));
    }
}
```

- [ ] **Step 2: Run it to verify it fails**

```bash
mvn -q -o test -Dtest=ApiServerTest
```

Expected: COMPILATION ERROR — `package com.example.as2.api does not exist`.

- [ ] **Step 3: Create `ApiServer`**

Move these members out of `Main.java` into a new
`src/main/java/com/example/as2/api/ApiServer.java`, in package
`com.example.as2.api`:

| From `Main.java` | Becomes |
| --- | --- |
| `handleHealth`, `handleInternalStatus`, `handleSubmit`, `handleAckCallback` | private instance methods |
| `isAuthorizedToken` (807-822) | `public static`, unchanged behavior |
| `isInboundAuthorized` (794-805) | private instance method |
| `validateSubmitRequest` (721-736) | `public static String validateSubmit(String caseId, String authority, String xmlPayload)` |
| `submitResponse` (261-273) | private static |
| `deriveStatus` (738-749) | delete — the receiver derives inbound status itself, and submit sets its own |
| records `SubmitRequest`, `AckCallbackRequest` (956-971) | move as-is |
| `sendJson`, `readJson` (944-954) | private static |

Add the new static helper the test requires:

```java
    public static String idempotencyLookup(String authority, String caseId, String idempotencyKey) {
        return authority + ":" + caseId + ":" + idempotencyKey;
    }
```

Give it a constructor taking the collaborators rather than reaching for statics:

```java
    public ApiServer(Store store, As2Sender sender, AckForwarder forwarder, String inboundToken) { ... }

    public HttpServer start(int port) throws IOException {
        HttpServer server = HttpServer.create(new InetSocketAddress("127.0.0.1", port), 0);
        server.createContext("/health", this::handleHealth);
        server.createContext("/submit", this::handleSubmit);
        server.createContext("/callbacks/ack", this::handleAckCallback);
        server.createContext("/internal/status", this::handleInternalStatus);
        server.setExecutor(Executors.newFixedThreadPool(8));
        server.start();
        return server;
    }
```

In `handleSubmit`, replace the whole `dispatchToAuthority` path with the new
sender, and capture the correlation keys:

```java
        AuthorityProfile profile = AuthorityProfile.fromEnvironment(authority);
        As2Sender.Result result = sender.send(profile, caseId, req.xmlPayload());
        E2bIdentifiers identifiers = E2bIdentifiers.fromSubmission(req.xmlPayload());

        SubmissionRecord record = new SubmissionRecord();
        record.submissionId = UUID.randomUUID().toString();
        // The AS2 Message-ID is echoed back as Original-Message-ID in every MDN,
        // so using it as the remote id makes correlation fall out of the protocol.
        record.remoteSubmissionId = result.as2MessageId();
        record.authority = authority;
        record.caseId = caseId;
        record.status = result.mdnReceived() && result.mdnSuccess()
                ? "ack1_received" : "submitted_ack1_pending";
        record.updatedAt = OffsetDateTime.now().toString();
        record.callbackUrl = Env.normalize(req.callbackUrl());
        record.idempotencyKey = idempotencyKey;
        record.as2MessageId = result.as2MessageId();
        record.mdnReceived = result.mdnReceived();
        record.mdnDisposition = result.mdnDisposition();
        record.expectedMic = result.expectedMic();
        record.receivedMic = result.receivedMic();
        record.mdnMicMatch = result.micMatch();
        record.e2bBatchNumber = identifiers.batchNumber();
        record.e2bMessageIdentifier = identifiers.messageIdentifier();
        record.outboundFilename = result.outboundFilename();
```

Keep the MIC enforcement behavior: when `AS2_ENFORCE_MDN_MIC` is truthy and
`result.micMatch()` is `Boolean.FALSE`, respond `502` with
`{"error":"dispatch_failed","detail":"MDN MIC mismatch"}` and do not store the
record.

When an MDN came back synchronously and correlated, forward it as ACK1
immediately:

```java
        if (result.mdnReceived()) {
            AckDecision ack1 = new AckDecision(
                    1,
                    result.mdnSuccess(),
                    result.mdnSuccess() ? "ACK1_MDN_PROCESSED" : "ACK1_MDN_FAILED",
                    result.mdnDisposition());
            if (!forwarder.forward(record, ack1)) {
                forwarder.enqueue(record, ack1, "initial forward failed");
            }
        }
```

- [ ] **Step 4: Reduce `Main` to wiring**

Replace `src/main/java/com/example/as2/Main.java` entirely:

```java
package com.example.as2;

import com.example.as2.ack.AckForwarder;
import com.example.as2.api.ApiServer;
import com.example.as2.as2.As2Receiver;
import com.example.as2.as2.As2Sender;
import com.example.as2.as2.ReceiverConfig;
import com.example.as2.config.AuthorityProfile;
import com.example.as2.config.Env;
import com.example.as2.state.Store;
import java.net.http.HttpClient;
import java.nio.file.Path;
import java.time.Duration;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.TimeUnit;

public class Main {
    public static void main(String[] args) throws Exception {
        validateConfig();

        Store store = new Store(Path.of(Env.getOrDefault("AS2_STATE_FILE", "./as2-state.json")));
        store.load();

        HttpClient httpClient = HttpClient.newBuilder()
                .connectTimeout(Duration.ofSeconds(5))
                .version(HttpClient.Version.HTTP_1_1)
                .build();

        AckForwarder forwarder =
                new AckForwarder(httpClient, store, Env.get("AS2_CALLBACK_TOKEN"));
        As2Sender sender = new As2Sender(httpClient);

        ApiServer api = new ApiServer(store, sender, forwarder, Env.get("AS2_INBOUND_TOKEN"));
        int apiPort = Env.envInt("AS2_SUBMITTER_PORT", 9090);
        api.start(apiPort);
        System.out.printf("AS2 submitter API listening on http://127.0.0.1:%d%n", apiPort);

        As2Receiver receiver =
                new As2Receiver(ReceiverConfig.fromEnvironment(), store, forwarder);
        receiver.start(
                Env.getOrDefault("AS2_RECEIVER_BIND", "0.0.0.0"),
                Env.envInt("AS2_RECEIVER_PORT", 4080),
                Env.getOrDefault("AS2_RECEIVER_PATH", "/as2/receive"));

        ScheduledExecutorService background = Executors.newScheduledThreadPool(2);
        background.scheduleAtFixedRate(forwarder::drain, 2, 2, TimeUnit.SECONDS);
    }

    /**
     * Fails fast on an unusable configuration. In strict mode every combination
     * a real deployment needs must be present, so a misconfigured gateway does
     * not surface later as a silently undelivered submission.
     */
    static void validateConfig() {
        if (!Env.envTruthy("AS2_STRICT_MODE")) {
            return;
        }
        Env.require("AS2_CALLBACK_TOKEN");
        Env.require("AS2_INBOUND_TOKEN");
        Env.require("AS2_FROM_ID");

        // Building the profile validates routing mode, environment, endpoint and crypto material.
        AuthorityProfile fda = AuthorityProfile.fromEnvironment("fda");
        if (!fda.endpointUrl().toLowerCase(java.util.Locale.ROOT).startsWith("https://")) {
            throw new IllegalStateException("FDA endpoint must use https in strict mode");
        }
        if (Env.get("AS2_MFDS_TO_ID") != null) {
            AuthorityProfile.fromEnvironment("mfds");
        }
    }
}
```

- [ ] **Step 5: Serve the receiver over TLS when a keystore is configured**

FDA requires the industry gateway URL to be HTTPS on port 443 or 4080. Plaintext
stays the default because that is what levels 3 and 4 use.

In `As2Receiver`, replace the body of `start` with:

```java
    public HttpServer start(String bindAddress, int port, String path) throws Exception {
        String keystorePath = Env.get("AS2_RECEIVER_TLS_KEYSTORE_PATH");
        String keystorePassword = Env.get("AS2_RECEIVER_TLS_KEYSTORE_PASSWORD");
        InetSocketAddress address = new InetSocketAddress(bindAddress, port);

        HttpServer server;
        String scheme;
        if (keystorePath != null) {
            if (keystorePassword == null) {
                throw new IllegalStateException(
                        "AS2_RECEIVER_TLS_KEYSTORE_PATH requires AS2_RECEIVER_TLS_KEYSTORE_PASSWORD");
            }
            char[] password = keystorePassword.toCharArray();
            KeyStore keyStore = KeyStore.getInstance(
                    Env.getOrDefault("AS2_RECEIVER_TLS_KEYSTORE_TYPE", "PKCS12"));
            try (java.io.InputStream in = java.nio.file.Files.newInputStream(
                    java.nio.file.Path.of(keystorePath))) {
                keyStore.load(in, password);
            }
            KeyManagerFactory kmf =
                    KeyManagerFactory.getInstance(KeyManagerFactory.getDefaultAlgorithm());
            kmf.init(keyStore, password);
            SSLContext sslContext = SSLContext.getInstance("TLS");
            sslContext.init(kmf.getKeyManagers(), null, null);

            HttpsServer https = HttpsServer.create(address, 0);
            https.setHttpsConfigurator(new HttpsConfigurator(sslContext));
            server = https;
            scheme = "https";
        } else {
            server = HttpServer.create(address, 0);
            scheme = "http";
        }

        server.createContext(path, this::handle);
        server.setExecutor(Executors.newFixedThreadPool(8));
        server.start();
        System.out.printf("AS2 receiver listening on %s://%s:%d%s%n", scheme, bindAddress, port, path);
        return server;
    }
```

Add the imports it needs:

```java
import com.example.as2.config.Env;
import com.sun.net.httpserver.HttpsConfigurator;
import com.sun.net.httpserver.HttpsServer;
import java.security.KeyStore;
import javax.net.ssl.KeyManagerFactory;
import javax.net.ssl.SSLContext;
```

`As2ReceiverTest` calls `process` directly and `LoopbackEndToEndTest` sets no
keystore, so both keep working over plaintext.

- [ ] **Step 6: Retire the old `MainTest`**

`MainTest` covered `isAuthorizedToken` (now on `ApiServer` and covered by
`ApiServerTest`) and `validateTlsConfigForAuthority` (deleted along with the
per-authority TLS client builder, which `AuthorityProfile` and the shared
`HttpClient` replace). Delete `src/test/java/com/example/as2/MainTest.java`.

```bash
git rm src/test/java/com/example/as2/MainTest.java
```

- [ ] **Step 7: Run the whole suite**

```bash
mvn -q -o test
```

Expected: BUILD SUCCESS across `EnvTest`, `StoreTest`, `MicTest`, `CmsTest`,
`MdnTest`, `FdaRoutingTest`, `AuthorityProfileTest`, `E2bIdentifiersTest`,
`AckClassifierTest`, `AckCorrelatorTest`, `AckForwarderTest`, `As2SenderHeadersTest`,
`As2ReceiverTest`, `ApiServerTest`, `SmimeDependencyTest`, `TestCertsTest`.

- [ ] **Step 8: Verify the jar starts**

```bash
mvn -q -o package -DskipTests
AS2_FROM_ID=E2BR3-SUBMITTER AS2_RECEIVER_PORT=14080 AS2_SUBMITTER_PORT=19090 \
  timeout 5 java -jar target/as2-submitter.jar
```

Expected: two listening lines, then the timeout kills it. A
`no object DCH for MIME type` failure here means the shade plugin dropped
Jakarta Mail's service registrations — add the `ServicesResourceTransformer` as
noted in Task 2 Step 5.

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "refactor: Main becomes wiring; add the AS2 receiver listener

Extracts the loopback API into ApiServer and starts the public AS2
receiver alongside it. Deletes the openas2 transport mode: OpenAS2 is
now the counterparty in interop tests, not our transport. remote_submission_id
becomes our outbound Message-ID, which every MDN echoes back as
Original-Message-ID."
```

---

## Task 18: Loopback end-to-end test

Our sender against our receiver, in one JVM, with a stub backend asserting the
exact callback contract.

**Files:**
- Test: `src/test/java/com/example/as2/LoopbackEndToEndTest.java` (create)

- [ ] **Step 1: Write the test**

Create `src/test/java/com/example/as2/LoopbackEndToEndTest.java`:

```java
package com.example.as2;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.example.as2.ack.AckForwarder;
import com.example.as2.as2.As2Receiver;
import com.example.as2.as2.As2Sender;
import com.example.as2.as2.ReceiverConfig;
import com.example.as2.config.AuthorityProfile;
import com.example.as2.state.Store;
import com.example.as2.state.SubmissionRecord;
import com.example.as2.testsupport.TestCerts;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.sun.net.httpserver.HttpServer;
import java.net.InetSocketAddress;
import java.net.http.HttpClient;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

/** Our sender talking to our receiver, with a stub backend checking the callback contract. */
class LoopbackEndToEndTest {
    private static final ObjectMapper MAPPER = new ObjectMapper();
    private static final String SUBMISSION =
            "<MCCI_IN200100UV01 xmlns=\"urn:hl7-org:v3\">"
            + "<id extension=\"BATCH-LOOP-1\" root=\"r\"/>"
            + "<PORR_IN049016UV><id extension=\"MSG-LOOP-1\" root=\"r\"/></PORR_IN049016UV>"
            + "</MCCI_IN200100UV01>";

    private HttpServer backend;
    private HttpServer receiverServer;
    private final List<String> callbacks = new ArrayList<>();
    private String backendUrl;

    @BeforeEach
    void startBackend() throws Exception {
        TestCerts.requireGenerated();
        backend = HttpServer.create(new InetSocketAddress("127.0.0.1", 0), 0);
        backend.createContext("/ack", exchange -> {
            synchronized (callbacks) {
                callbacks.add(new String(exchange.getRequestBody().readAllBytes(), StandardCharsets.UTF_8));
            }
            exchange.sendResponseHeaders(200, 0);
            exchange.close();
        });
        backend.start();
        backendUrl = "http://127.0.0.1:" + backend.getAddress().getPort() + "/ack";
    }

    @AfterEach
    void stop() {
        if (backend != null) {
            backend.stop(0);
        }
        if (receiverServer != null) {
            receiverServer.stop(0);
        }
    }

    @Test
    void signedEncryptedSubmissionRoundTripsAndTheAckReachesTheBackend(@TempDir Path dir)
            throws Exception {
        Store store = new Store(dir.resolve("state.json"));
        AckForwarder forwarder = new AckForwarder(HttpClient.newHttpClient(), store, "callback-token");

        // The receiver plays the authority: it holds the partner key pair.
        ReceiverConfig authorityConfig = new ReceiverConfig(
                "FDA_AERS",
                TestCerts.privateKey("partner"),
                TestCerts.certificate("partner"),
                Map.of("E2BR3-SUBMITTER", TestCerts.certificate("submitter")),
                "sha-256",
                false);
        receiverServer = new As2Receiver(authorityConfig, store, forwarder)
                .start("127.0.0.1", 0, "/as2/receive");
        int port = receiverServer.getAddress().getPort();

        Map<String, String> env = new HashMap<>();
        env.put("AS2_FROM_ID", "E2BR3-SUBMITTER");
        env.put("AS2_FDA_ENDPOINT_URL", "http://127.0.0.1:" + port + "/as2/receive");
        env.put("AS2_ENABLE_CRYPTO", "1");
        env.put("AS2_SIGNING_PKCS12_PATH", "certs/submitter.p12");
        env.put("AS2_SIGNING_PKCS12_PASSWORD", "changeit");
        env.put("AS2_FDA_ENCRYPT_CERT_PEM_PATH", "certs/partner.crt");
        AuthorityProfile profile = AuthorityProfile.build("fda", env::get);

        As2Sender.Result result =
                new As2Sender(HttpClient.newHttpClient()).send(profile, "case-loop-1", SUBMISSION);

        assertTrue(result.mdnReceived(), "no MDN came back on the synchronous response");
        assertTrue(result.mdnSuccess(), "MDN disposition was not processed: " + result.mdnDisposition());
        assertEquals(
                Boolean.TRUE,
                result.micMatch(),
                "MIC mismatch: expected " + result.expectedMic() + " got " + result.receivedMic());
    }

    @Test
    void anInboundAckFileIsForwardedWithTheBackendContract(@TempDir Path dir) throws Exception {
        Store store = new Store(dir.resolve("state.json"));
        SubmissionRecord record = new SubmissionRecord();
        record.submissionId = "s1";
        record.remoteSubmissionId = "<outbound@as2-submitter>";
        record.as2MessageId = "<outbound@as2-submitter>";
        record.caseId = "case-loop-1";
        record.e2bMessageIdentifier = "MSG-LOOP-1";
        record.callbackUrl = backendUrl;
        store.putSubmission(record);

        AckForwarder forwarder = new AckForwarder(HttpClient.newHttpClient(), store, "callback-token");
        ReceiverConfig config = new ReceiverConfig(
                "E2BR3-SUBMITTER",
                TestCerts.privateKey("submitter"),
                TestCerts.certificate("submitter"),
                Map.of("FDA_AERS", TestCerts.certificate("partner")),
                "sha-256",
                false);

        String ack = "<MCCI_IN200101UV01 xmlns=\"urn:hl7-org:v3\">"
                + "<id extension=\"MSG-LOOP-1\" root=\"r\"/>"
                + "<acknowledgement typeCode=\"CA\"/></MCCI_IN200101UV01>";

        Map<String, String> headers = new HashMap<>();
        headers.put("as2-from", "FDA_AERS");
        headers.put("as2-to", "E2BR3-SUBMITTER");
        headers.put("message-id", "<ack@fda>");
        headers.put("content-type", "application/xml");
        headers.put("content-disposition", "attachment; filename=\"case-loop-1.ack\"");

        As2Receiver.Response response = new As2Receiver(config, store, forwarder)
                .process(headers, ack.getBytes(StandardCharsets.UTF_8));

        assertEquals(200, response.status());
        assertEquals("ack3_received", store.getSubmission("s1").status);

        assertEquals(1, callbacks.size(), "backend received no callback");
        JsonNode body = MAPPER.readTree(callbacks.get(0));
        assertEquals("<outbound@as2-submitter>", body.get("remote_submission_id").asText());
        assertEquals(3, body.get("ack_level").asInt());
        assertTrue(body.get("success").asBoolean());
        assertNotNull(body.get("ack_code"));
        assertEquals(5, body.size());
    }
}
```

- [ ] **Step 2: Run it**

```bash
./scripts/gen_test_certs.sh
mvn -q -o test -Dtest=LoopbackEndToEndTest
```

Expected: `Tests run: 2, Failures: 0, Errors: 0`.

- [ ] **Step 3: Commit**

```bash
git add src/test/java/com/example/as2/LoopbackEndToEndTest.java
git commit -m "test: loopback end-to-end sender to receiver

Proves the signed and encrypted round trip agrees on the MIC, and that an
inbound .ack reaches the backend with exactly the five snake_case fields
GatewayAckCallbackInput expects."
```

---

## Task 19: Reconfigure the OpenAS2 mock as an FDA counterparty

`~/projects/java/mock-mfds-openas2` is renamed and repartnered so it plays
`FDA_AERS` against our `E2BR3-SUBMITTER`, with the same certificates our tests
use.

**Working directory:** `/Users/hyundonghoon/projects/java`

**Files:**
- Rename: `mock-mfds-openas2/` to `mock-fda-openas2/`
- Modify: `mock-fda-openas2/config/partnerships.xml`
- Modify: `mock-fda-openas2/docker-compose.yml`
- Modify: `mock-fda-openas2/README.md`

- [ ] **Step 1: Rename and clear the stale test traffic**

```bash
cd /Users/hyundonghoon/projects/java
git -C mock-mfds-openas2 status --short 2>/dev/null || echo "not a git repo"
mv mock-mfds-openas2 mock-fda-openas2
rm -rf mock-fda-openas2/data/inbox/* mock-fda-openas2/data/mdn/* mock-fda-openas2/data/msgheaders/*
```

The March 2026 traffic under `data/` is from the old `MFDS_MOCK` partnership and
would otherwise be mistaken for output from a new run.

- [ ] **Step 2: Import our test certificates into the mock's keystore**

```bash
cd /Users/hyundonghoon/projects/java/mock-fda-openas2
keytool -importkeystore \
  -srckeystore /Users/hyundonghoon/projects/java/as2-submitter/certs/partner.p12 \
  -srcstoretype PKCS12 -srcstorepass changeit -srcalias partner \
  -destkeystore config/as2_certs.p12 -deststoretype PKCS12 -deststorepass changeit \
  -destalias fda_aers -noprompt
keytool -importcert -file /Users/hyundonghoon/projects/java/as2-submitter/certs/submitter.crt \
  -keystore config/as2_certs.p12 -storetype PKCS12 -storepass changeit \
  -alias e2br3_submitter -noprompt
keytool -list -keystore config/as2_certs.p12 -storetype PKCS12 -storepass changeit | grep -E "fda_aers|e2br3_submitter"
```

Expected: both aliases listed. `fda_aers` is a `PrivateKeyEntry`,
`e2br3_submitter` a `trustedCertEntry`.

- [ ] **Step 3: Repartner as FDA**

Replace `config/partnerships.xml`:

```xml
<partnerships>
  <partner name="Submitter"
           as2_id="E2BR3-SUBMITTER"
           x509_alias="e2br3_submitter"
           email="submitter@example.local"/>

  <partner name="FdaAers"
           as2_id="FDA_AERS"
           x509_alias="fda_aers"
           email="fda-aers-mock@example.local"/>

  <partnership name="Submitter-to-FdaAers">
    <sender name="Submitter"/>
    <receiver name="FdaAers"/>
    <attribute name="store_received_file_to"
               value="$properties.storageBaseDir$/inbox/$msg.receiver.as2_id$/$date.yyyy-MM-dd$/$msg.sender.as2_id$-$rand.12345$-$msg.content-disposition.filename$"/>
    <attribute name="reject_unsigned_messages" value="true"/>
    <attribute name="prevent_canonicalization_for_mic" value="false"/>
    <attribute name="sign" value="sha-256"/>
    <attribute name="encrypt" value="aes256"/>
  </partnership>

  <partnership name="FdaAers-to-Submitter">
    <sender name="FdaAers"/>
    <receiver name="Submitter"/>
    <attribute name="protocol" value="as2"/>
    <attribute name="as2_url" value="$properties.submitterAs2Url$"/>
    <attribute name="as2_mdn_to" value="FDA_AERS"/>
    <attribute name="as2_mdn_options"
               value="signed-receipt-protocol=required, pkcs7-signature; signed-receipt-micalg=required, sha-256"/>
    <attribute name="sign" value="sha-256"/>
    <attribute name="encrypt" value="aes256"/>
    <attribute name="content_transfer_encoding" value="binary"/>
  </partnership>
</partnerships>
```

`reject_unsigned_messages` is set to `true` so a regression that stops signing
fails loudly here rather than silently passing.

- [ ] **Step 4: Point the mock at our receiver and expose the ports**

Replace `docker-compose.yml`:

```yaml
services:
  openas2-mock-fda:
    image: local/openas2-mock-fda:dev
    build:
      context: https://github.com/OpenAS2/OpenAs2App.git#master
      dockerfile: Dockerfile
    container_name: openas2-mock-fda
    ports:
      - "5080:10080"
      - "5081:10081"
    environment:
      # Where OpenAS2 pushes ACK files and asynchronous MDNs back to us.
      SUBMITTER_AS2_URL: "${SUBMITTER_AS2_URL:-http://host.docker.internal:4080/as2/receive}"
    extra_hosts:
      - "host.docker.internal:host-gateway"
    volumes:
      - ./config:/opt/openas2/config
      - ./data:/opt/openas2/data
```

Add `submitterAs2Url` to `config/config.xml` inside its `<properties>` element so
`$properties.submitterAs2Url$` resolves:

```xml
    <property name="submitterAs2Url" value="${SUBMITTER_AS2_URL}"/>
```

- [ ] **Step 5: Start it and confirm it is listening**

```bash
cd /Users/hyundonghoon/projects/java/mock-fda-openas2
docker compose up --build -d
sleep 20
docker compose logs --tail 40 | grep -iE "started|listening|partnership" | head
curl -s -o /dev/null -w "%{http_code}\n" -X POST http://127.0.0.1:5080/
```

Expected: startup lines mentioning the two partnerships, and a non-connection-refused
status from the curl (OpenAS2 rejects the empty body, which is fine — we are only
proving the port is open).

- [ ] **Step 6: Update the README and commit**

Rewrite `README.md` to describe the FDA role: partner IDs `E2BR3-SUBMITTER` and
`FDA_AERS`, ports 5080/5081, the `SUBMITTER_AS2_URL` variable, and the fact that
this stands in for `upload-api-esgng.fda.gov:4080`.

```bash
cd /Users/hyundonghoon/projects/java/mock-fda-openas2
git add -A 2>/dev/null && git commit -m "chore: repartner the OpenAS2 mock as FDA_AERS" 2>/dev/null \
  || echo "not a git repo; nothing to commit"
```

---

## Task 20: Cross-implementation interop run (local)

**Files:**
- Create: `/Users/hyundonghoon/projects/java/as2-submitter/scripts/interop_fda.sh`
- Delete: `scripts/openas2_submit_verify.sh`, `scripts/run_interop_matrix.sh` (both assume the removed `openas2` transport)

- [ ] **Step 1: Write the interop script**

Create `scripts/interop_fda.sh`:

```bash
#!/usr/bin/env bash
# Level 3: our AS2 submitter against OpenAS2 acting as FDA_AERS.
#
# Proves both directions:
#   outbound - OpenAS2 accepts our signed+encrypted submission and returns a
#              signed MDN whose MIC matches ours
#   inbound  - OpenAS2 delivers a .ack over AS2 to our receiver, which decrypts,
#              verifies, correlates and answers with an MDN OpenAS2 accepts
set -euo pipefail

SUBMITTER_DIR="${SUBMITTER_DIR:-/Users/hyundonghoon/projects/java/as2-submitter}"
MOCK_DIR="${MOCK_DIR:-/Users/hyundonghoon/projects/java/mock-fda-openas2}"
MOCK_AS2_URL="${MOCK_AS2_URL:-http://127.0.0.1:5080}"
RECEIVER_PORT="${RECEIVER_PORT:-4080}"
API_PORT="${API_PORT:-9090}"
BACKEND_PORT="${BACKEND_PORT:-8099}"
TOKEN="${AS2_INBOUND_TOKEN:-interop-token}"
WORK="${WORK:-/tmp/as2-interop-fda}"

rm -rf "$WORK" && mkdir -p "$WORK"
cd "$SUBMITTER_DIR"

echo "==> 1/6 certificates"
[ -f certs/submitter.p12 ] || ./scripts/gen_test_certs.sh

echo "==> 2/6 build"
mvn -q -o package -DskipTests

echo "==> 3/6 stub backend on :$BACKEND_PORT"
python3 - "$BACKEND_PORT" "$WORK/callbacks.jsonl" <<'PY' &
import sys, json
from http.server import BaseHTTPRequestHandler, HTTPServer
port, out = int(sys.argv[1]), sys.argv[2]
class H(BaseHTTPRequestHandler):
    def do_POST(self):
        body = self.rfile.read(int(self.headers.get("content-length", 0)))
        with open(out, "a") as f:
            f.write(body.decode("utf-8") + "\n")
        self.send_response(200); self.end_headers()
    def log_message(self, *a): pass
HTTPServer(("127.0.0.1", port), H).serve_forever()
PY
BACKEND_PID=$!
trap 'kill $BACKEND_PID 2>/dev/null || true; kill ${SUBMITTER_PID:-0} 2>/dev/null || true' EXIT
sleep 1

echo "==> 4/6 mock FDA gateway"
( cd "$MOCK_DIR" && SUBMITTER_AS2_URL="http://host.docker.internal:$RECEIVER_PORT/as2/receive" \
    docker compose up -d >/dev/null )
sleep 20

echo "==> 5/6 submitter"
AS2_FROM_ID=E2BR3-SUBMITTER \
AS2_FDA_ROUTING_MODE=routing_id \
AS2_FDA_ENDPOINT_URL="$MOCK_AS2_URL" \
AS2_FDA_ENCRYPT_CERT_PEM_PATH="$SUBMITTER_DIR/certs/partner.crt" \
AS2_ENABLE_CRYPTO=1 \
AS2_ENFORCE_MDN_MIC=1 \
AS2_SIGNING_PKCS12_PATH="$SUBMITTER_DIR/certs/submitter.p12" \
AS2_SIGNING_PKCS12_PASSWORD=changeit \
AS2_RECEIVER_PARTNER_CERTS="FDA_AERS=$SUBMITTER_DIR/certs/partner.crt" \
AS2_RECEIVER_PORT="$RECEIVER_PORT" \
AS2_SUBMITTER_PORT="$API_PORT" \
AS2_INBOUND_TOKEN="$TOKEN" \
AS2_CALLBACK_TOKEN="$TOKEN" \
AS2_STATE_FILE="$WORK/state.json" \
  java -jar target/as2-submitter.jar > "$WORK/submitter.log" 2>&1 &
SUBMITTER_PID=$!
sleep 3

echo "==> 6/6 submit"
CASE_ID="interop-$(date +%s)"
cat > "$WORK/payload.xml" <<XML
<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3">
  <id extension="BATCH-$CASE_ID" root="2.16.840.1.113883.3.989.2.1.3.22"/>
  <PORR_IN049016UV><id extension="MSG-$CASE_ID" root="2.16.840.1.113883.3.989.2.1.3.1"/></PORR_IN049016UV>
</MCCI_IN200100UV01>
XML

python3 - "$WORK/payload.xml" "$CASE_ID" "$BACKEND_PORT" > "$WORK/request.json" <<'PY'
import json, sys
xml = open(sys.argv[1]).read()
print(json.dumps({
    "caseId": sys.argv[2],
    "authority": "fda",
    "xmlPayload": xml,
    "callbackUrl": f"http://127.0.0.1:{sys.argv[3]}/internal/submissions/callbacks/ack",
}))
PY

curl -sS -X POST "http://127.0.0.1:$API_PORT/submit" \
  -H "content-type: application/json" -H "x-api-token: $TOKEN" \
  --data @"$WORK/request.json" | tee "$WORK/submit-response.json"
echo

echo
echo "==> outbound checks"
python3 - "$WORK/submit-response.json" <<'PY'
import json, sys
r = json.load(open(sys.argv[1]))
ok = True
def check(label, cond):
    global ok
    print(f"  [{'PASS' if cond else 'FAIL'}] {label}")
    ok = ok and cond
check("MDN received", r.get("mdn_received") is True)
check("MDN MIC matches", r.get("mdn_mic_match") is True)
check("remote id is our Message-ID", str(r.get("remote_submission_id", "")).endswith("@as2-submitter>"))
sys.exit(0 if ok else 1)
PY

echo
echo "==> payload landed in the mock's inbox"
find "$MOCK_DIR/data/inbox/FDA_AERS" -type f -newermt '-2 minutes' | tail -3

echo
echo "==> now deliver an ACK back over AS2"
cat > "$WORK/ack.xml" <<XML
<MCCI_IN200101UV01 xmlns="urn:hl7-org:v3">
  <id extension="MSG-$CASE_ID" root="2.16.840.1.113883.3.989.2.1.3.1"/>
  <acknowledgement typeCode="CA"/>
</MCCI_IN200101UV01>
XML
docker compose -f "$MOCK_DIR/docker-compose.yml" cp \
  "$WORK/ack.xml" "openas2-mock-fda:/opt/openas2/data/outbox/E2BR3-SUBMITTER/$CASE_ID.ack" \
  2>/dev/null || docker exec openas2-mock-fda sh -c \
  "mkdir -p /opt/openas2/data/outbox/E2BR3-SUBMITTER && cat > /opt/openas2/data/outbox/E2BR3-SUBMITTER/$CASE_ID.ack" \
  < "$WORK/ack.xml"

echo "  waiting up to 60s for the backend callback..."
for _ in $(seq 1 60); do
  if [ -s "$WORK/callbacks.jsonl" ] && grep -q '"ack_level": *3' "$WORK/callbacks.jsonl"; then
    break
  fi
  sleep 1
done

echo
echo "==> inbound checks"
python3 - "$WORK/callbacks.jsonl" <<'PY'
import json, os, sys
path = sys.argv[1]
if not os.path.exists(path):
    print("  [FAIL] no callbacks received"); sys.exit(1)
rows = [json.loads(line) for line in open(path) if line.strip()]
ok = True
def check(label, cond):
    global ok
    print(f"  [{'PASS' if cond else 'FAIL'}] {label}")
    ok = ok and cond
check("at least one callback", len(rows) > 0)
check("an ACK3 arrived", any(r.get("ack_level") == 3 for r in rows))
check("ACK3 reports success", all(r.get("success") for r in rows if r.get("ack_level") == 3))
check("body has exactly the five contract fields", all(
    set(r) == {"remote_submission_id", "ack_level", "success", "ack_code", "ack_message"} for r in rows))
sys.exit(0 if ok else 1)
PY

echo
echo "Artifacts in $WORK"
```

- [ ] **Step 2: Delete the scripts that assumed the removed transport**

```bash
cd /Users/hyundonghoon/projects/java/as2-submitter
git rm scripts/openas2_submit_verify.sh scripts/run_interop_matrix.sh
```

- [ ] **Step 3: Run it**

```bash
chmod +x scripts/interop_fda.sh
./scripts/interop_fda.sh
```

Expected: every outbound and inbound check reports `PASS`.

A MIC mismatch here is the finding this whole level exists to produce. Debug it
against `/tmp/as2-interop-fda/submitter.log` and the mock's
`data/msgheaders/`, which records the exact headers OpenAS2 saw. Do not disable
`AS2_ENFORCE_MDN_MIC` to get a green run.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "test: cross-implementation interop against OpenAS2 as FDA

Two independent AS2 implementations agreeing on MIC, canonicalization,
certificates and MDN structure is the closest available proxy for FDA
interoperability. Removes the scripts that assumed the openas2 transport."
```

---

## Task 21: Local to EC2 run

Same as Task 20 with the mock on EC2, which is the only way to exercise the
asynchronous MDN path and real TLS.

**Files:**
- Create: `docs/interop-ec2.md` in `as2-submitter`

- [ ] **Step 1: Write the runbook**

Create `docs/interop-ec2.md`:

```markdown
# Level 4: local submitter, EC2 gateway

Level 3 runs both sides on one host, so the "asynchronous" MDN completes before
the sending call returns and never really exercises the receiver. Putting the
gateway on EC2 forces a genuine second connection back to us.

## EC2 side

1. Launch a small instance with Docker, in a security group allowing inbound
   TCP 5080 and 5081 from your workstation's public IP only.
2. Copy the mock and start it, pointing it back at your workstation:

   ```bash
   scp -r ~/projects/java/mock-fda-openas2 ec2-user@$EC2_HOST:~/
   ssh ec2-user@$EC2_HOST \
     "cd mock-fda-openas2 && SUBMITTER_AS2_URL=http://$MY_PUBLIC_IP:4080/as2/receive docker compose up -d"
   ```

## Workstation side

Your receiver must be reachable from EC2. Either open TCP 4080 inbound on your
router, or run a reverse tunnel:

```bash
ssh -R 4080:127.0.0.1:4080 ec2-user@$EC2_HOST -N &
```

With the tunnel, set `SUBMITTER_AS2_URL=http://127.0.0.1:4080/as2/receive` on
the EC2 side instead of your public IP.

Then run the level 3 script against the remote gateway, with asynchronous MDN
enabled:

```bash
MOCK_AS2_URL="http://$EC2_HOST:5080" \
AS2_ASYNC_MDN_URL="http://$MY_PUBLIC_IP:4080/as2/receive" \
  ./scripts/interop_fda.sh
```

## What differs from level 3

- `mdn_received` is now **false** on the submit response, because the MDN is
  asynchronous. The submission is `submitted_ack1_pending`.
- ACK1 arrives moments later as a separate callback from the receiver. The run
  is green when the backend has recorded both an `ack_level` 1 and an
  `ack_level` 3 callback.
- Real network latency, MTU and TLS termination are in the path.

## Checks

```bash
grep -c '"ack_level": 1' /tmp/as2-interop-fda/callbacks.jsonl   # expect >= 1
grep -c '"ack_level": 3' /tmp/as2-interop-fda/callbacks.jsonl   # expect >= 1
```

## Teardown

```bash
ssh ec2-user@$EC2_HOST "cd mock-fda-openas2 && docker compose down"
kill %1   # the reverse tunnel, if used
```

Terminate the instance when finished — the security group opens ports to the
internet and should not outlive the test.
```

- [ ] **Step 2: Extend the interop script for the asynchronous case**

In `scripts/interop_fda.sh`, the outbound check block asserts
`mdn_received is True`. Make that conditional so the same script serves both
levels — replace the two MDN checks with:

```python
if os.environ.get("AS2_ASYNC_MDN_URL"):
    check("MDN is asynchronous, so none on the response", r.get("mdn_received") is not True)
else:
    check("MDN received", r.get("mdn_received") is True)
    check("MDN MIC matches", r.get("mdn_mic_match") is True)
```

and add `import os` to that block. Pass `AS2_ASYNC_MDN_URL` through to the
submitter's environment in the launch block.

- [ ] **Step 3: Run it against EC2 and record the result**

Follow `docs/interop-ec2.md`. Expected: all checks `PASS`, and both an
`ack_level` 1 and an `ack_level` 3 callback in `callbacks.jsonl`.

- [ ] **Step 4: Commit**

```bash
git add docs/interop-ec2.md scripts/interop_fda.sh
git commit -m "test: EC2 runbook for asynchronous MDN verification

A same-host run completes the asynchronous MDN before the sending call
returns, so it never exercises the receiver. A remote gateway forces a
genuine second connection back to us."
```

---

## Task 22: Documentation

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Rewrite the README**

Update it to match reality:

- **Endpoints:** the loopback API (`/health`, `/internal/status`, `/submit`,
  `/callbacks/ack` on 9090) and the public AS2 receiver
  (`/as2/receive` on 4080). State that `/callbacks/ack` is a manual injection
  path for operators and tests, not how real acknowledgements arrive.
- **Features:** FDA routing modes, S/MIME sign and encrypt, MDN generation and
  verification, MIC enforcement, ACK classification and correlation, orphan
  retention, backend callback with retry.
- **Environment variables:** the full table from Task 13, plus
  `AS2_RECEIVER_PORT`, `AS2_RECEIVER_BIND`, `AS2_RECEIVER_PATH`,
  `AS2_RECEIVER_PARTNER_CERTS`, `AS2_RECEIVER_REQUIRE_SIGNATURE`,
  `AS2_RECEIVER_TLS_KEYSTORE_PATH`, `AS2_ACK_LEVEL_BY_EXT`,
  `AS2_ENFORCE_MDN_MIC`, `AS2_STRICT_MODE`, `AS2_STATE_FILE`,
  `AS2_INBOUND_TOKEN`, `AS2_CALLBACK_TOKEN`, and the ACK forward retry trio.
- **Remove** every mention of `AS2_TRANSPORT_MODE`, `AS2_OPENAS2_BASE_DIR`, the
  OpenAS2 setup section, the old compose section, and the per-authority TLS
  truststore and keystore variables, all of which no longer exist.
- **Testing:** the four levels, with the commands for each.
- **FDA onboarding:** note that production access requires the Gateway
  Configuration Information form emailed to ESGNGSupport@fda.hhs.gov, that our
  gateway URL must be HTTPS on port 443 or 4080, and that FDA supplies one
  routing ID per signing certificate.

- [ ] **Step 2: Verify the documented variables all exist**

```bash
cd /Users/hyundonghoon/projects/java/as2-submitter
for v in $(grep -oE 'AS2_[A-Z0-9_]+' README.md | sort -u); do
  grep -rq "\"$v\"" src/main/java || echo "documented but unused: $v"
done
for v in $(grep -rhoE '"AS2_[A-Z0-9_]+"' src/main/java | tr -d '"' | sort -u); do
  grep -q "$v" README.md || echo "used but undocumented: $v"
done
```

Expected: no output.

- [ ] **Step 3: Final full verification**

```bash
mvn -o clean test
```

Expected: BUILD SUCCESS, zero failures, zero errors.

- [ ] **Step 4: Commit**

```bash
git add README.md
git commit -m "docs: document the receiver, FDA routing and the test levels

Removes the transport-mode and per-authority TLS variables that no longer
exist, and adds the receiver configuration and FDA onboarding notes."
```

---

## Done

At this point:

- The submitter speaks AS2 in both directions with proper S/MIME.
- FDA routing follows the guide, with unusable header combinations rejected at
  startup.
- MDNs are generated, parsed and signature-verified, and the MIC is computed
  over the right bytes.
- Inbound `.ack` files reach the backend as ACK3 with the existing contract
  unchanged, and unmatched acknowledgements survive as orphans.
- Four levels of test, the third of which is checked by an independent AS2
  implementation.

Still open, and deliberately out of scope: MFDS transport, which needs an
official specification before it can be built on anything better than guesswork.
