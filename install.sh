#!/bin/bash

################################################################################
# Deterministic Forensic Observatory — Installation Script
#
# This script sets up the complete environment for the Observatory:
# - Installs Rust (if needed)
# - Clones the repository (if not already present)
# - Builds the release binary
# - Runs smoke tests
# - Prints quick-start commands
#
# Usage:
#   bash install.sh
#
# Supported: Linux, macOS, Windows (WSL2)
################################################################################

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'  # No Color

echo -e "${BLUE}╔════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Deterministic Forensic Observatory v2V.1 — Installation           ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Step 1: Check and install Rust
echo -e "${YELLOW}[1/5] Checking Rust installation...${NC}"
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    echo -e "${GREEN}✓ Rust already installed: $RUST_VERSION${NC}"
else
    echo -e "${YELLOW}→ Installing Rust via rustup...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
    echo -e "${GREEN}✓ Rust installed${NC}"
fi

# Step 2: Verify Rust version
echo ""
echo -e "${YELLOW}[2/5] Verifying Rust version...${NC}"
RUST_VERSION=$(rustc --version | awk '{print $2}')
MIN_VERSION="1.70"
if [[ $(printf '%s\n' "$MIN_VERSION" "$RUST_VERSION" | sort -V | head -n1) == "$MIN_VERSION" ]]; then
    echo -e "${GREEN}✓ Rust version $RUST_VERSION is compatible${NC}"
else
    echo -e "${RED}✗ Rust version $RUST_VERSION < $MIN_VERSION (minimum required)${NC}"
    echo "Run: rustup update"
    exit 1
fi

# Step 3: Clone or verify repository
echo ""
echo -e "${YELLOW}[3/5] Setting up repository...${NC}"

REPO_DIR=$(pwd)
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}✗ Cargo.toml not found in current directory${NC}"
    echo "Run this script from the repository root directory"
    exit 1
fi

if [ ! -d ".git" ]; then
    echo -e "${YELLOW}→ Initializing git repository...${NC}"
    git init
    git add -A
    git commit -m "Initial commit: Deterministic Forensic Observatory v2V.1"
    git tag -a v2V.1-Phase5d-FROZEN -m "Phase 5d frozen with deterministic guarantees"
    echo -e "${GREEN}✓ Repository initialized${NC}"
else
    echo -e "${GREEN}✓ Repository exists${NC}"
fi

# Step 4: Build release binary
echo ""
echo -e "${YELLOW}[4/5] Building release binary (this may take 1-2 minutes)...${NC}"
echo ""

if cargo build --release; then
    BINARY_SIZE=$(du -h target/release/deterministic-launcher | awk '{print $1}')
    echo ""
    echo -e "${GREEN}✓ Build successful (binary: $BINARY_SIZE)${NC}"
else
    echo -e "${RED}✗ Build failed. Check Cargo.toml and dependencies.${NC}"
    exit 1
fi

# Step 5: Run smoke tests
echo ""
echo -e "${YELLOW}[5/5] Running smoke tests...${NC}"

if cargo test --release --lib 2>&1 | grep -q "test result: ok"; then
    echo -e "${GREEN}✓ All tests passed${NC}"
else
    echo -e "${YELLOW}⚠ Some tests did not pass. Installation is complete, but verify functionality.${NC}"
fi

# Success message
echo ""
echo -e "${GREEN}╔════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  ✓ Installation Complete                                           ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════════╝${NC}"
echo ""

echo -e "${BLUE}Quick Start:${NC}"
echo ""
echo "  1. View the GUI with sample trace:"
echo -e "     ${YELLOW}cargo run --release -- observatory-gui replay.json${NC}"
echo ""
echo "  2. Validate a trace:"
echo -e "     ${YELLOW}cargo run --release -- validate trace.json${NC}"
echo ""
echo "  3. Compare two runs for divergences:"
echo -e "     ${YELLOW}cargo run --release -- consensus run1.json run2.json${NC}"
echo ""
echo "  4. Export consensus analysis:"
echo -e "     ${YELLOW}cargo run --release -- consensus-export run1.json run2.json${NC}"
echo ""

echo -e "${BLUE}Documentation:${NC}"
echo "  • README.md — Full system overview"
echo "  • QUICKSTART.md — 5-minute tutorial"
echo "  • docs/PHASE_5D_FREEZE.md — Formal guarantees and freeze report"
echo "  • docs/ARCHITECTURE_DETAILED.md — Complete system architecture"
echo ""

echo -e "${BLUE}Command Reference:${NC}"
echo "  cargo run --release -- --help"
echo ""

echo -e "${GREEN}Ready to analyze kernel traces!${NC}"
