# memos-rs

`memos-rs` is a Rust reimplementation of the Memos backend, built as a self-hostable single binary with SQLite first and deployment paths aimed at Debian VPS hosts and Arch Linux development machines.

## Phase 7 scope

The project now includes:

- the service foundation, SQLite, bootstrap, auth, memo CRUD, and static frontend hosting
- tag-driven GitHub release automation for Linux artifacts
- a hardened systemd unit for non-root deployment
- a production-oriented config template for `/etc/memos-rs/config.toml`

## Deployment defaults

Production deployment now assumes:

- binary: `/usr/local/bin/memos-rs`
- config: `/etc/memos-rs/config.toml`
- data directory: `/var/lib/memos-rs`
- SQLite database: `/var/lib/memos-rs/data/memos-rs.db`
- frontend assets, if enabled: `/var/lib/memos-rs/frontend/dist`
- service user: `memos-rs`

When `memos-rs serve` is started without an explicit config path and there is no local `config.toml`, it will also look for `/etc/memos-rs/config.toml`.

## Packaging assets

The repository ships these deployment files:

- [config.example.toml](/home/aerith/archlinux/workshop/github/memos-rs/config.example.toml): development-friendly example
- [config.toml](/home/aerith/archlinux/workshop/github/memos-rs/packaging/config.toml): production-oriented packaging template
- [memos-rs.service](/home/aerith/archlinux/workshop/github/memos-rs/packaging/memos-rs.service): systemd unit

## Arch Linux local development

Use the development-oriented config example:

```bash
cp config.example.toml config.toml
cargo run -- serve --config config.toml
```

Run the local validation set:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release
```

## Debian VPS installation

You can either build locally and copy the binary, or download a tagged release artifact. For a manual install on Debian:

```bash
cargo build --release
sudo install -Dm755 target/release/memos-rs /usr/local/bin/memos-rs
sudo install -Dm644 packaging/config.toml /etc/memos-rs/config.toml
sudo install -Dm644 packaging/memos-rs.service /etc/systemd/system/memos-rs.service
sudo useradd --system --home /var/lib/memos-rs --shell /usr/sbin/nologin memos-rs
sudo mkdir -p /var/lib/memos-rs/data
sudo chown -R memos-rs:memos-rs /var/lib/memos-rs
sudoedit /etc/memos-rs/config.toml
sudo systemctl daemon-reload
sudo systemctl enable --now memos-rs
```

Before the first start, set a strong non-empty `auth.jwt_secret` in `/etc/memos-rs/config.toml`. The packaged template now fails fast until that value is supplied.

Check service health and logs:

```bash
systemctl status memos-rs
sudo journalctl -u memos-rs -f
curl http://127.0.0.1:5230/healthz
```

## systemd service

The packaged systemd unit runs as the dedicated `memos-rs` user and includes a moderate hardening baseline:

- `NoNewPrivileges=true`
- `PrivateTmp=true`
- `ProtectSystem=strict`
- `ProtectHome=true`
- `ProtectControlGroups=true`
- `ProtectKernelModules=true`
- `ProtectKernelTunables=true`
- `LockPersonality=true`
- `MemoryDenyWriteExecute=true`
- `RestrictSUIDSGID=true`
- `RestrictRealtime=true`
- `SystemCallArchitectures=native`

It also uses `StateDirectory=memos-rs`, so systemd can manage `/var/lib/memos-rs` consistently.

## Static frontend hosting

If you have a built frontend, set:

```toml
[frontend]
static_dir = "/var/lib/memos-rs/frontend/dist"
```

The server will then:

- serve `/` from `index.html`
- serve frontend assets from the configured directory
- fall back to `index.html` for non-API frontend routes
- preserve `/api/*` and `/healthz`

## Release automation

Push a tag such as `v0.1.0` to trigger the release workflow:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The release workflow builds:

- `x86_64-unknown-linux-gnu`
- `x86_64-unknown-linux-musl`
- `aarch64-unknown-linux-gnu`

Each GitHub Release artifact includes:

- `memos-rs`
- `README.md`
- `config.example.toml`
- `packaging/config.toml`
- `packaging/memos-rs.service`
- a matching `.sha256` checksum file

## API quick start

Create the first admin:

```bash
curl -X POST http://127.0.0.1:5230/api/v1/bootstrap \
  -H 'Content-Type: application/json' \
  -d '{
    "username": "admin",
    "displayName": "Admin User",
    "email": "admin@example.com",
    "password": "supersecret"
  }'
```

Login:

```bash
curl -X POST http://127.0.0.1:5230/api/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d '{
    "username": "admin",
    "password": "supersecret"
  }'
```

Create a memo:

```bash
curl -X POST http://127.0.0.1:5230/api/v1/memos \
  -H "Authorization: Bearer JWT_HERE" \
  -H 'Content-Type: application/json' \
  -d '{
    "content": "# First memo",
    "visibility": "private"
  }'
```

## Deferred behavior

Some behavior is still intentionally deferred:

- full visibility enforcement for public or unlisted memos
- dedicated pin or archive mutation routes
- installer automation beyond the packaged assets in `packaging/`

## What should be implemented next

The current execution plan is complete after Phase 7.
