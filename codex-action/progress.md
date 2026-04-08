# memos-rs progress

## Project status
- Current status: completed
- Overall progress: 7 / 7 phases completed
- Current phase: none
- Last updated: 2026-04-08 17:17:15 +0800

## Phase summary
- [x] Phase 1: Bootstrap, health check, and CI
- [x] Phase 2: SQLite, migrations, and bootstrap admin
- [x] Phase 3: Authentication
- [x] Phase 4: Memo CRUD
- [x] Phase 5: Static frontend hosting
- [x] Phase 6: GitHub Actions release automation
- [x] Phase 7: systemd and deployment polish

## Current blocker
- None

## Follow-up items
- Run a disposable `v*` tag rehearsal on GitHub before the first public release. The workflow shape and local package path are verified, but the hosted `cargo zigbuild` matrix and artifact publication path still need one real CI execution.

## Latest execution summary
- Review-and-hardening completed against the seven-phase plan, with live verification of the run path, auth path, CRUD path, and local release packaging path.
- Fixed API namespace leakage into the SPA fallback, made bearer auth parsing scheme-case-insensitive, made bootstrap-first-user creation atomic, downgraded a concurrent memo update/delete reload race from `500` to `404`, and hardened the packaged production config to fail fast until `auth.jwt_secret` is set.

## Latest files changed
- `src/app.rs`
- `src/auth.rs`
- `src/bootstrap.rs`
- `src/memo/mod.rs`
- `README.md`
- `packaging/config.toml`
- `codex-action/progress.md`

