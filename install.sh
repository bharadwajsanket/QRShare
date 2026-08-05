#!/usr/bin/env bash
set -euo pipefail

# GitHub repository details
REPO="bharadwajsanket/QRShare"

# Define ANSI color codes portably using printf
CYAN=$(printf '\033[36m')
GREEN=$(printf '\033[32m')
YELLOW=$(printf '\033[33m')
RED=$(printf '\033[31m')
BOLD=$(printf '\033[1m')
RESET=$(printf '\033[0m')

# 1. Print Header
banner=(
"  ██████╗  ██████╗  ███████╗██╗  ██╗  █████╗  ██████╗  ███████╗"
" ██╔═══██╗ ██╔══██╗ ██╔════╝██║  ██║ ██╔══██╗ ██╔══██╗ ██╔════╝"
" ██║   ██║ ██████╔╝ ███████╗███████║ ███████║ ██████╔╝ █████╗  "
" ██║ ▄ ██║ ██╔══██╗ ╚════██║██╔══██║ ██╔══██║ ██╔══██╗ ██╔══╝  "
" ╚██████╔╝ ██║  ██║ ███████║██║  ██║ ██║  ██║ ██║  ██║ ███████╗"
"  ╚═══██╔╝  ╚═╝  ╚═╝ ╚══════╝╚═╝  ╚═╝ ╚═╝  ╚═╝ ╚═╝  ╚═╝ ╚══════╝"
)

printf "%s\n" "$CYAN"
for line in "${banner[@]}"; do
    printf "%s\n" "$line"
done
printf "%s\n" "$RESET"
printf "         %s%sQRShare Installer — Production Edition%s\n" "$BOLD" "$CYAN" "$RESET"
printf "         ==========================================\n\n"

# 2. Spinner Helpers
SPIN_PID=""
start_spinner() {
    local msg="$1"
    local frames=(⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏)
    (
        local delay=0.1
        while true; do
            for frame in "${frames[@]}"; do
                printf "\r\033[K%s %s" "${CYAN}${frame}${RESET}" "$msg"
                sleep "$delay"
            done
        done
    ) &
    SPIN_PID=$!
}

stop_spinner() {
    local exit_code=$1
    local success_msg="$2"
    local fail_msg="$3"
    if [ -n "${SPIN_PID:-}" ]; then
        kill "$SPIN_PID" >/dev/null 2>&1 || true
        wait "$SPIN_PID" >/dev/null 2>&1 || true
        SPIN_PID=""
    fi
    if [ "$exit_code" -eq 0 ]; then
        printf "\r\033[K%s %s\n" "${GREEN}✓${RESET}" "$success_msg"
    else
        printf "\r\033[K%s %s\n" "${RED}✗${RESET}" "$fail_msg"
        exit 1
    fi
}

check_cmd() {
    command -v "$1" >/dev/null 2>&1
}

# 3. Verify Prerequisites
start_spinner "Checking installer prerequisites..."
if ! check_cmd curl; then
    stop_spinner 1 "" "Error: 'curl' is required but not installed. Please install curl and try again."
fi
if ! check_cmd tar; then
    stop_spinner 1 "" "Error: 'tar' is required but not installed. Please install tar and try again."
fi
if ! check_cmd sha256sum && ! check_cmd shasum; then
    stop_spinner 1 "" "Error: Neither 'sha256sum' nor 'shasum' is installed. A checksum utility is required."
fi
stop_spinner 0 "Installer prerequisites verified" ""

# 4. Detect Existing Installation
EXISTING_PATH=""
CURRENT_VERSION=""
if check_cmd qrshare; then
    EXISTING_PATH=$(command -v qrshare)
elif [ -f "/usr/local/bin/qrshare" ]; then
    EXISTING_PATH="/usr/local/bin/qrshare"
elif [ -f "$HOME/.local/bin/qrshare" ]; then
    EXISTING_PATH="$HOME/.local/bin/qrshare"
fi

