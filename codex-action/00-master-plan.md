# memos-rs execution plan

## How to work
1. Read `codex-action/01-overview.md`.
2. Read `codex-action/02-extra-rules.md`.
3. Read this file.
4. If `codex-action/progress.md` exists, read it first.
5. Find the first unchecked phase in the phase checklist below.
6. Read that phase file.
7. Execute only the current phase until it is complete.
8. Run the validation commands for that phase.
9. Update `codex-action/progress.md`.
10. Check off the completed phase in this file.
11. If validation passes and no blocker exists, continue automatically to the next unchecked phase.
12. Stop only if:
   - all phases are complete, or
   - a blocker is encountered, or
   - a required clarification cannot be resolved from repository context.

## Phase checklist
- [x] Phase 1: Bootstrap, health check, and CI  
  File: `codex-action/phase-1-bootstrap-health-ci.md`

- [x] Phase 2: SQLite, migrations, and bootstrap admin  
  File: `codex-action/phase-2-sqlite-migration-bootstrap-admin.md`

- [x] Phase 3: Authentication  
  File: `codex-action/phase-3-auth.md`

- [x] Phase 4: Memo CRUD  
  File: `codex-action/phase-4-memo-crud.md`

- [x] Phase 5: Static frontend hosting  
  File: `codex-action/phase-5-static-frontend-hosting.md`

- [x] Phase 6: GitHub Actions release automation  
  File: `codex-action/phase-6-github-actions-release.md`

- [x] Phase 7: systemd and deployment polish  
  File: `codex-action/phase-7-systemd-deployment-polish.md`

## Phase execution rule
Work on only one phase at a time.

Do not start a later phase until the current phase is fully complete, unless:
- the current phase file explicitly marks some work as optional, or
- a blocker is documented in `codex-action/progress.md`.

## Definition of done for each phase
A phase is complete only if all of the following are true:
- the requested code, config, and documentation changes are implemented
- the project still builds
- formatting passes
- linting passes if configured
- tests pass if present
- `codex-action/progress.md` is updated
- the phase checkbox in this file is checked

If any of these conditions fail, the phase is not complete.

## Validation commands
Use these commands when applicable:
- `cargo fmt --all --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `cargo build --release`

If a phase requires additional checks, run those as well.

## Progress update requirements
After each phase attempt, update `codex-action/progress.md` with:
- timestamp
- current phase
- completed items
- files changed
- commands run
- validation results
- blockers or open issues
- recommended next step

## Start-of-phase output
Before making changes, briefly state:
- the current phase
- the files likely to change
- how success will be verified

## End-of-phase output
After making changes, briefly state:
- what changed
- what commands were run
- what passed or failed
- whether the phase is complete
- whether work will continue to the next phase

## Failure handling
If blocked:
1. stop advancing to later phases
2. document the blocker clearly in `codex-action/progress.md`
3. describe what was attempted
4. describe what remains unresolved

## Completion rule
When all phase checkboxes are checked:
- update `codex-action/progress.md` to mark the project as fully completed for the current plan
- provide a final summary of completed work
- stop
