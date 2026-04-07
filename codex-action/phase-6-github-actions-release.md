Task: add release automation for memos-rs.

Requirements:
- add a release GitHub Actions workflow
- trigger on version tags like v*
- build release binaries for:
  - x86_64-unknown-linux-gnu
  - x86_64-unknown-linux-musl
  - aarch64-unknown-linux-gnu
- package artifacts as tar.gz
- generate sha256 checksums
- upload artifacts to GitHub Release

Optional:
- use cargo-zigbuild if it simplifies cross compilation
- keep CI readable and maintainable

Documentation:
- describe how to cut a release tag
- explain where artifacts appear
- document any cross-compilation caveats