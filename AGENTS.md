# Local UI verification

- Run UI workflows with `scripts/ui-env.sh <name> <backend-port> <frontend-port>` so each task keeps its own database. Never reset or reuse `app_db` for UI verification.
- Reuse the repository `target/`. Do not set `CARGO_TARGET_DIR`, run concurrent Cargo jobs, or use `cargo run`.
- The UI runner never builds. When backend source changed, run exactly `cargo build -p web-server --bin web-server` once, then restart the runner. Frontend changes use the existing dev server hot reload.
- Use `scripts/ui-env.sh reset <name>` only when that named UI database must be deliberately discarded.
