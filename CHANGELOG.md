# Changelog

All notable changes to QRShare are documented in this file.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [3.5.4] — 2026-08-05

### Added
- Plain text sharing support via `--text <TEXT>` option with markdown rendering and syntax highlighting
- Clipboard sharing support via `--clipboard` option to automatically share clipboard text or images
- stdin pipe support (`cat file.md | qrshare`) to easily share text input
- Dedicated landing page for shared URLs (replaces automatic redirect)
- Option to select terminal QR code themes globally via `--theme <THEME>`
- mDNS responder to automatically advertise `http://qrshare.local:<port>` on the local network

### Changed
- Refactored templates to share consistent header, title, timestamp, and brand footer
- Upgraded clipboard copy logic to Clipboard API with highly reliable legacy fallback
- Upgraded share button to use Web Share API with copy URL fallback (no extra dialogs)

## [2.5.4] — 2026-08-03

### Added
- Completely redesigned browser interface — premium minimal aesthetic inspired by Apple, Arc, Linear, and Vercel
- Automatic Light & Dark theme support via `prefers-color-scheme` — zero user interaction required
- Inter typeface loaded via Google Fonts with system font fallback for offline environments
- CSS design token system (`common.css`) — unified color palette, spacing, and animation variables for all pages
- Dedicated `file_icon_svg()` helper in `templates.rs` — per-MIME-category stroke SVG icons (image, video, audio, document, archive, code, generic)
- Shared `head()` function in `templates.rs` — eliminates repeated boilerplate across page templates
- Metadata chips on the file page — type and size rendered as compact inline badges
- Lock icon on the password page — purple accent circle with padlock SVG
- Friendly error titles and per-status icons in `error.rs` — "Share Expired" (403), "Not Found" (404), "Access Denied" (401), "Invalid Request" (400)
- HTTP status code shown as a small muted badge on error pages (no longer the primary heading)
- Subtle glass card effect (`backdrop-filter: blur(20px)`) applied only in dark mode, clean white in light mode
- Redirect page redesigned with CSS-only spinner and monospace URL pill
- `install.ps1` — new Windows PowerShell installer with architecture detection, SHA256 verification, and User PATH configuration

### Changed
- Primary button color changed from neutral black/white to `#7C3AED` (Violet-600) — consistent purple accent across all interactive elements
- Card entry animation tightened to 180ms `cubic-bezier(0.16, 1, 0.3, 1)` spring easing
- Button hover state changed to `translateY(-1px)` lift with accent glow instead of scale-down
- Preview area in the file page now renders edge-to-edge (no padding) for images and video
- Password page copy rewritten: "Protected Share" / "Enter the password to access this file."
- Error page copy rewritten to be receiver-friendly — no raw HTTP codes, no stack traces
- Footer copy updated from attribution to "Shared with QRShare" linking the repository

### Fixed
- `install.sh` fallback version bumped to `v2.5.4`
- `install.ps1` fallback version bumped to `v2.5.4`
- Redundant `&` borrows in `format!` arguments removed (Clippy `useless_borrows_in_formatting`)
- Hex color literals (`#09090B`, `#FAFAFA`) moved out of `r#"..."#` raw strings to avoid Rust 2021 prefix parsing errors

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
