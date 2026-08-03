<div align="center">

# QRShare

**Fast · Private · Local**

Share files, folders, and URLs from your terminal to any device on your local network instantly — no accounts, no cloud, and no configuration required.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)
[![GitHub release](https://img.shields.io/github/v/release/bharadwajsanket/QRShare.svg?style=flat-square)](https://github.com/bharadwajsanket/QRShare/releases)
[![GitHub Actions](https://img.shields.io/github/actions/workflow/status/bharadwajsanket/QRShare/release.yml?style=flat-square&label=CI)](https://github.com/bharadwajsanket/QRShare/actions)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-brightgreen?style=flat-square)](#supported-platforms)

</div>

---

```
$ qrshare ./presentation.pdf

  ██████╗  ██████╗  ███████╗██╗  ██╗  █████╗  ██████╗  ███████╗
 ██╔═══██╗ ██╔══██╗ ██╔════╝██║  ██║ ██╔══██╗ ██╔══██╗ ██╔════╝
 ██║   ██║ ██████╔╝ ███████╗███████║ ███████║ ██████╔╝ █████╗  
 ██║ ▄ ██║ ██╔══██╗ ╚════██║██╔══██║ ██╔══██║ ██╔══██╗ ██╔══╝  
 ╚██████╔╝ ██║  ██║ ███████║██║  ██║ ██║  ██║ ██║  ██║ ███████╗
  ╚═══██╔╝  ╚═╝  ╚═╝ ╚══════╝╚═╝  ╚═╝ ╚═╝  ╚═╝ ╚═╝  ╚═╝ ╚══════╝

                    Fast  •  Private  •  Local

  ┌────────────────────────────────────────────────────────┐
  │ File       presentation.pdf                            │
  │ Size       4.2 MB                                      │
  │ Address    http://192.168.1.1:54321                    │
  │ Security   Open Access (LAN Only)                      │
  │ Limit      Once                                        │
  └────────────────────────────────────────────────────────┘

    █▀▀▀▀▀█ ▄▄▀█▀ █▀▀▀▀▀█
    █ ███ █ ▀▄▀▄█ █ ███ █
    █ ▀▀▀ █ █▀ ▀█ █ ▀▀▀ █
    ▀▀▀▀▀▀▀ █▄▀▄█ ▀▀▀▀▀▀▀
    █▀▀▀▀▀█  ▄▄   █▀▀▀▀▀█
    █ ███ █ ▀▄▀▀█ █ ███ █
    █ ▀▀▀ █  ▄▀ ▄ █ ▀▀▀ █
    ▀▀▀▀▀▀▀ ▀▀▀▀▀ ▀▀▀▀▀▀▀

  → Scan the QR code or open: http://192.168.1.1:54321
  → Expires: after 1 download

  📡 Waiting for connections...
```

---

## Features

| | Feature | Description |
|---|---|---|
| ⚡ | **Instant Sharing** | Single command. QR code rendered directly in your terminal. Clients download in seconds. |
| 📁 | **Files, Folders, or URLs** | Share a single file, a whole directory, or redirect clients to a local or remote URL. |
| 🗜️ | **ZIP on Demand** | Directories are packaged on the fly as ZIP archives — fast generation with no pre-compression delay. |
| 🎬 | **Media Previews** | Premium browser experience with inline previews for images, audio/video seek bars, PDF viewports, and markdown. |
| ⏳ | **Expiring Shares** | Auto-shutdown after a time duration (`--expire 10m`) or a download count limit (`--limit 1`, `--once`, `--twice`). |
| 🔒 | **Password Protection** | Cryptographically sound, constant-time validation; prompted securely or passed inline. |
| 🛡️ | **Path Traversal Protection** | Double-canonicalized local resolution blocks symlink escapes and directory traversal. |
| 📦 | **Zero Dependencies** | Single compiled binary. Requires no runtime configurations or external web dependencies. |

---

## Installation

### macOS & Linux (Shell)
Use our universal installer to download the binary, verify its checksum integrity, and configure your shell PATH environment:
```bash
curl -fsSL https://raw.githubusercontent.com/bharadwajsanket/QRShare/main/install.sh | bash
```

### Windows (PowerShell)
Execute the PowerShell installer in your console to install `qrshare` locally under your profile and update your environment path:
```powershell
irm https://raw.githubusercontent.com/bharadwajsanket/QRShare/main/install.ps1 | iex
```

### Cargo
Install directly from crates.io via Cargo:
```bash
cargo install qrshare
```

### Manual Release Binaries
Pre-built binaries for all supported platforms and architecture combinations are available on the [Releases](https://github.com/bharadwajsanket/QRShare/releases) page. Hash checks are provided in `SHA256SUMS`.

---

## Quick Start

### Share a File
```bash
qrshare document.pdf
```

### Share a Directory (Browsable UI, Download as ZIP)
```bash
qrshare ./my_photos/
```

### Share/Redirect a URL
```bash
qrshare https://github.com/bharadwajsanket/QRShare
```

---

## Examples

### Expire Share After One Download
```bash
qrshare secret.zip --limit 1
```

### Password-protect a Share (Secure prompt)
```bash
qrshare report.pdf --password
```

### Bind to a Specific Interface and Port
```bash
qrshare photo.jpg --host 192.168.1.50 --port 9000
```

### Open the Share URL in Default Browser
```bash
qrshare notes.md --open
```

---

## Security

> [!IMPORTANT]
> **Threat Model:** QRShare is designed for trusted local networks. It is not intended to be exposed directly to the public internet.

- **Deduplicated Session IDs** — Ephemeral cookie tokens track range requests to distinguish a single browser preview flow from multiple distinct downloads.
- **Constant-Time Verification** — Password checks are compared using constant-time algorithms to block timing side-channel attacks.
- **Strict Path Resolution** — Files are verified against canonical roots to eliminate directory traversal.
- **Complete Privacy** — QRShare compiles web resources locally. Zero external CDNs or trackers are queried during any network transaction.

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

---

## Building from Source

### Prerequisites
- Stable [Rust toolchain](https://rustup.rs) (MSRV **1.75**)

### Compile
```bash
git clone https://github.com/bharadwajsanket/QRShare
cd QRShare
cargo build --release
# Compiled executable located at: target/release/qrshare
```

### Test Suite
Run the suite of unit and integration/regression tests:
```bash
cargo test
```

---

## CLI Reference

### Security Options
* `-P, --password [value]` — Require a password to authorize access. Prompts securely if no inline value is specified.

### Sharing Options
* `-l, --limit <count>` — Automatically shut down the server after N successful transfers.
* `-e, --expire <duration>` — Automatically shut down after a duration (e.g. `5m`, `2h`, `30s`).
* `--once` — Legacy alias to exit after 1 download.
* `--twice` — Legacy alias to exit after 2 downloads.

### Network Options
* `-H, --host <address>` — Network interface IP to bind to. (Defaults to auto-detected local IP).
* `-p, --port <port>` — Listen on a specific port. (Defaults to a random available port).

### General Options
* `-o, --open` — Open the sharing URL in your default browser on startup.
* `-h, --help` — Print help documentation.
* `-V, --version` — Print version.

---

## License

This project is licensed under the MIT License — see [LICENSE](LICENSE) for details.

---

## Contributing

Contributions that maintain the simplicity and LAN-only philosophy of QRShare are welcome. Please refer to [CONTRIBUTING.md](CONTRIBUTING.md) for more details.