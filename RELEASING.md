# Release Guide (QRShare)

This guide documents the manual release verification, compilation, packaging, and publishing steps for QRShare.

---

## 1. Pre-Release Verification

Always check formatting, static analysis, and test suites prior to triggering a release flow:

```bash
# Verify formatting
cargo fmt --check

# Verify lints
cargo clippy --all-targets --all-features -- -D warnings

# Execute test suite
cargo test
```

---

## 2. Version Updates

Ensure that version identifiers are bumped consistently across all components to refer to the release version (e.g. `1.5.4`):

- **Cargo.toml**: Update the package version:
  ```toml
  [package]
  version = "1.5.4"
  ```
- **install.sh**: Set the fallback version string:
  ```bash
  LATEST_TAG="v1.5.4"
  ```
- **install.ps1**: Set the fallback version string:
  ```powershell
  $latestTag = "v1.5.4"
  ```
- **CHANGELOG.md**: Add an entry under the version and date headers following the [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format.

---

## 3. Compiling Release Assets

Compile statically linked binaries for all target platforms.

### macOS Build
```bash
# Apple Silicon (ARM64)
cargo build --release --target aarch64-apple-darwin
tar -czf qrshare-macos-arm64.tar.gz -C target/aarch64-apple-darwin/release qrshare

# Intel (x86_64)
cargo build --release --target x86_64-apple-darwin
tar -czf qrshare-macos-x86_64.tar.gz -C target/x86_64-apple-darwin/release qrshare
```

### Linux Build (Statically Linked using musl)
Requires cross-compilation toolchains or `cargo-zigbuild`:
```bash
# Linux x86_64
cargo zigbuild --release --target x86_64-unknown-linux-musl
tar -czf qrshare-linux-x86_64.tar.gz -C target/x86_64-unknown-linux-musl/release qrshare

# Linux ARM64
cargo zigbuild --release --target aarch64-unknown-linux-musl
tar -czf qrshare-linux-arm64.tar.gz -C target/aarch64-unknown-linux-musl/release qrshare

# Linux ARMv7
cargo zigbuild --release --target armv7-unknown-linux-musleabihf
tar -czf qrshare-linux-armv7.tar.gz -C target/armv7-unknown-linux-musleabihf/release qrshare
```

### Windows Build
```bash
# Windows x86_64
cargo build --release --target x86_64-pc-windows-msvc
Compress-Archive -Path target/x86_64-pc-windows-msvc/release/qrshare.exe -DestinationPath qrshare-windows-x86_64.zip

# Windows ARM64
cargo build --release --target aarch64-pc-windows-msvc
Compress-Archive -Path target/aarch64-pc-windows-msvc/release/qrshare.exe -DestinationPath qrshare-windows-arm64.zip
```

---

## 4. Generate Checksums

Generate SHA256 checksum verification files for the built release assets:

```bash
# macOS/Linux
sha256sum qrshare-* > SHA256SUMS

# Windows (PowerShell)
Get-FileHash -Path qrshare-* -Algorithm SHA256 | Format-Table Hash, Path
```

---

## 5. Publishing

### Create GitHub Release
1. Tag the release commit:
   ```bash
   git tag -a v1.5.4 -m "Release v1.5.4"
   git push origin v1.5.4
   ```
2. Create a GitHub Release in the repository corresponding to the tag.
3. Upload all `.tar.gz`, `.zip` archives, and the `SHA256SUMS` checksum verification file.
4. Copy description notes from `CHANGELOG.md`.

### Publish to Crates.io
Verify credentials and publish to crates.io:
```bash
cargo publish
```
