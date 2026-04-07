# Deployment notes for memos-rs

## Target environments
- Debian VPS
- Arch Linux local machine

## Deployment goal
A single Rust binary with simple config and data directories.

## Expected runtime layout
- binary: /usr/local/bin/memos-rs
- config: /etc/memos-rs/config.toml
- data: /var/lib/memos-rs
- logs: journalctl via systemd preferred

## Service expectations
- runs as dedicated non-root user if practical
- restarts on failure
- binds to configurable host and port
- graceful shutdown on SIGTERM

## Debian expectations
- systemd unit provided in packaging/memos-rs.service
- documented install steps
- SQLite file path should be easy to set

## Arch Linux expectations
- cargo run for development
- release binary runnable directly
- no Debian-specific assumptions in core runtime

## Release expectations
- GitHub Actions should build Linux artifacts
- provide tar.gz archives
- provide sha256 checksums

## Nice-to-have later
- install.sh helper
- sample reverse proxy config for Nginx or Caddy
- musl builds for easier portability