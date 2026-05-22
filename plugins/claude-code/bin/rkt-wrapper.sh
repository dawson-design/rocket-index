#!/bin/bash
# RocketIndex wrapper - downloads binary on first use
# This avoids bundling a 30MB binary in the plugin repo

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RKT_BIN="$SCRIPT_DIR/rkt"
VERSION="0.1.0-beta.33"

# Detect platform
case "$(uname -s)" in
    Darwin)
        case "$(uname -m)" in
            arm64) PLATFORM="aarch64-apple-darwin" ;;
            x86_64) PLATFORM="x86_64-apple-darwin" ;;
            *) echo "Unsupported architecture: $(uname -m)" >&2; exit 1 ;;
        esac
        ;;
    Linux)
        case "$(uname -m)" in
            x86_64) PLATFORM="x86_64-unknown-linux-gnu" ;;
            aarch64) PLATFORM="aarch64-unknown-linux-gnu" ;;
            *) echo "Unsupported architecture: $(uname -m)" >&2; exit 1 ;;
        esac
        ;;
    *)
        echo "Unsupported OS: $(uname -s)" >&2
        exit 1
        ;;
esac

# Check if we need to download (missing or wrong version)
NEED_DOWNLOAD=false
if [ ! -x "$RKT_BIN" ]; then
    NEED_DOWNLOAD=true
else
    # Check installed version matches expected version
    INSTALLED_VERSION=$("$RKT_BIN" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?' || true)
    INSTALLED_VERSION="${INSTALLED_VERSION:-unknown}"
    if [ "$INSTALLED_VERSION" != "$VERSION" ]; then
        echo "Updating RocketIndex from $INSTALLED_VERSION to $VERSION..." >&2
        NEED_DOWNLOAD=true
    fi
fi

download_file() {
    local url="$1"
    local output="$2"

    if command -v curl &> /dev/null; then
        curl -fsSL "$url" -o "$output"
    elif command -v wget &> /dev/null; then
        wget -q "$url" -O "$output"
    else
        echo "Error: curl or wget required" >&2
        exit 1
    fi
}

sha256_file() {
    local file="$1"

    if command -v sha256sum &> /dev/null; then
        sha256sum "$file" | awk '{print $1}'
    elif command -v shasum &> /dev/null; then
        shasum -a 256 "$file" | awk '{print $1}'
    else
        echo "Error: sha256sum or shasum required" >&2
        exit 1
    fi
}

if [ "$NEED_DOWNLOAD" = true ]; then
    echo "Downloading RocketIndex $VERSION for $PLATFORM..." >&2

    DOWNLOAD_URL="https://github.com/rocket-tycoon/rocket-index/releases/download/v${VERSION}/rocketindex-v${VERSION}-${PLATFORM}.tar.gz"
    CHECKSUM_URL="${DOWNLOAD_URL}.sha256"

    # Create temp directory
    TMP_DIR=$(mktemp -d)
    trap 'rm -rf "$TMP_DIR"' EXIT

    # Download and verify before extracting
    download_file "$DOWNLOAD_URL" "$TMP_DIR/rkt.tar.gz"
    download_file "$CHECKSUM_URL" "$TMP_DIR/rkt.tar.gz.sha256"

    EXPECTED_SHA256=$(awk '{print $1; exit}' "$TMP_DIR/rkt.tar.gz.sha256")
    ACTUAL_SHA256=$(sha256_file "$TMP_DIR/rkt.tar.gz")
    if [ -z "$EXPECTED_SHA256" ] || [ "$EXPECTED_SHA256" != "$ACTUAL_SHA256" ]; then
        echo "Error: RocketIndex download checksum verification failed" >&2
        exit 1
    fi

    tar -xzf "$TMP_DIR/rkt.tar.gz" -C "$TMP_DIR"

    # Move binary to plugin bin directory
    mv "$TMP_DIR/rkt" "$RKT_BIN"
    chmod +x "$RKT_BIN"

    echo "RocketIndex installed successfully" >&2
fi

# Execute rkt with all arguments
exec "$RKT_BIN" "$@"