if [ -n "$EXISTING_PATH" ]; then
    CURRENT_VERSION=$("$EXISTING_PATH" --version 2>/dev/null | awk '{print $2}' || echo "unknown")
    printf "%sℹ Existing installation of QRShare (%s) found at: %s%s\n" "$YELLOW" "$CURRENT_VERSION" "$EXISTING_PATH" "$RESET"
fi

# 5. Detect Platform & Architecture
start_spinner "Detecting host operating system and architecture..."
OS_TYPE=""
case "$(uname -s)" in
    Darwin*)  OS_TYPE="macos";;
    Linux*)   OS_TYPE="linux";;
    *)        stop_spinner 1 "" "Error: Unsupported operating system: $(uname -s)";;
esac

ARCH_TYPE=""
case "$(uname -m)" in
    x86_64*)  ARCH_TYPE="x86_64";;
    arm64*|aarch64*) ARCH_TYPE="arm64";;
    armv7*)   ARCH_TYPE="armv7";;
    *)        stop_spinner 1 "" "Error: Unsupported architecture: $(uname -m)";;
esac
stop_spinner 0 "System detected: $OS_TYPE ($ARCH_TYPE)" ""

# 6. Resolve Latest Version Tag
start_spinner "Fetching latest version metadata..."
LATEST_TAG=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' || true)

if [ -z "$LATEST_TAG" ]; then
    LATEST_TAG="v3.5.4"
    stop_spinner 0 "Connection to GitHub API failed. Defaulting to: $LATEST_TAG" ""
else
    stop_spinner 0 "Target version resolved: $LATEST_TAG" ""
fi

# 7. Configure Temp Directory
BINARY_NAME="qrshare-${OS_TYPE}-${ARCH_TYPE}.tar.gz"
DOWNLOAD_URL="https://github.com/$REPO/releases/download/${LATEST_TAG}/${BINARY_NAME}"
CHECKSUMS_URL="https://github.com/$REPO/releases/download/${LATEST_TAG}/SHA256SUMS"

TMP_DIR=$(mktemp -d)
clean_up() {
    if [ -n "${SPIN_PID:-}" ]; then
        kill "$SPIN_PID" >/dev/null 2>&1 || true
        wait "$SPIN_PID" >/dev/null 2>&1 || true
    fi
    rm -rf "$TMP_DIR"
}
trap clean_up EXIT

# 8. Download Release Archive & Checksums
printf "\n%sDownloading release assets...%s\n" "$BOLD" "$RESET"
if ! curl -L --progress-bar "$DOWNLOAD_URL" -o "${TMP_DIR}/${BINARY_NAME}"; then
    printf "%s✗ Error: Failed to download binary archive from %s%s\n" "$RED" "$DOWNLOAD_URL" "$RESET"
    exit 1
fi

start_spinner "Downloading SHA256 checksums..."
if ! curl -fsSL "$CHECKSUMS_URL" -o "${TMP_DIR}/SHA256SUMS"; then
    stop_spinner 1 "" "Error: Failed to download SHA256SUMS from $CHECKSUMS_URL"
fi
stop_spinner 0 "Checksum validation list downloaded" ""

# 9. Verify Checksum Integrity
start_spinner "Verifying archive hash integrity..."
CHECKSUM_OK=0
if check_cmd sha256sum; then
    if (cd "$TMP_DIR" && sha256sum --ignore-missing -c SHA256SUMS >/dev/null 2>&1); then
        CHECKSUM_OK=1
    fi
elif check_cmd shasum; then
    expected=$(grep "${BINARY_NAME}" "${TMP_DIR}/SHA256SUMS" | awk '{print $1}')
    actual=$(shasum -a 256 "${TMP_DIR}/${BINARY_NAME}" | awk '{print $1}')
    if [ "$expected" = "$actual" ] && [ -n "$expected" ]; then
        CHECKSUM_OK=1
    fi
fi

if [ "$CHECKSUM_OK" -ne 1 ]; then
    stop_spinner 1 "" "Error: Checksum verification FAILED. The downloaded archive may be corrupted. Aborting."
fi
stop_spinner 0 "Hash verification passed (SHA256 match)" ""

