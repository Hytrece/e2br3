# Export-Owned Message Header Design

## Goal

Treat ICH E2B(R3) N.1/N.2 message-header data as outbound transport-envelope
metadata. The case editor must not expose or persist that data through either
the CI or SD page.

## Ownership

- CI owns case-identification fields only.
- SD owns sender and receiver case data only.
- Export/Submission owns message-header routing and generated transmission
  values.
- The existing standalone backend `MessageHeaderBmc` remains the persistence
  boundary. It must not be duplicated inside an SD-specific BMC.

## Frontend changes

- Remove the Message Header inputs from the SD page.
- Remove Message Header dirty snapshots, save tasks, contracts, and path
  ownership from both CI and SD case-editor save paths.
- Remove N.1/N.2 rows from the SD editor contract because they are not editable
  SD fields.
- Before XML export or regulatory submission, build the outbound header from:
  - the selected sender routing template;
  - the selected receiver routing option;
  - freshly generated message/batch identifiers and timestamps.
- Generate `batchTransmissionDate` in the same export/submission preparation
  path. It must not depend on a value manually entered in the case editor.

## Backend changes

- Keep `MessageHeaderBmc` as the independent message-header model.
- Keep its API endpoint for the export/submission preparation boundary.
- Remove any SD-specific editor mapping to message-header fields.
- Do not move message-header columns into sender or case-identification models.

## Data flow

1. A user edits and saves CI or SD without touching message-header storage.
2. The user chooses an export/submission authority, sender route, and receiver
   route.
3. Export/submission preparation creates the outbound N.1/N.2 values and writes
   them through the standalone message-header endpoint.
4. XML export reads that prepared header.
5. Submission/export history retains the resulting outbound artifact.

Imported message headers may remain available in backend storage for XML
round-trip compatibility, but they are not editable through CI or SD and must
not define the outbound route when the export/submission preparation supplies a
new value.

## Tests

- SD renders no Message Header inputs.
- CI and SD saves never call the message-header endpoint.
- CI/SD persistence contracts contain no message-header ownership.
- Export and submission preparation populate message number, batch number,
  sender identifiers, receiver identifiers, message date, and batch
  transmission date.
- The SD editor registry contains no N.1/N.2 editor fields.
- Existing standalone `MessageHeaderBmc` model and XML export coverage continue
  to pass.

