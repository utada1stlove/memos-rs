Task: improve production deployment support for Debian VPS and Arch Linux.

Requirements:
- provide a production-ready systemd service file
- define a default config location, such as /etc/memos-rs/config.toml
- define a default data directory, such as /var/lib/memos-rs
- run as a dedicated non-root user where practical
- document installation steps for Debian
- document local development steps for Arch Linux

Optional:
- provide an install.sh helper
- provide a packaging/ directory for deployment assets