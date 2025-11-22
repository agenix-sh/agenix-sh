#!/bin/bash
# AGEniX Installation Script
# Usage: curl -fsSL https://agenix.sh/install.sh | bash
#    or: curl -fsSL https://agenix.sh/install.sh | bash -s -- --dir /custom/path

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
VERSION="${AGENIX_VERSION:-0.1.0}"
INSTALL_DIR="${AGENIX_INSTALL_DIR:-$HOME/.local/bin}"
BASE_URL="https://github.com/agenix-sh"
COMPONENTS=("agx" "agq" "agw")

# Banner
echo ""
echo -e "${BLUE}╔═══════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║         AGEniX Installation Script        ║${NC}"
echo -e "${BLUE}║      Agentic Execution Framework          ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════╝${NC}"
echo ""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        --version)
            VERSION="$2"
            shift 2
            ;;
        --help)
            echo "AGEniX Installation Script"
            echo ""
            echo "Usage:"
            echo "  curl -fsSL https://agenix.sh/install.sh | bash"
            echo "  curl -fsSL https://agenix.sh/install.sh | bash -s -- [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --dir <path>       Install directory (default: ~/.local/bin)"
            echo "  --version <ver>    Specific version to install (default: 0.1.0)"
            echo "  --help             Show this help message"
            echo ""
            echo "Environment Variables:"
            echo "  AGENIX_INSTALL_DIR  Override default install directory"
            echo "  AGENIX_VERSION      Override default version"
            echo ""
            echo "Examples:"
            echo "  # Install to default location"
            echo "  curl -fsSL https://agenix.sh/install.sh | bash"
            echo ""
            echo "  # Install to custom directory"
            echo "  curl -fsSL https://agenix.sh/install.sh | bash -s -- --dir /usr/local/bin"
            echo ""
            echo "  # Install specific version"
            echo "  curl -fsSL https://agenix.sh/install.sh | bash -s -- --version 0.2.0"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Detect platform and architecture
detect_platform() {
    local os arch

    # Detect OS
    case "$(uname -s)" in
        Darwin*)
            os="apple-darwin"
            ;;
        Linux*)
            os="unknown-linux-gnu"
            ;;
        *)
            echo -e "${RED}Error: Unsupported operating system: $(uname -s)${NC}"
            echo "Supported: macOS (Darwin), Linux"
            exit 1
            ;;
    esac

    # Detect architecture
    case "$(uname -m)" in
        x86_64)
            arch="x86_64"
            ;;
        arm64|aarch64)
            arch="aarch64"
            ;;
        *)
            echo -e "${RED}Error: Unsupported architecture: $(uname -m)${NC}"
            echo "Supported: x86_64, arm64/aarch64"
            exit 1
            ;;
    esac

    echo "${arch}-${os}"
}

# Download and verify binary
download_binary() {
    local component=$1
    local platform=$2
    local version=$3
    local url="${BASE_URL}/${component}/releases/download/v${version}/${component}-${version}-${platform}"
    local dest="${INSTALL_DIR}/${component}"
    local temp_file=$(mktemp)

    echo -e "${BLUE}Downloading ${component} v${version} for ${platform}...${NC}"

    # Download with progress
    if command -v curl >/dev/null 2>&1; then
        if ! curl -fL --progress-bar "${url}" -o "${temp_file}"; then
            echo -e "${RED}Error: Failed to download ${component}${NC}"
            echo "URL: ${url}"
            rm -f "${temp_file}"
            return 1
        fi
    elif command -v wget >/dev/null 2>&1; then
        if ! wget -q --show-progress "${url}" -O "${temp_file}"; then
            echo -e "${RED}Error: Failed to download ${component}${NC}"
            echo "URL: ${url}"
            rm -f "${temp_file}"
            return 1
        fi
    else
        echo -e "${RED}Error: Neither curl nor wget found. Please install one.${NC}"
        exit 1
    fi

    # Verify it's a valid executable
    if ! file "${temp_file}" | grep -q "executable"; then
        echo -e "${RED}Error: Downloaded file is not a valid executable${NC}"
        rm -f "${temp_file}"
        return 1
    fi

    # Make executable
    chmod +x "${temp_file}"

    # Move to install directory
    mv "${temp_file}" "${dest}"

    echo -e "${GREEN}✓ ${component} installed${NC}"
    return 0
}

