#!/bin/bash
#
# Arch Linux Bootstrap Script
#
# This script acts as the "Stage 0" bootloader for the installation process.
# It ensures the minimal viable environment exists (Network, Git, Rust)
# before handing control over to the compiled Rust "Install Wizard".
#
# Security Features:
# 1. Enforces non-root execution (to comply with makepkg security).
# 2. Uses "Fail Fast" mode (set -e) to prevent partial state corruption.
# 3. Validates network connectivity before attempting package operations.

# --- Safety Flags ---
# -e: Exit immediately if a command exits with a non-zero status.
# -E: Inherit the ERR trap in subshells.
set -eE

# Error Trap: Provides a line number context if the script crashes.
trap 'echo -e "\n❌ Bootstrap failed at line $LINENO"; exit 1' ERR

# --- Context Resolution ---
# Robustly determine the script's physical location, handling symlinks.
START_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [ -z "$START_DIR" ] || [ "$START_DIR" = "/" ]; then
  START_DIR="$PWD"
fi
cd "$START_DIR"

# --- ANSI Colors ---
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

NEW_REPO_NAME="Genoa"
LEGACY_REPO_NAME="rust-wayland-power"
NEW_REPO_URL="https://github.com/Mccalabrese/Genoa.git"
LEGACY_REPO_URL="https://github.com/Mccalabrese/rust-wayland-power.git"

echo -e "${BLUE}🔵 [Stage 1] Bootstrapping Environment...${NC}"

# 1. Security Check: Root Privileges
# We MUST NOT run as root because 'makepkg' (used later for AUR) strictly forbids it
# to prevent arbitrary code execution risks during build.
if [ "$EUID" -eq 0 ]; then
  echo -e "${RED}❌ Do not run this script as root!${NC}"
  echo "   The installer requires standard user permissions to build packages."
  echo "   It will ask for sudo passwords when necessary."
  exit 1
fi

# 2. Pre-Flight Check: Connectivity
echo -e "${BLUE}Checking network connectivity...${NC}"
if ! ping -c 1 archlinux.org &>/dev/null; then
  echo -e "${RED}⚠️  No internet connection detected.${NC}"
  echo "   Please connect to Wi-Fi (iwctl) or Ethernet before proceeding."
  exit 1
fi

# 3. State Synchronization
# We run a full system upgrade (-Syu) to ensure the local package database
# matches the binaries we are about to install.
echo -e "${BLUE}Synchronizing package databases...${NC}"
sudo pacman -Syu --noconfirm archlinux-keyring pacman-mirrorlist

# 4. Repository Discovery
# Determines if we are running from inside the cloned repo,
# or if it exists in the current folder (Resume Mode),
# or if we need to fetch it.

if [ -f "sysScripts/install-wizard/Cargo.toml" ]; then
  echo "✅ Running from inside repository."
  REPO_ROOT="$PWD"
elif [ -d "$NEW_REPO_NAME" ]; then
  echo -e "${GREEN}📂 Found existing repository. Resuming installation...${NC}"
  cd "$NEW_REPO_NAME"
  REPO_ROOT="$PWD"
elif [ -d "$LEGACY_REPO_NAME" ]; then
  echo -e "${GREEN}📂 Found existing repository. Resuming installation...${NC}"
  cd "$LEGACY_REPO_NAME"
  REPO_ROOT="$PWD"
else
  if [ -d ".git" ]; then
    # We are in a generic git repo (unlikely but possible)
    REPO_ROOT="$PWD"
  else
    echo -e "${GREEN}Cloning repository...${NC}"
    # Install git only if missing
    if ! command -v git &>/dev/null; then
      sudo pacman -S --needed --noconfirm git
    fi

    # Clone the repo (prefer the new canonical location, then fallback)
    if git clone "$NEW_REPO_URL" "$NEW_REPO_NAME"; then
      cd "$NEW_REPO_NAME"
      REPO_ROOT="$PWD"
    else
      echo -e "${BLUE}Primary repository clone failed. Trying legacy URL...${NC}"
      git clone "$LEGACY_REPO_URL" "$LEGACY_REPO_NAME"
      cd "$LEGACY_REPO_NAME"
      REPO_ROOT="$PWD"
    fi
  fi
fi

# 5. Toolchain Provisioning
# Installs the compiler infrastructure required to build the Rust wizard.
echo -e "${BLUE}Provisioning build toolchain...${NC}"
# base-devel: Required for compiling C dependencies (gcc, make, sudo, etc.)
# rustup: The Rust toolchain installer (preferred over system 'rust' package for flexibility)
sudo pacman -S --needed --noconfirm base-devel rustup git pkgconf wget curl ca-certificates

# 6. Rust Environment Setup
# We cannot rely on 'command -v cargo' because pacman installs empty shims.
# We must explicitly force rustup to install the stable toolchain.
echo -e "${GREEN}Ensuring Rust stable toolchain is active...${NC}"
rustup default stable

# Ensure cargo binaries are in PATH for this session
export PATH="$HOME/.cargo/bin:$PATH"

# 7. Handover to Rust
echo -e "${BLUE}🔵 [Stage 2] Launching Install Wizard...${NC}"
echo -e "Compiling installer binary..."

# Verify directory exists before cd to prevent obscure error messages
if [ ! -d "$REPO_ROOT/sysScripts/install-wizard" ]; then
  echo -e "${RED}❌ Critical Error: Installer source code not found at:${NC}"
  echo "   $REPO_ROOT/sysScripts/install-wizard"
  exit 1
fi

cd "$REPO_ROOT/sysScripts/install-wizard"

# Build and execute the Rust installer in Release mode for speed and pass the REPO_ROOT to the installer.
# The Rust binary handles the complex logic, hardware detection, and UI.
# export REPO_ROOT="$REPO_ROOT"
# cargo run --release
REPO_ROOT="$REPO_ROOT" cargo run --release

# If Rust exits with 0, we are done.
if [ $? -eq 0 ]; then
  echo -e "${GREEN}✅ Bootstrap complete! Rebooting is recommended.${NC}"
  exit 0
else
  echo -e "${RED}❌ Installer exited with errors.${NC}"
  exit 1
fi
