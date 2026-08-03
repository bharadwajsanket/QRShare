<div align="center">

# QRShare

**Fast · Private · Local**

Share files, folders, and URLs from your terminal to any device on your local network instantly — no accounts, no cloud, and no configuration required.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)
[![GitHub release](https://img.shields.io/github/v/release/bharadwajsanket/QRShare.svg?style=flat-square)](https://github.com/bharadwajsanket/QRShare/releases)
[![GitHub Actions](https://img.shields.io/github/actions/workflow/status/bharadwajsanket/QRShare/release.yml?style=flat-square&label=CI)](https://github.com/bharadwajsanket/QRShare/actions)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-brightgreen?style=flat-square)](#supported-platforms)
[![Crates.io](https://img.shields.io/crates/v/qrshare.svg?style=flat-square)](https://crates.io/crates/qrshare)

</div>

---

```
$ qrshare ./report.pdf --limit 1

  ██████╗  ██████╗  ███████╗██╗  ██╗  █████╗  ██████╗  ███████╗
 ██╔═══██╗ ██╔══██╗ ██╔════╝██║  ██║ ██╔══██╗ ██╔══██╗ ██╔════╝
 ██║   ██║ ██████╔╝ ███████╗███████║ ███████║ ██████╔╝ █████╗  
 ██║ ▄ ██║ ██╔══██╗ ╚════██║██╔══██║ ██╔══██║ ██╔══██╗ ██╔══╝  
 ╚██████╔╝ ██║  ██║ ███████║██║  ██║ ██║  ██║ ██║  ██║ ███████╗
  ╚═══██╔╝  ╚═╝  ╚═╝ ╚══════╝╚═╝  ╚═╝ ╚═╝  ╚═╝ ╚═╝  ╚═╝ ╚══════╝

                    Fast  •  Private  •  Local

  ┌────────────────────────────────────────────────────────┐
  │ File       report.pdf                                  │
  │ Size       3.1 MB                                      │
  │ Address    http://192.168.1.1:54321                    │
  │ Security   Open Access (LAN Only)                      │
  │ Limit      Once                                        │
  └────────────────────────────────────────────────────────┘

    █▀▀▀▀▀█ ▄▄▀█▀ █▀▀▀▀▀█
    █ ███ █ ▀▄▀▄█ █ ███ █
    █ ▀▀▀ █ █▀ ▀█ █ ▀▀▀ █
    ▀▀▀▀▀▀▀ █▄▀▄█ ▀▀▀▀▀▀▀

  → Scan the QR code or open: http://192.168.1.1:54321
  → Expires: after 1 download

📡 Waiting for connections...
```

---

## What's New in v2.5.4

QRShare v2.5.4 is a major UI/UX release. The Rust backend, server performance, and download logic are completely unchanged. Everything new is in the browser experience delivered to the receiver.

### ✦ Completely Redesigned Browser Interface

The receiver-facing web UI has been rebuilt from scratch. The design is inspired by Apple, Arc Browser, Linear, and Vercel — premium, minimal, and elegant.

| Before (v1.5.4) | After (v2.5.4) |
|---|---|
| Generic dark card | Refined glass card with subtle depth |
| Black/white buttons | Purple (`#7C3AED`) primary accent throughout |
| Plain metadata text | Compact inline type + size chips |
| Generic error codes | Friendly titles with per-status icons |
| System font | Inter with system font fallback |

### ✦ Automatic Light & Dark Theme

The UI automatically follows the device's `prefers-color-scheme` — no toggle, no setting, no cookie. Both themes are first-class:

- **Dark** — `#09090B` background, glass card, purple accent
- **Light** — `#FAFAFA` background, white card, same purple accent

### ✦ Premium Mobile-First Experience

Designed primarily for phone browsers — the device receiving the QR share. Every element uses large touch targets, safe-area insets, and readable spacing. No horizontal scrolling on any screen size.

### ✦ Better Every Page

| Page | Improvement |
|---|---|
| **File** | Edge-to-edge preview · file icon per type · metadata chips · purple download button |
| **Password** | Lock icon · "Protected Share" heading · inline error pill |
| **Folder** | Clean breadcrumbs · accent icon hover · Download ZIP button |
| **Redirect** | CSS spinner · monospace URL pill · "Taking you there…" |
| **Error** | Friendly title per error type · no raw HTTP codes · calm icon |

### ✦ Lightweight Frontend

- Vanilla HTML, CSS, and minimal JavaScript
- No React, Vue, Angular, or any framework
- No Tailwind runtime, no GSAP, no Lottie
- CSS animations only (150–200ms, opacity + transform)
- Everything works on older Android phones

---

## Features

| | Feature | Description |
|---|---|---|
| ⚡ | **Instant Sharing** | Single command. QR code rendered directly in your terminal. Clients download in seconds. |
| 📁 | **Files, Folders, or URLs** | Share a single file, a whole directory, or redirect clients to a local or remote URL. |
| 🗜️ | **ZIP on Demand** | Directories are packaged on the fly as ZIP archives. |
| 🎬 | **Media Previews** | Inline previews for images, audio/video with seek bars, PDF viewports, and rendered Markdown. |
| ⏳ | **Expiring Shares** | Auto-shutdown after a duration (`--expire 10m`) or a download count limit (`--limit 1`, `--once`, `--twice`). |
| 🔒 | **Password Protection** | Constant-time password validation with secure prompt. |
| 🛡️ | **Path Traversal Protection** | Double-canonicalized resolution blocks symlink escapes and directory traversal. |
| 🌗 | **Light & Dark Theme** | Automatically follows `prefers-color-scheme`. Both themes are polished. |
| 📱 | **Mobile-First UI** | Large touch targets, safe-area insets, zero horizontal scroll. |
| 📦 | **Single Binary** | No runtime dependencies, no configuration files, no cloud. |

---

## Installation

### macOS & Linux
```bash
curl -fsSL https://raw.githubusercontent.com/bharadwajsanket/QRShare/main/install.sh | bash
```

### Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/bharadwajsanket/QRShare/main/install.ps1 | iex
```

### Cargo
```bash
cargo install qrshare
```

### Manual
Pre-built binaries with SHA256 checksums are available on the [Releases](https://github.com/bharadwajsanket/QRShare/releases) page.

---

## Quick Start

### Share a File
```bash
qrshare document.pdf
```

### Share a Directory (Browsable, Download as ZIP)
```bash
qrshare ./my_photos/
```

### Share a URL (Redirect)
```bash
qrshare https://example.com
```

### Expire After One Download
```bash
qrshare secret.zip --limit 1
```

### Password-Protected Share
```bash
qrshare report.pdf --password
```

### Specific Interface and Port
```bash
qrshare photo.jpg --host 192.168.1.50 --port 9000
```

### Open Automatically in Browser
```bash
qrshare notes.md --open
```

---

## CLI Reference

```
Usage: qrshare [OPTIONS] <TARGET>

Arguments:
  <TARGET>  File, directory, or URL to share

Security Options:
  -P, --password [<PASSWORD>]  Require a password. Prompts securely if no value given.

Sharing Options:
  -e, --expire <EXPIRE>        Auto-shutdown after duration (e.g. 5m, 2h, 30s)
  -l, --limit <LIMIT>          Auto-shutdown after N successful downloads
      --once                   Alias for --limit 1
      --twice                  Alias for --limit 2

Network Options:
  -p, --port <PORT>            Port to listen on (default: random available port)
  -H, --host <HOST>            Interface IP to bind (default: auto-detected LAN IP)

General Options:
  -o, --open                   Open sharing URL in default browser on startup
  -h, --help                   Print help
  -V, --version                Print version
```

---

## Security

> [!IMPORTANT]
> **Threat Model:** QRShare is designed for trusted local networks. It is not intended to be exposed directly to the public internet.

- **Ephemeral Session Tokens** — Cookie-based session IDs deduplicate range requests so a single browser preview does not count as multiple downloads.
- **Constant-Time Verification** — Password checks use constant-time comparison to block timing side-channel attacks.
- **Strict Path Resolution** — Files are verified against canonical roots to eliminate directory traversal.
- **Zero External Requests** — The receiver's browser makes no CDN, analytics, or tracking calls. All assets are embedded in the binary at compile time.

---

## Supported Platforms

| Platform | Architecture | Release Asset |
|---|---|---|
| **macOS** | Apple Silicon (ARM64) | `qrshare-macos-arm64.tar.gz` |
| **macOS** | Intel (x86\_64) | `qrshare-macos-x86_64.tar.gz` |
| **Linux** | x86\_64 (musl static) | `qrshare-linux-x86_64.tar.gz` |
| **Linux** | ARM64 (musl static) | `qrshare-linux-arm64.tar.gz` |
| **Linux** | ARMv7 (musl static) | `qrshare-linux-armv7.tar.gz` |
| **Windows** | x86\_64 | `qrshare-windows-x86_64.zip` |
| **Windows** | ARM64 | `qrshare-windows-arm64.zip` |

All Linux binaries are statically linked with musl — no `glibc` dependency, runs on any distribution.

---

## Building from Source

**Prerequisites:** Stable [Rust toolchain](https://rustup.rs) (MSRV **1.75**)

```bash
git clone https://github.com/bharadwajsanket/QRShare
cd QRShare
cargo build --release
# Binary: target/release/qrshare
```

**Run tests:**
```bash
cargo test
```

**Check formatting and lints:**
```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

---

## Contributing

Contributions that maintain the simplicity and LAN-only philosophy of QRShare are welcome. Please open an issue before starting a large change so the approach can be discussed first.

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## License

MIT — see [LICENSE](LICENSE).