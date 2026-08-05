# Release Guide (QRShare)

This guide documents the manual verification and release flow for QRShare maintainers. All platform asset builds, packaging, checksum generation, and release attachments are automated via GitHub Actions when a release is published.

---

## Step 1: Pre-Release Verification

Always check formatting, static analysis, and execute the test suites locally prior to triggering a release flow:

```bash
# Verify formatting conforms to style guidelines
cargo fmt --check

# Verify code linting
cargo clippy --all-targets --all-features -- -D warnings

# Execute test suite
cargo test

# Build debug binary to verify compilation
cargo build
```

---

## Step 2: Version Updates

Ensure that version identifiers are bumped consistently across all components to refer to the target release version (e.g. `3.5.4`):

- **Cargo.toml**: Update the package version:
  ```toml
  [package]
  version = "3.5.4"
  ```
- **install.sh**: Set the fallback version string:
  ```bash
  LATEST_TAG="v3.5.4"
  ```
- **install.ps1**: Set the fallback version string:
  ```powershell
  $latestTag = "v3.5.4"
  ```
- **CHANGELOG.md**: Add an entry under the version and date headers following the [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format.

---

## Step 3: Triggering automated asset builds

The workflow `.github/workflows/release.yml` triggers automatically **only** when a Release is manually published on GitHub.

1. **Commit and Push changes**:
   ```bash
   git add Cargo.toml install.sh install.ps1 CHANGELOG.md
   git commit -m "chore: prepare release v3.5.4"
   git push origin main
   ```

2. **Create and Push the Release Tag**:
   ```bash
   git tag -a v3.5.4 -m "Release v3.5.4"
   git push origin v3.5.4
   ```

3. **Draft and Publish the Release on GitHub**:
   - Go to the GitHub repository page.
   - Click on **Releases** → **Draft a new release**.
   - Select the tag `v3.5.4`.
   - Write the release title (e.g., `Release v3.5.4`).
   - Paste the release notes from `CHANGELOG.md`.
   - Click **Publish release**.

4. **GitHub Actions Automation**:
   - Once published, GitHub Actions starts the asset builder job.
   - The job compiles all macOS, Linux, and Windows targets, verifies their binary outputs, packages them with `README.md` and `LICENSE`, calculates unified SHA256 checksums, and uploads them directly to the published release automatically.

5. **Publish to Crates.io**:
   ```bash
   cargo publish
   ```
