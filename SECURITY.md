# Security Policy

## Supported Versions

| Version | Supported |
|---|---|
| 1.5.x (latest) | ✅ |
| < 1.5 | ❌ |

Only the latest stable release receives security fixes.

---

## Threat Model

QRShare is designed for **trusted local area networks**.

It is explicitly **not** designed to be exposed to the public internet. Its security guarantees assume:

- The LAN is trusted (you control who is on your network)
- File content is not sensitive beyond the duration of the share
- Shares are short-lived (seconds to minutes)

If you need public internet file sharing, use a purpose-built service. QRShare is the wrong tool for that use case.

---

## Reporting a Vulnerability

If you discover a security vulnerability, please **do not open a public GitHub Issue**.

Report vulnerabilities privately by emailing the maintainer directly, or by using [GitHub's private security advisory feature](https://github.com/bharadwajsanket/QRShare/security/advisories/new).

Please include:

- A clear description of the vulnerability
- Steps to reproduce
- The potential impact
- Any suggested mitigations

You will receive a response within **72 hours** acknowledging receipt.

---

## Security Properties

| Property | Implementation |
|---|---|
| Path traversal protection | Double-canonicalization via `std::fs::canonicalize` in `security.rs` |
| Session management | Single-use ephemeral UUID tokens per share instance |
| Password verification | Constant-time XOR accumulator comparison in `session.rs` |
| Cookie security | `HttpOnly`, `SameSite=Lax` |
| No external calls | Web UI uses only native OS font stack — zero CDN or third-party requests |
| No telemetry | Zero outbound connections from the binary |

---

## Known Limitations

- **No HTTPS** — traffic between the server and clients is plaintext HTTP. This is acceptable for trusted LAN use but means traffic is visible to other devices on the network.
- **Password length side-channel** — the current `constant_time_compare` implementation returns early if lengths differ, leaking whether the submitted password has the correct length. For the stated LAN threat model this is low risk.
- **Password in process table** — passing `--password <value>` inline makes the password visible in `ps aux` to other local users. Use `--password` without a value to be prompted securely.
