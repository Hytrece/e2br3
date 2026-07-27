# Built-in RBAC Menu Display Consistency Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make built-in profile menu privileges an exact UI projection of the identity's effective policy-registry grants.

**Architecture:** `built_in_menu_privileges` will consume the already-resolved `BuiltInIdentityKind`, load that identity's direct grants from `PolicyRegistry`, expand implications with `effective_grants`, and project each implemented grant through its canonical UI binding. Callers will pass identity kinds already known from the authorization snapshot or built-in role construction, avoiding a new role-string mapping.

**Tech Stack:** Rust, `lib-core` authorization registry, Axum web API tests, Cargo test runner, PostgreSQL isolated-test script.

## Global Constraints

- The policy registry is the only source of built-in grants.
- Platform Administrator remains non-operational and receives no new Notice grant.
- Sponsor Administrator effective privileges remain unchanged.
- Custom-role persistence, PDF rows, and reserved Email grants remain unchanged.
- Production changes must follow red-green TDD.

---

### Task 1: Derive built-in menu display from effective registry grants

**Files:**
- Modify: `crates/libs/lib-core/src/authorization/tests.rs`
- Modify: `crates/libs/lib-core/src/authorization/menu_privileges.rs`
- Modify: `crates/services/web-server/src/web/rest/user_rest/handlers.rs`
- Modify: `crates/services/web-server/src/web/rest/permission_profile_rest.rs`
- Test: `crates/services/web-server/tests/authz/rbac_users/permission_profiles_web.rs`

**Interfaces:**
- Consumes: `PolicyRegistry::built_in_identity(BuiltInIdentityKind)`, `PolicyRegistry::effective_grants`, `GrantDefinition::ui_binding`, and `AuthorizationSnapshotW::identity().built_in_kind()`.
- Produces: `pub fn built_in_menu_privileges(kind: BuiltInIdentityKind) -> Vec<AdminMenuPrivilege>`.

- [ ] **Step 1: Write failing core regressions**

Add tests that call `built_in_menu_privileges` for Platform, Sponsor CRO, and Sponsor Company identities. Assert that Platform has exactly one `admin` row with Read/Edit enabled and no `home_notice` row. For every tested identity, reconstruct enabled `(menu_key, field)` pairs from its effective registry grants and assert exact equality with the displayed pairs.

```rust
#[test]
fn built_in_menu_privileges_are_exact_effective_grant_projections() {
    for kind in [
        BuiltInIdentityKind::PlatformAdministrator,
        BuiltInIdentityKind::SponsorCroAdministrator,
        BuiltInIdentityKind::SponsorCompanyAdministrator,
    ] {
        let identity = policy_registry().built_in_identity(kind).unwrap();
        let effective = policy_registry()
            .effective_grants(identity.grants.iter().map(|grant| grant.as_str()))
            .unwrap();
        let displayed = built_in_menu_privileges(kind);
        assert_display_matches_effective_grants(&displayed, &effective);
    }
}

#[test]
fn platform_administrator_does_not_advertise_notice_privileges() {
    let displayed = built_in_menu_privileges(
        BuiltInIdentityKind::PlatformAdministrator,
    );
    assert_eq!(displayed.len(), 1);
    assert_eq!(displayed[0].menu_key, "admin");
    assert!(displayed[0].can_read);
    assert!(displayed[0].can_edit);
}
```

- [ ] **Step 2: Run core tests and verify RED**

Run:

```bash
cargo test -p lib-core authorization::tests::built_in_menu_privileges -- --nocapture
```

Expected: compilation or assertion failure because the old function accepts a role string and advertises `home_notice` for Platform Administrator.

- [ ] **Step 3: Write failing profile API regression**

Add a system-administrator profile request to `permission_profiles_web.rs`. Assert `/api/users/me/profile` returns an `admin` row, no `home_notice` row, contains `settings.update`, and contains neither `notice.read` nor `notice.update` in `eligibleActions`.

```rust
assert!(privileges.iter().any(|row| row["menu_key"] == "admin"));
assert!(privileges.iter().all(|row| row["menu_key"] != "home_notice"));
assert!(actions.iter().any(|action| action == "settings.update"));
assert!(actions.iter().all(|action| action != "notice.read"));
assert!(actions.iter().all(|action| action != "notice.update"));
```

- [ ] **Step 4: Run the web regression and verify RED**

Run:

```bash
scripts/test-isolated-db.sh -p web-server --test authz test_system_admin_profile_menu_privileges_match_eligible_actions -- --exact --nocapture
```

Expected: FAIL because the profile currently includes `home_notice` despite lacking Notice actions.

- [ ] **Step 5: Implement the minimal registry projection**

Change `built_in_menu_privileges` to accept `BuiltInIdentityKind`. Resolve the registry identity, expand its grants using `effective_grants`, ignore reserved grants, and set only the UI bindings present in the resulting effective set.

```rust
pub fn built_in_menu_privileges(
    kind: BuiltInIdentityKind,
) -> Vec<AdminMenuPrivilege> {
    let registry = policy_registry();
    let Some(identity) = registry.built_in_identity(kind) else {
        return Vec::new();
    };
    let effective = registry
        .effective_grants(identity.grants.iter().map(|grant| grant.as_str()))
        .expect("canonical built-in identity grants must be valid");
    // Project effective implemented grants through canonical UI bindings.
}
```

Update current-user profile generation to pass `snapshot.identity().built_in_kind()` when present. Update built-in permission-profile row construction to pass the identity kind already selected by `visible_built_in_roles`.

- [ ] **Step 6: Run focused tests and verify GREEN**

Run:

```bash
cargo test -p lib-core authorization::tests::built_in_menu_privileges -- --nocapture
scripts/test-isolated-db.sh -p web-server --test authz test_system_admin_profile_menu_privileges_match_eligible_actions -- --exact --nocapture
```

Expected: all focused tests pass and the isolated database is dropped.

- [ ] **Step 7: Run authorization regression suites**

Run:

```bash
scripts/test-isolated-db.sh -p lib-core --test rbac_grant_profiles
scripts/test-isolated-db.sh -p web-server --test authz -- --test-threads=1
cargo fmt --check
git diff --check
```

Expected: grant-profile tests and the complete web authorization suite pass; formatting and whitespace checks report no errors.

- [ ] **Step 8: Commit the implementation**

```bash
git add crates/libs/lib-core/src/authorization/tests.rs \
  crates/libs/lib-core/src/authorization/menu_privileges.rs \
  crates/services/web-server/src/web/rest/user_rest/handlers.rs \
  crates/services/web-server/src/web/rest/permission_profile_rest.rs \
  crates/services/web-server/tests/authz/rbac_users/permission_profiles_web.rs
git commit -m "fix: align built-in RBAC menu display"
```
