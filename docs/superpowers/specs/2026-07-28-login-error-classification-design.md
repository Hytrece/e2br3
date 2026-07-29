# Login Error Classification Design

## Goal

Replace the single public `LOGIN_FAIL` response with a small set of stable,
user-actionable login error codes while preserving the existing backend error
pipeline, frontend API client, HTTP behavior, and protection of credential
details.

## Current Behavior and Root Cause

The login handler already distinguishes `LoginFailEmailNotFound`,
`LoginFailUserHasNoPwd`, `LoginFailPwdNotMatching`, and
`LoginFailUserCtxCreate`. The public mapping in
`lib-web::error::Error::client_status_and_error` collapses all four variants to
`ClientError::LOGIN_FAIL`. The frontend API client extracts that public variant
as `ApiError.code`, but `AuthContext.login` discards the code and forwards only
the raw message to the login page.

Consequently the login page displays `LOGIN_FAIL` rather than a useful message.
The failure is not caused by the global API client hiding the error; it is
caused by coarse backend classification followed by loss of the structured code
at the authentication-context boundary.

## Public Error Contract

The public client-error enum will expose exactly these login variants:

- `LOGIN_INVALID_CREDENTIALS`
- `LOGIN_PASSWORD_NOT_CONFIGURED`
- `LOGIN_ACCOUNT_UNAVAILABLE`

Internal errors map as follows:

| Internal error | Public error |
| --- | --- |
| `LoginFailEmailNotFound` | `LOGIN_INVALID_CREDENTIALS` |
| `LoginFailPwdNotMatching` | `LOGIN_INVALID_CREDENTIALS` |
| `LoginFailUserHasNoPwd` | `LOGIN_PASSWORD_NOT_CONFIGURED` |
| `LoginFailUserCtxCreate` | `LOGIN_ACCOUNT_UNAVAILABLE` |

Unknown-email and wrong-password failures remain indistinguishable to prevent
account enumeration. Public responses must never contain `user_id` or the
internal error variant.

All three errors retain the current `403 Forbidden` status. This change does
not alter middleware routing, response serialization, cookies, token handling,
or the common error response shape.

## Frontend Handling

`AuthContext.login` will preserve the API client's existing `ApiError.code` in
its result instead of reducing the failure to a message string. A focused login
error-message mapper will translate the three approved codes:

| Error code | User-facing message |
| --- | --- |
| `LOGIN_INVALID_CREDENTIALS` | `Email or password is incorrect.` |
| `LOGIN_PASSWORD_NOT_CONFIGURED` | `A password has not been configured for this account. Contact your administrator.` |
| `LOGIN_ACCOUNT_UNAVAILABLE` | `This account is currently unavailable. Contact your administrator.` |

Network errors and unrecognized codes retain the existing safe fallback
behavior. The login page continues to use its existing inline error banner and
toast; no visual redesign is included.

The mapper must use `code`, not backend debug detail. This prevents development
debug details from becoming user-visible login copy.

## Explicitly Out of Scope

- Distinguishing an unknown email from an incorrect password.
- Adding account-locking, rate-limiting, or password-expiration state.
- Separately exposing inactive accounts or access-window expiration. The current
  database query filters those rows before the handler can classify them.
- Changing HTTP status codes or the shared error response schema.
- Refactoring general frontend error handling.

## Testing

Backend tests will verify each internal-to-public mapping, including the shared
invalid-credentials classification. Existing request-level wrong-password and
unknown-email tests will additionally assert the public response code.

Frontend tests will verify the three code-to-copy mappings and the safe fallback.
The login flow test will verify that `AuthContext.login` preserves the structured
code and that the login page displays mapped copy rather than the raw enum name.

## Success Criteria

- Wrong password and unknown email both return
  `LOGIN_INVALID_CREDENTIALS` with no distinguishing detail.
- Missing password and invalid account context return their own stable public
  codes.
- The login page shows actionable English copy for all three codes.
- Existing non-login error behavior remains unchanged.
- Targeted backend and frontend tests pass.
