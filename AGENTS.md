# memos-rs agent instructions

You are working on a Rust reimplementation of the Memos backend.

## Project goal
Build a self-hostable single-binary Rust service that can gradually replace the backend of Memos while keeping deployment simple for Debian VPS and Arch Linux.

## Product constraints
- Do not rewrite the frontend unless explicitly asked.
- Do not attempt full feature parity in one pass.
- Prioritize a stable MVP first.
- SQLite is the first supported database.
- The architecture must remain extensible for PostgreSQL and MySQL later.
- The result should be suitable for GitHub Actions CI and release builds.

## Technical preferences
- Rust stable
- axum
- tokio
- serde
- sqlx
- clap
- tracing
- tower-http
- argon2
- jsonwebtoken or an equivalent standard token solution

## Code quality rules
- Avoid unnecessary abstractions early.
- Avoid unwrap/expect in production paths unless clearly justified.
- Prefer explicit error types.
- Keep modules small and coherent.
- Add tests where practical.
- End each phase in a compiling, runnable state.
- Do not change unrelated files.
- When introducing dependencies, explain why they are needed.
- Prefer small, coherent changes over broad speculative rewrites.

## Repository guidance files
Use these files as task context:
- `codex-action/00-master-plan.md`
- `codex-action/01-overview.md`
- `codex-action/02-extra-rules.md`
- `codex-action/progress.md`
- `codex-action/phase-1-bootstrap-health-ci.md`
- `codex-action/phase-2-sqlite-migration-bootstrap-admin.md`
- `codex-action/phase-3-auth.md`
- `codex-action/phase-4-memo-crud.md`
- `codex-action/phase-5-static-frontend-hosting.md`
- `codex-action/phase-6-github-actions-release.md`
- `codex-action/phase-7-systemd-deployment-polish.md`
- `notes/feature-matrix.md`
- `notes/api-map.md`
- `notes/schema-notes.md`
- `notes/deployment-notes.md`

## Required execution workflow
When starting work in this repository:

1. Read `codex-action/01-overview.md`.
2. Read `codex-action/02-extra-rules.md`.
3. Read `codex-action/00-master-plan.md`.
4. Read `codex-action/progress.md` if it exists.
5. In `codex-action/00-master-plan.md`, find the first unchecked phase.
6. Read that phase file.
7. Execute only the current phase until it is complete.
8. Run validation commands.
9. Update `codex-action/progress.md`.
10. Check off the finished phase in `codex-action/00-master-plan.md`.
11. If validation passes and no blocker exists, continue automatically to the next unchecked phase.
12. Stop only if all phases are complete or a blocker is encountered.

## Phase order
Execute phases in this order:
1. `codex-action/phase-1-bootstrap-health-ci.md`
2. `codex-action/phase-2-sqlite-migration-bootstrap-admin.md`
3. `codex-action/phase-3-auth.md`
4. `codex-action/phase-4-memo-crud.md`
5. `codex-action/phase-5-static-frontend-hosting.md`
6. `codex-action/phase-6-github-actions-release.md`
7. `codex-action/phase-7-systemd-deployment-polish.md`

## Phase completion checklist
A phase is complete only if all of the following are true:
- the requested code, config, and documentation changes are implemented
- the project still builds
- formatting passes
- linting passes if configured
- tests pass if present
- `codex-action/progress.md` is updated
- the corresponding checkbox in `codex-action/00-master-plan.md` is checked

If any item above fails, the phase is not complete.
Continue working on the same phase until it passes, or document a blocker.

## Validation commands
Prefer these commands when applicable:
- `cargo fmt --all --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `cargo build --release`

If a phase introduces other required checks, run those too.

## Progress file format
When updating `codex-action/progress.md`, include:
- current timestamp
- current phase
- completed items
- files changed
- commands run
- validation results
- blockers or open issues
- recommended next phase

## Planning behavior
Before editing, briefly summarize:
- what phase is being executed
- what files will likely change
- how success will be verified

After editing, summarize:
- what changed
- what passed
- whether the phase is complete
- whether continuing to the next phase

## Scope priority
P0:
- project skeleton
- config loading
- logging
- serve CLI
- health endpoint
- graceful shutdown
- SQLite wiring
- migrations
- user bootstrap
- auth
- memo CRUD
- static file hosting

P1:
- tags
- pin/archive
- resource upload
- frontend compatibility improvements

P2:
- multi-user parity
- PostgreSQL
- MySQL
- deeper Memos API parity
- advanced integrations

## Avoid
- giant rewrites without checkpoints
- changing unrelated files
- speculative features not needed for MVP
- adding heavy dependencies without need
- skipping unfinished phases