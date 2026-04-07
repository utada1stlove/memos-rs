Task: bootstrap the memos-rs service foundation.

Implement the following now:

1. Create the project structure with at least:
- src/main.rs
- src/cli.rs
- src/config.rs
- src/server.rs
- src/app.rs
- src/error.rs
- src/state.rs
- migrations/
- .github/workflows/ci.yml
- config.example.toml
- README.md
- packaging/memos-rs.service

2. Build a minimal runnable service:
- binary name: memos-rs
- CLI with a `serve` subcommand
- load config from config.toml and environment variables
- start an axum HTTP server
- expose GET /healthz
- return JSON like {"status":"ok"}
- structured logging with tracing
- graceful shutdown support

3. Engineering quality:
- cargo fmt
- cargo clippy --all-targets --all-features -- -D warnings
- cargo test
- reasonable error handling
- clear module boundaries for future auth, user, memo, db modules

4. GitHub Actions:
- trigger on push and pull_request
- run fmt check
- run clippy
- run tests
- run release build

5. Documentation:
- explain local run on Arch Linux
- explain run on Debian VPS
- include example systemd service
- include config example

Output format:
1. Brief implementation plan
2. File tree
3. Full code and configs
4. Exact local test commands
5. What should be implemented next

Do not stop at planning. Start implementing.