# 10. Extract Binary
start_spinner "Extracting target release binary..."
if ! tar -xzf "${TMP_DIR}/${BINARY_NAME}" -C "$TMP_DIR"; then
    stop_spinner 1 "" "Error: Failed to extract tar archive."
fi
stop_spinner 0 "Extraction complete" ""

# 11. Select and Validate Destination Path
INSTALL_DIR="/usr/local/bin"
if [ -n "$EXISTING_PATH" ]; then
    INSTALL_DIR=$(dirname "$EXISTING_PATH")
fi

USE_SUDO=""
if [ ! -w "$INSTALL_DIR" ]; then
    if check_cmd sudo && [ -t 0 ]; then
        printf "%sℹ Permission denied for %s. Elevation requested.%s\n" "$YELLOW" "$INSTALL_DIR" "$RESET"
        USE_SUDO="sudo"
    else
        INSTALL_DIR="${HOME}/.local/bin"
        mkdir -p "$INSTALL_DIR"
        printf "%sℹ Using user-local path fallback: %s%s\n" "$YELLOW" "$INSTALL_DIR" "$RESET"
    fi
fi

# 12. Install Executable
start_spinner "Installing executable to ${INSTALL_DIR}/qrshare..."
if ! $USE_SUDO cp "${TMP_DIR}/qrshare" "${INSTALL_DIR}/qrshare"; then
    stop_spinner 1 "" "Error: Failed to copy binary to destination directory."
fi
if ! $USE_SUDO chmod +x "${INSTALL_DIR}/qrshare"; then
    stop_spinner 1 "" "Error: Failed to set executable permissions."
fi
stop_spinner 0 "Binary installed successfully" ""

# 13. Verify Executable Runs
start_spinner "Verifying target execution compatibility..."
if ! "${INSTALL_DIR}/qrshare" --help >/dev/null 2>&1; then
    stop_spinner 1 "" "Error: Binary verification failed. Executable crashed or is incompatible with host architecture."
fi
stop_spinner 0 "Execution validation passed" ""

# 14. Configure Shell PATH if needed
PATH_UPDATED=0
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    start_spinner "Checking shell configuration path environment..."
    SHELL_RC=""
    if [[ "$SHELL" == */zsh ]]; then
        SHELL_RC="$HOME/.zshrc"
    elif [[ "$SHELL" == */bash ]]; then
        SHELL_RC="$HOME/.bashrc"
    else
        SHELL_RC="$HOME/.profile"
    fi

    if [ -f "$SHELL_RC" ] && [ -w "$SHELL_RC" ]; then
        if ! grep -q "export PATH=\"\$PATH:$INSTALL_DIR\"" "$SHELL_RC" && ! grep -q "export PATH=\"$INSTALL_DIR:\$PATH\"" "$SHELL_RC"; then
            printf "\n# Added by QRShare Installer\nexport PATH=\"\$PATH:$INSTALL_DIR\"\n" >> "$SHELL_RC"
            PATH_UPDATED=1
        fi
    fi
    stop_spinner 0 "Shell configuration path verified" ""
fi

# 15. Summary
printf "\n┌────────────────────────────────────────────────────────┐\n"
printf "│ %s%sInstallation Completed Successfully!%s                  │\n" "$BOLD" "$GREEN" "$RESET"
printf "├────────────────────────────────────────────────────────┤\n"
printf "│ Binary Path:  %-40s │\n" "${INSTALL_DIR}/qrshare"
printf "│ Version:      %-40s │\n" "${LATEST_TAG}"
printf "│ Platform:     %-40s │\n" "${OS_TYPE} (${ARCH_TYPE})"
printf "└────────────────────────────────────────────────────────┘\n"

if [ "$PATH_UPDATED" -eq 1 ]; then
    printf "\n%s%s⚠️  Notice:%s Added %s to your PATH in %s\n" "$BOLD" "$YELLOW" "$RESET" "$INSTALL_DIR" "$SHELL_RC"
    printf "Please reload your shell or run: %ssource %s%s to start using 'qrshare'.\n" "$CYAN" "$SHELL_RC" "$RESET"
else
    printf "\nQuick start: Run %sqrshare --help%s to explore usage.\n" "$CYAN" "$RESET"
fi
