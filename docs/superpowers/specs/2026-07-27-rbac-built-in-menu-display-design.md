# Built-in RBAC Menu Display Consistency Design

## Problem

The authorization registry gives the Platform Administrator only the
`admin.edit` grant, whose effective closure includes `admin.read`. Runtime
`eligibleActions` are compiled from those grants. However,
`built_in_menu_privileges` separately hardcodes the Platform Administrator's
allowed menus as `home_notice` and `admin`, so the profile advertises Notice
Read/Edit even though Notice actions are denied.

This is a remaining duplicate authorization representation: displayed menu
privileges and executable grants do not share the same source.

## Decision

The policy registry remains the sole source for built-in role grants. Displayed
menu privileges for a built-in role will be derived from that identity's
effective registry grants, including implied grants, and then converted through
the grants' canonical UI bindings.

The Platform Administrator remains a platform control-plane identity. It will
not receive new Notice actions merely to match stale display metadata. Its
profile will advertise only the `admin` privilege row supported by its actual
effective grants. Sponsor Administrator identities retain their existing Safety
Database grants and corresponding menu rows.

## Data Flow

1. Canonicalize the role identifier and resolve its built-in identity kind.
2. Read that identity's direct grants from `PolicyRegistry`.
3. Expand implied grants with `PolicyRegistry::effective_grants`.
4. For each implemented effective grant, set its canonical UI field on the
   corresponding menu row.
5. Return an empty list for roles that are not built-in identities.

No menu allowlist or role-specific privilege list will remain in the display
adapter.

## API Behaviour

- `/api/users/me/profile.privileges` and `eligibleActions` describe the same
  effective grant set for built-in identities.
- The Platform Administrator no longer advertises `home_notice` Read/Edit.
- Sponsor CRO and Company Administrators keep their existing fixed menu
  privileges.
- Custom role storage and normalization are unchanged.
- PDF Role & Privilege rows and their availability are unchanged.

## Verification

- Add a unit regression proving every built-in displayed UI flag maps to an
  effective grant owned by that identity, and every effective implemented grant
  with a UI binding is displayed.
- Add an explicit Platform Administrator regression proving `admin` is present
  and `home_notice` is absent.
- Preserve Sponsor Administrator privilege expectations.
- Run the focused `lib-core` authorization tests and the web profile/role-admin
  authorization tests.
- Regenerate/check the frontend authorization contract only if the canonical
  registry contract changes; this design does not change it.

## Non-goals

- Granting operational Notice permissions to the Platform Administrator.
- Changing custom-role privileges or PDF rows.
- Implementing the reserved Report Due Mail feature.
- Refactoring unrelated administrator routes.