## Latest commands run
- `cargo fmt --all --check`
- `cargo fmt`
- `cargo fmt --all --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `cargo build --release`
- `./target/release/memos-rs serve --config packaging/config.toml`
- `cargo build --release --locked --target x86_64-unknown-linux-gnu`
- local `curl` smoke probes for `/healthz`, `/api/v1/bootstrap`, `/api/v1/auth/login`, `/api/v1/auth/me`, `/api/v1/memos`, `/`, `/notes/123`, `/app.js`, and `/api/not-a-route`

## Latest validation results
- `cargo fmt --all --check`: passed
- `cargo clippy --all-targets --all-features -- -D warnings`: passed
- `cargo test`: passed, 15 tests green
- `cargo build --release`: passed
- `./target/release/memos-rs serve --config packaging/config.toml`: failed fast as expected with `auth.jwt_secret must not be empty`
- `cargo build --release --locked --target x86_64-unknown-linux-gnu`: passed
- local run/auth/CRUD/static smoke path: passed after fixes
- local release packaging rehearsal: passed, tarball and sha256 produced with the expected files

## Recommended next step
- Use a disposable Git tag in GitHub to rehearse the hosted release workflow once before depending on public release artifacts.

---

## Execution log

### Entry template
Use this structure for each update:

#### [TIMESTAMP]
- Phase:
- Objective:
- Completed items:
- Files changed:
- Commands run:
- Validation results:
- Blockers:
- Open issues:
- Recommended next phase:

---

### Initial entry

#### [INITIAL]
- Phase: none
- Objective: initialize the execution workflow
- Completed items:
  - created planning and tracking structure
- Files changed:
  - `AGENTS.md`
  - `codex-action/00-master-plan.md`
  - `codex-action/progress.md`
- Commands run:
  - none
- Validation results:
  - not run
- Blockers:
  - none
- Open issues:
  - implementation not started
- Recommended next phase:
  - `codex-action/phase-1-bootstrap-health-ci.md`

#### [2026-04-08 15:37:36 +0800]
- Phase: `codex-action/phase-1-bootstrap-health-ci.md`
- Objective: bootstrap the Rust service foundation with health check, config loading, logging, docs, CI, and deployment scaffolding
- Completed items:
  - created a new `memos-rs` Rust crate manifest with the phase-1 dependency set
  - implemented the `serve` CLI entrypoint, config loading, structured tracing setup, shared app state, and graceful Axum server startup
  - added `GET /healthz` with JSON response and a reserved `/api/v1` router namespace for later phases
  - added unit tests for config loading and the health endpoint
  - replaced the placeholder README with Arch Linux and Debian VPS run documentation, config guidance, test commands, and next-phase notes
  - added `config.example.toml`, `packaging/memos-rs.service`, `migrations/README.md`, and `.github/workflows/ci.yml`
- Files changed:
  - `Cargo.toml`
  - `src/main.rs`
  - `src/cli.rs`
  - `src/config.rs`
  - `src/server.rs`
  - `src/app.rs`
  - `src/error.rs`
  - `src/state.rs`
  - `README.md`
  - `config.example.toml`
  - `migrations/README.md`
  - `packaging/memos-rs.service`
  - `.github/workflows/ci.yml`
- Commands run:
  - `cargo fmt --all --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`
  - `cargo build --release`
- Validation results:
  - all four required validation commands failed immediately because `cargo` is not installed in the current shell
- Blockers:
  - Rust stable tooling is unavailable locally, so the phase cannot be verified or marked complete
- Open issues:
  - `Cargo.lock` has not been generated because Cargo could not run
  - the new code remains unverified until the Rust toolchain is available
- Recommended next phase:
  - stay on `codex-action/phase-1-bootstrap-health-ci.md` until the Rust toolchain is installed and validation passes

#### [2026-04-08 15:55:09 +0800]
- Phase: `codex-action/phase-1-bootstrap-health-ci.md`
- Objective: rerun Phase 1 validation with the available Rust stable toolchain and only advance if all required checks pass
- Completed items:
  - reran the full Phase 1 validation command set in the project root
  - applied `cargo fmt` to resolve formatting drift detected by the first fmt check
  - generated `Cargo.lock` during dependency resolution
  - confirmed Phase 1 is complete and advanced the plan to Phase 2
- Files changed:
  - `Cargo.lock`
  - `src/config.rs`
  - `src/error.rs`
  - `src/server.rs`
  - `codex-action/00-master-plan.md`
  - `codex-action/progress.md`
- Commands run:
  - `cargo fmt --all --check`
  - `cargo fmt`
  - `cargo fmt --all --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`
  - `cargo build --release`
- Validation results:
  - `cargo fmt --all --check`: passed after applying standard formatting
  - `cargo clippy --all-targets --all-features -- -D warnings`: passed
  - `cargo test`: passed, 4 tests green
  - `cargo build --release`: passed
- Blockers:
  - none
- Open issues:
  - Phase 2 SQLite/bootstrap work has not started yet
- Recommended next phase:
  - `codex-action/phase-2-sqlite-migration-bootstrap-admin.md`

#### [2026-04-08 16:06:34 +0800]
- Phase: `codex-action/phase-2-sqlite-migration-bootstrap-admin.md`
- Objective: add SQLite persistence, migrations, and a first-run bootstrap-admin flow without starting later auth work early
- Completed items:
  - added `sqlx` SQLite startup, parent-directory creation, automatic migrations, and a shared database pool in application state
  - added the initial reversible schema migration for `users` and `memos` with timestamps and basic indexes
  - implemented `POST /api/v1/bootstrap` to create the first admin user only when no users exist
  - stored bootstrap passwords with Argon2 hashes instead of plaintext
  - extended config with `database.kind`, `database.url`, and `database.max_connections` plus `MEMOS_RS_DATABASE_*` overrides
  - added tests for database initialization, bootstrap success, and bootstrap conflict after the first user exists
  - updated README and config examples with schema notes, bootstrap usage, and migration commands
- Files changed:
  - `.gitignore`
  - `Cargo.toml`
  - `Cargo.lock`
  - `src/main.rs`
  - `src/bootstrap.rs`
  - `src/config.rs`
  - `src/db.rs`
  - `src/app.rs`
  - `src/error.rs`
  - `src/state.rs`
  - `README.md`
  - `config.example.toml`
  - `migrations/README.md`
  - `migrations/20260408160000_initial_schema.up.sql`
  - `migrations/20260408160000_initial_schema.down.sql`
  - `codex-action/00-master-plan.md`
  - `codex-action/progress.md`
- Commands run:
  - `cargo fmt --all --check`
  - `cargo fmt`
  - `cargo fmt --all --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`
  - `cargo build --release`
- Validation results:
  - `cargo fmt --all --check`: passed after applying standard formatting
  - `cargo clippy --all-targets --all-features -- -D warnings`: passed
  - `cargo test`: passed, 8 tests green
  - `cargo build --release`: passed
- Blockers:
  - none
- Open issues:
  - authentication has not been implemented yet
- Recommended next phase:
  - `codex-action/phase-3-auth.md`

#### [2026-04-08 16:13:16 +0800]
- Phase: `codex-action/phase-3-auth.md`
- Objective: add login, JWT issuance, and bearer-token auth middleware while keeping bootstrap setup intact
- Completed items:
  - added `POST /api/v1/auth/login` with Argon2 password verification against the SQLite-backed user store
  - added stateless JWT issuance with configurable secret, issuer, and token TTL
  - added bearer-token auth middleware and a protected `GET /api/v1/auth/me` endpoint
  - returned clear `401 Unauthorized` JSON errors for invalid credentials and missing or invalid tokens
  - documented the auth flow, login curl usage, protected-route curl usage, and the current stateless logout tradeoff
  - added tests for successful login, invalid login, and protected-route access without a token
- Files changed:
  - `Cargo.toml`
  - `Cargo.lock`
  - `src/main.rs`
  - `src/auth.rs`
  - `src/config.rs`
  - `src/app.rs`
  - `src/error.rs`
  - `README.md`
  - `config.example.toml`
  - `codex-action/00-master-plan.md`
  - `codex-action/progress.md`
- Commands run:
  - `cargo fmt --all --check`
  - `cargo fmt`
  - `cargo fmt --all --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`
  - `cargo build --release`
- Validation results:
  - `cargo fmt --all --check`: passed after applying standard formatting
  - `cargo clippy --all-targets --all-features -- -D warnings`: passed
  - `cargo test`: passed, 11 tests green
  - `cargo build --release`: passed
- Blockers:
  - none
- Open issues:
  - memo CRUD has not been implemented yet
- Recommended next phase:
  - `codex-action/phase-4-memo-crud.md`

#### [2026-04-08 16:21:18 +0800]
- Phase: `codex-action/phase-4-memo-crud.md`
- Objective: implement the core memo CRUD APIs on top of the authenticated SQLite-backed service foundation
- Completed items:
  - added a dedicated `src/memo` module with create, list, get, update, and delete handlers backed by `sqlx`
  - added authenticated memo routes under `/api/v1/memos`
  - added creator filtering and created-time ordering for memo lists
  - returned sensible JSON memo payloads with visibility, pinned, archived, and timestamp fields
  - enforced owner-scoped memo access for normal users while allowing admin access across creators
  - documented curl examples for all memo CRUD routes and the currently deferred visibility and pin/archive behavior
  - added tests covering memo CRUD and list filtering/order behavior
- Files changed:
  - `src/main.rs`
  - `src/auth.rs`
  - `src/app.rs`
  - `src/error.rs`
  - `src/memo/mod.rs`
  - `README.md`
  - `codex-action/00-master-plan.md`
  - `codex-action/progress.md`
- Commands run:
  - `cargo fmt --all --check`
  - `cargo fmt`
  - `cargo fmt --all --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`
  - `cargo build --release`
- Validation results:
  - `cargo fmt --all --check`: passed after applying standard formatting
  - `cargo clippy --all-targets --all-features -- -D warnings`: passed
  - `cargo test`: passed, 13 tests green
  - `cargo build --release`: passed
- Blockers:
  - none
- Open issues:
  - static frontend hosting has not been implemented yet
- Recommended next phase:
  - `codex-action/phase-5-static-frontend-hosting.md`

#### [2026-04-08 16:28:52 +0800]
- Phase: `codex-action/phase-5-static-frontend-hosting.md`
- Objective: allow the backend to host a prebuilt frontend directory without breaking API or health routes
- Completed items:
  - added optional `frontend.static_dir` configuration and environment override support
  - added a static-file fallback service for frontend assets and SPA-style index fallback
  - preserved `/api/v1/*` routing and `GET /healthz` while serving frontend files at the root
  - documented the expected frontend build output and how to run the backend with static hosting enabled
  - added a test covering root index serving, asset serving, SPA fallback, and preserved API routing
- Files changed:
  - `Cargo.toml`
  - `src/main.rs`
  - `src/config.rs`
  - `src/frontend.rs`
  - `src/app.rs`
  - `README.md`
  - `config.example.toml`
  - `codex-action/00-master-plan.md`
  - `codex-action/progress.md`
- Commands run:
  - `cargo fmt --all --check`
  - `cargo fmt`
  - `cargo fmt --all --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`
  - `cargo build --release`
- Validation results:
  - `cargo fmt --all --check`: passed after applying standard formatting
  - `cargo clippy --all-targets --all-features -- -D warnings`: passed
  - `cargo test`: passed, 14 tests green
  - `cargo build --release`: passed
- Blockers:
  - none
- Open issues:
  - GitHub release automation has not been implemented yet
- Recommended next phase:
  - `codex-action/phase-6-github-actions-release.md`

#### [2026-04-08 16:31:02 +0800]
- Phase: `codex-action/phase-6-github-actions-release.md`
- Objective: add tagged GitHub release automation for Linux build artifacts and checksums
- Completed items:
  - added a tag-triggered release workflow at `.github/workflows/release.yml`
  - built release artifacts for `x86_64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, and `aarch64-unknown-linux-gnu`
  - packaged each target as a `.tar.gz` archive with the binary and deployment assets
  - generated matching `.sha256` checksum files
  - configured automatic upload of packaged artifacts to the GitHub Release for the tag
  - documented how to cut a release tag, where artifacts appear, and the `cargo-zigbuild` cross-compilation caveat
- Files changed:
  - `.github/workflows/release.yml`
  - `README.md`
  - `codex-action/00-master-plan.md`
  - `codex-action/progress.md`
- Commands run:
  - `cargo fmt --all --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`
  - `cargo build --release`
- Validation results:
  - `cargo fmt --all --check`: passed
  - `cargo clippy --all-targets --all-features -- -D warnings`: passed
  - `cargo test`: passed, 14 tests green
  - `cargo build --release`: passed
- Blockers:
  - none
- Open issues:
  - final deployment polish and production systemd hardening have not been completed yet
- Recommended next phase:
  - `codex-action/phase-7-systemd-deployment-polish.md`

#### [2026-04-08 16:34:34 +0800]
- Phase: `codex-action/phase-7-systemd-deployment-polish.md`
- Objective: polish production deployment for Debian VPS and Arch Linux with clearer config defaults and stronger systemd assets
- Completed items:
  - taught the default config loader to fall back to `/etc/memos-rs/config.toml` when no local `config.toml` is present
  - added `packaging/config.toml` with production-oriented defaults for `/var/lib/memos-rs`
  - hardened `packaging/memos-rs.service` with a dedicated state directory and additional systemd sandboxing settings
  - updated release packaging so tagged artifacts include the production config template
  - refreshed the README with explicit Debian installation steps, Arch development steps, deployment defaults, and systemd expectations
- Files changed:
  - `src/config.rs`
  - `packaging/config.toml`
  - `packaging/memos-rs.service`
  - `.github/workflows/release.yml`
  - `README.md`
  - `codex-action/00-master-plan.md`
  - `codex-action/progress.md`
- Commands run:
  - `cargo fmt --all --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`
  - `cargo build --release`
- Validation results:
  - `cargo fmt --all --check`: passed
  - `cargo clippy --all-targets --all-features -- -D warnings`: passed
  - `cargo test`: passed, 14 tests green
  - `cargo build --release`: passed
- Blockers:
  - none
- Open issues:
  - none within the current seven-phase plan
- Recommended next phase:
  - none; current execution plan complete

#### [2026-04-08 17:17:15 +0800]
- Phase: review-and-hardening
- Objective: audit the completed codebase against the seven-phase plan, verify the run/auth/CRUD/release paths, and leave only concrete follow-up items
- Completed items:
  - audited the implementation against the original seven-phase plan and confirmed the scoped feature set is present
  - reran the full Rust validation set and restored a passing baseline after the review-driven fixes
  - live-verified the run path (`/healthz`), bootstrap path, auth path, CRUD path, SPA/static path, and local release packaging path
  - fixed the SPA fallback so unknown `/api/*` routes now return `404` instead of serving `index.html`
  - fixed bearer auth parsing to accept case-insensitive schemes like `Authorization: bearer ...`
  - made first-user bootstrap atomic so concurrent first-run requests cannot create multiple initial users
  - changed the memo update path to return `404` instead of `500` if the memo disappears before the reload step completes
  - hardened the packaged production config so it fails fast until a real `auth.jwt_secret` is supplied
- Files changed:
  - `src/app.rs`
  - `src/auth.rs`
  - `src/bootstrap.rs`
  - `src/memo/mod.rs`
  - `README.md`
  - `packaging/config.toml`
  - `codex-action/progress.md`
- Commands run:
  - `cargo fmt --all --check`
  - `cargo fmt`
  - `cargo fmt --all --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`
  - `cargo build --release`
  - `./target/release/memos-rs serve --config packaging/config.toml`
  - `cargo build --release --locked --target x86_64-unknown-linux-gnu`
  - localhost `curl` probes for health, bootstrap, login, bearer auth, memo CRUD, static assets, SPA fallback, and unknown `/api/*` routes
- Validation results:
  - `cargo fmt --all --check`: passed after applying standard formatting
  - `cargo clippy --all-targets --all-features -- -D warnings`: passed
  - `cargo test`: passed, 15 tests green
  - `cargo build --release`: passed
  - packaged production config: rejected an empty `auth.jwt_secret` as intended
  - local hosted-release surrogate: `cargo build --release --locked --target x86_64-unknown-linux-gnu` plus archive packaging and checksum generation passed
  - live smoke path: passed after the router/auth/bootstrap hardening fixes
- Blockers:
  - none
- Open issues:
  - the real GitHub-hosted release workflow still needs a disposable tag rehearsal because that environment cannot be executed end to end from this local shell
- Recommended next phase:
  - none; current execution plan remains complete
