# Changelog

All notable changes to QRShare are documented in this file.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.5.4] — 2026-07-30

### Added
- `src/util.rs` — shared `format_size` and `html_escape` utilities, eliminating duplicate implementations across modules
- `CHANGELOG.md`, `CONTRIBUTING.md`, `SECURITY.md`, `.editorconfig` — standard community health files
- GitHub issue templates and pull request template
- `rust-version = "1.75"` MSRV field in `Cargo.toml`
- Complete crates.io metadata (`license`, `repository`, `homepage`, `keywords`, `categories`)
- SHA256 checksum download and verification step in `install.sh`
- Binary existence check and asset completeness verification in the release workflow
- `Swatinem/rust-cache` in CI to speed up builds
- `fail-fast: false` in the build matrix so a single target failure does not abort all others

### Changed
- Replaced Google Fonts CDN `@import` in `templates.rs` with the native OS system font stack — the web UI now makes **zero external network requests**
- `aarch64-apple-darwin` CI target now runs on `macos-14` (Apple Silicon runner) for native compilation
- All Linux musl targets now use `cargo-zigbuild` for correct musl cross-linker toolchain (previously used incorrect GNU cross-linkers)
- `get_dir_size` in `main.rs` now runs inside `tokio::task::spawn_blocking` to avoid blocking the async executor during startup
- Added `SAFETY:` documentation to both `unsafe` blocks in `TrackingBody::poll_frame`
- Updated `Cargo.toml` author field to standard display name (removed URL from author entry)

### Fixed
- ZIP feature description in README no longer claims "no disk buffering" (the current implementation writes to a temp file before streaming)
- CI badge in README now correctly references `release.yml`
- Removed nonexistent `-n <count>` flag from CLI reference table in README
- Installer description updated to reflect SHA256 checksum verification

### Removed
- Duplicate `format_size` function from `src/main.rs` and `src/server.rs`
- Duplicate `html_escape` function from `src/server.rs` and `src/error.rs`
- Google Fonts CDN dependency from the embedded web UI

---

## [1.5.3] — 2026-07-29

### Added
- Redesigned terminal startup banner with ANSI shadow font
- Glassmorphism web UI with dark/light theme support
- Safe-area inset support for mobile browser compatibility
- On-the-fly ZIP streaming for directory shares
- Download limit tracking via session cookie and socket address fallback
- Password protection with constant-time comparison
- Media previews: images, video/audio with seek bar, PDF iframe, rendered Markdown
- Path traversal protection with double-canonicalization
- `install.sh` with animated spinner and platform detection

---

*Entries for versions prior to 1.5.3 are not available.*
