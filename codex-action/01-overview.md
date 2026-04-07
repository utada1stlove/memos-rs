We are building a new project called memos-rs in the current repository.

Goal:
Reimplement the Memos backend in Rust as a self-hostable single-binary service, optimized for deployment on Debian VPS and Arch Linux, with GitHub Actions builds and releases.

Important context:
- Upstream Memos is already a single Go binary with a React frontend and supports SQLite, MySQL, and PostgreSQL.
- We are NOT trying to rewrite the whole product at once.
- We are building a Rust backend foundation first.
- Keep the frontend out of scope unless needed later.
- SQLite is the first database target.

Technical stack:
- Rust stable
- axum
- tokio
- serde
- sqlx with SQLite first
- clap
- tracing
- tower-http
- argon2
- jsonwebtoken or another standard token approach

Constraints:
- Build incrementally
- Keep architecture clean and production-oriented
- Avoid unnecessary abstraction
- Avoid unwrap/expect in runtime code
- Add tests where practical
- Every milestone must end in a compiling and runnable state

Your behavior:
- First inspect the repository state
- Then propose a short implementation plan
- Then show the file tree to be created or modified
- Then implement
- Then show exact commands to test locally
- Then summarize next steps

Do not aim for full Memos parity yet.
Focus on a stable MVP foundation.