# Check if install directory is in PATH
check_path() {
    if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
        echo ""
        echo -e "${YELLOW}Warning: ${INSTALL_DIR} is not in your PATH${NC}"
        echo ""
        echo "Add it to your PATH by adding this line to your shell profile:"
        echo ""

        # Detect shell and provide appropriate instructions
        case "$SHELL" in
            */bash)
                echo -e "  ${BLUE}echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ~/.bashrc${NC}"
                echo -e "  ${BLUE}source ~/.bashrc${NC}"
                ;;
            */zsh)
                echo -e "  ${BLUE}echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ~/.zshrc${NC}"
                echo -e "  ${BLUE}source ~/.zshrc${NC}"
                ;;
            */fish)
                echo -e "  ${BLUE}fish_add_path ${INSTALL_DIR}${NC}"
                ;;
            *)
                echo -e "  ${BLUE}export PATH=\"${INSTALL_DIR}:\$PATH\"${NC}"
                ;;
        esac
        echo ""
    fi
}

# Main installation
main() {
    echo "Installation Configuration:"
    echo "  Version: v${VERSION}"
    echo "  Install directory: ${INSTALL_DIR}"
    echo ""

    # Detect platform
    PLATFORM=$(detect_platform)
    echo "Detected platform: ${PLATFORM}"
    echo ""

    # Check dependencies
    if ! command -v curl >/dev/null 2>&1 && ! command -v wget >/dev/null 2>&1; then
        echo -e "${RED}Error: Neither curl nor wget found${NC}"
        echo "Please install curl or wget and try again"
        exit 1
    fi

    # Create install directory if it doesn't exist
    if [ ! -d "${INSTALL_DIR}" ]; then
        echo "Creating install directory: ${INSTALL_DIR}"
        mkdir -p "${INSTALL_DIR}"
    fi

    # Check write permissions
    if [ ! -w "${INSTALL_DIR}" ]; then
        echo -e "${RED}Error: No write permission for ${INSTALL_DIR}${NC}"
        echo "Try running with sudo or choose a different directory with --dir"
        exit 1
    fi

    # Download all components
    echo "Installing AGEniX components..."
    echo ""

    local failed_components=()
    for component in "${COMPONENTS[@]}"; do
        if ! download_binary "${component}" "${PLATFORM}" "${VERSION}"; then
            failed_components+=("${component}")
        fi
    done

    echo ""

    # Report results
    if [ ${#failed_components[@]} -eq 0 ]; then
        echo -e "${GREEN}╔════════════════════════════════════════════╗${NC}"
        echo -e "${GREEN}║  AGEniX Installation Complete!            ║${NC}"
        echo -e "${GREEN}╚════════════════════════════════════════════╝${NC}"
        echo ""
        echo "Installed components:"
        for component in "${COMPONENTS[@]}"; do
            version_output=$("${INSTALL_DIR}/${component}" --version 2>&1 | head -1 || echo "unknown")
            echo -e "  ${GREEN}✓${NC} ${component}: ${version_output}"
        done
        echo ""

        check_path

        echo "Quick Start:"
        echo "  1. Start AGQ (queue manager):"
        echo -e "     ${BLUE}agq --bind 127.0.0.1:6379 --session-key \$(openssl rand -hex 32)${NC}"
        echo ""
        echo "  2. Start AGX REPL (planner):"
        echo -e "     ${BLUE}export AGQ_ADDR=127.0.0.1:6379${NC}"
        echo -e "     ${BLUE}export AGQ_SESSION_KEY=<your-session-key>${NC}"
        echo -e "     ${BLUE}agx${NC}"
        echo ""
        echo "Documentation: https://github.com/agenix-sh/agenix"
        echo ""
    else
        echo -e "${RED}╔════════════════════════════════════════════╗${NC}"
        echo -e "${RED}║  Installation completed with errors        ║${NC}"
        echo -e "${RED}╚════════════════════════════════════════════╝${NC}"
        echo ""
        echo "Failed to install:"
        for component in "${failed_components[@]}"; do
            echo -e "  ${RED}✗${NC} ${component}"
        done
        echo ""
        echo "This may be because:"
        echo "  - The specified version (${VERSION}) doesn't have releases for ${PLATFORM}"
        echo "  - Network connectivity issues"
        echo "  - GitHub rate limiting"
        echo ""
        echo "Please check:"
        for component in "${failed_components[@]}"; do
            echo "  ${BASE_URL}/${component}/releases"
        done
        exit 1
    fi
}

# Run main installation
main

exit 0
