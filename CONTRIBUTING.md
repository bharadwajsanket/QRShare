# Contributing to QRShare

Thank you for your interest in contributing. QRShare is intentionally small and focused.

Before starting any significant work, **please open an issue first** to discuss the approach. This prevents wasted effort and keeps the project aligned with its philosophy.

---

## Philosophy

QRShare is:

- **Fast** — startup is instant; transfers do not add latency
- **Private** — no telemetry, no cloud, no external network calls
- **Local** — LAN-only by design
- **Zero-config** — one command, one QR code, done
- **Single binary** — no runtime dependencies

Contributions that conflict with these goals will not be merged.

---

## Prerequisites

- Rust stable toolchain (`rustup` — https://rustup.rs)
- `cargo` in your `PATH`

Minimum supported Rust version: **1.75**

---

## Building from Source

```bash
git clone https://github.com/bharadwajsanket/QRShare
cd QRShare
cargo build
```

Release build:
```bash
cargo build --release
```

---

## Running Tests

```bash
cargo test
```

All tests must pass before submitting a PR.

---

## Code Style

```bash
# Format
cargo fmt

# Lint (must be warning-free)
cargo clippy --all-targets -- -D warnings
```

Run both before every commit. PRs that fail `clippy -D warnings` will not be merged.

---

## Project Structure

```
src/
  main.rs       — entry point, CLI parsing, startup banner
  cli.rs        — clap argument definitions
  server.rs     — Axum route handlers and server startup
  templates.rs  — embedded HTML/CSS/JS templates
  security.rs   — path traversal protection
  session.rs    — password authentication and session tokens
  network.rs    — IP detection and port allocation
  qr.rs         — terminal QR code rendering
  zip.rs        — directory ZIP generation and streaming
  error.rs      — AppError type and HTML error responses
  util.rs       — shared utility functions (format_size, html_escape)
```

---

## Submitting a Pull Request

1. Fork the repository
2. Create a feature branch: `git checkout -b feat/my-change`
3. Make changes
4. Run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`
5. Commit with a clear, single-sentence message
6. Open a PR referencing the related issue

---

## Unsafe Code

Any `unsafe` block must include a `// SAFETY:` comment that precisely explains the invariants that make it sound. PRs adding undocumented `unsafe` will not be merged.

---

## Questions

Open a GitHub Discussion for general questions. Use Issues only for confirmed bugs and concrete feature proposals.
