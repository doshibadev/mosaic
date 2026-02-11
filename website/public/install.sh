#!/bin/sh
# Mosaic Installer (Linux / macOS)

set -e

# Detect OS
OS="$(uname -s)"
case "$OS" in
    Linux*)     OS_TYPE=linux;;
    Darwin*)    OS_TYPE=macos;;
    *)          echo "Unsupported OS: $OS"; exit 1;;
esac

# Detect Architecture
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64)    ARCH_TYPE=amd64;;
    aarch64)   ARCH_TYPE=amd64;; # We only have amd64 build configured for now
    arm64)     ARCH_TYPE=amd64;; # Rosetta usually handles it on Mac, or we need to add aarch64 build
    *)         echo "Unsupported Architecture: $ARCH"; exit 1;;
esac

ASSET_NAME="mosaic-${OS_TYPE}-${ARCH_TYPE}"
INSTALL_DIR="$HOME/.mosaic/bin"
EXE_NAME="mosaic"

echo "Installing Mosaic for ${OS_TYPE}..."

# Fetch latest release URL from GitHub API
# We use the 'latest' endpoint which redirects to the tag.
# For simplicity in this initial version, we assume standard GitHub release naming.
REPO="doshibadev/mosaic"
DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ASSET_NAME}"

# Create install directory
mkdir -p "$INSTALL_DIR"

# Download
echo "Downloading from ${DOWNLOAD_URL}..."
curl -fsSL "$DOWNLOAD_URL" -o "$INSTALL_DIR/$EXE_NAME"

# Make executable
chmod +x "$INSTALL_DIR/$EXE_NAME"

# Add to PATH if not already present
SHELL_CONFIG=""
case "$SHELL" in
    */zsh) SHELL_CONFIG="$HOME/.zshrc" ;;
    */bash) SHELL_CONFIG="$HOME/.bashrc" ;;
    *) SHELL_CONFIG="$HOME/.profile" ;; # Fallback
esac

if [ -n "$SHELL_CONFIG" ]; then
    if ! grep -q "$INSTALL_DIR" "$SHELL_CONFIG"; then
        echo "" >> "$SHELL_CONFIG"
        echo "# Mosaic Package Manager" >> "$SHELL_CONFIG"
        echo "export PATH="\$PATH:$INSTALL_DIR"" >> "$SHELL_CONFIG"
        echo "Added $INSTALL_DIR to $SHELL_CONFIG"
        echo "Please restart your terminal or run: source $SHELL_CONFIG"
    else
        echo "Path already configured in $SHELL_CONFIG"
    fi
fi

echo ""
echo "Mosaic installed successfully!"
echo "Run 'mosaic --help' to get started."
