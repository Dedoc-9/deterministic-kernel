# Installation Guide

Complete step-by-step instructions for installing the Deterministic Forensic Observatory.

## System Requirements

- **OS:** Linux (Ubuntu 20.04+), macOS (10.15+), or Windows (WSL2)
- **RAM:** 512 MB minimum (1 GB recommended)
- **Disk:** 100 MB for installation + binary
- **Internet:** Required for Rust installation
- **Shell:** Bash

## Option 1: Automated Installation (Recommended)

The easiest way to get started:

```bash
git clone https://github.com/yourusername/deterministic-forensic-observatory.git
cd deterministic-forensic-observatory
bash install.sh
```

The script will:
1. Check for Rust (install if missing)
2. Verify Rust version
3. Clone/init git repository
4. Build release binary (1-2 minutes)
5. Run smoke tests
6. Print quick-start commands

**Estimated time:** 3-5 minutes (depending on internet speed)

## Option 2: Manual Installation (Step-by-Step)

### Step 1: Install Rust

If you don't have Rust installed:

```bash
# Download and run rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Load Rust in your shell
source $HOME/.cargo/env

# Verify installation
rustc --version  # Should show 1.70.0 or later
```

### Step 2: Clone the Repository

```bash
git clone https://github.com/yourusername/deterministic-forensic-observatory.git
cd deterministic-forensic-observatory
```

### Step 3: Build the Release Binary

```bash
# Build (this will take 1-2 minutes on first run)
cargo build --release

# Binary location
ls -lh target/release/deterministic-launcher
```

Expected output:
```
-rwxr-xr-x  deterministic-launcher  15M
```

### Step 4: Run Tests (Optional)

```bash
# Run smoke tests
cargo test --release --lib

# Expected output:
# test result: ok. 5 passed
```

### Step 5: Verify Installation

```bash
# Test the CLI help
cargo run --release -- --help

# Should print available commands:
# - validate <trace.json>
# - observatory-gui <trace.json>
# - consensus <run1.json> <run2.json>
# - consensus-export <run1.json> <run2.json>
```

## Installation Verification

To verify everything is installed correctly:

```bash
# 1. Check Rust
rustc --version  # Should be 1.70.0+
cargo --version  # Should be 1.70.0+

# 2. Check binary exists
ls target/release/deterministic-launcher

# 3. Test basic command
cargo run --release -- validate replay.json
# Should print: FORENSIC REPORT...

# 4. Test GUI launch (if X11/display available)
cargo run --release -- observatory-gui replay.json
# Should open GUI window
```

## Platform-Specific Notes

### Linux (Ubuntu/Debian)

```bash
# If Rust installation fails, try:
sudo apt-get update
sudo apt-get install build-essential curl

# Then run Rust installer
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
```

**GUI dependencies:**
```bash
# For full X11 support
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

### macOS

```bash
# Install Xcode command-line tools if needed
xcode-select --install

# Then install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Load Rust
source $HOME/.cargo/env
```

### Windows (WSL2)

```bash
# In WSL2 terminal
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# For GUI, ensure WSL2 has display forwarding
# See: https://docs.microsoft.com/en-us/windows/wsl/tutorials/gui-apps
```

### Docker (Optional)

If you prefer containerized installation:

```dockerfile
FROM ubuntu:22.04

RUN apt-get update && apt-get install -y \
    curl \
    git \
    build-essential

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN ~/.cargo/bin/cargo --version

WORKDIR /app
COPY . .
RUN ~/.cargo/bin/cargo build --release

ENTRYPOINT ["./target/release/deterministic-launcher"]
```

Build:
```bash
docker build -t observatory .
docker run observatory validate trace.json
```

## Troubleshooting

### "command not found: cargo"

```bash
# Rust installation didn't load in shell
source $HOME/.cargo/env

# Add to ~/.bashrc to persist
echo 'source $HOME/.cargo/env' >> ~/.bashrc
```

### "error: could not compile"

```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

### "GUI window won't open (Linux)"

```bash
# Check X11 is available
echo $DISPLAY  # Should print :0 or :1

# If empty, you need X11 forwarding or an alternative display
# On headless systems, use CLI commands instead
```

### "out of memory"

If building fails with memory errors:

```bash
# Check available RAM
free -h

# Limit parallel jobs
cargo build --release -j 1

# Or use swap (temporary workaround)
sudo dd if=/dev/zero of=/swapfile bs=1G count=2
sudo mkswap /swapfile
sudo swapon /swapfile
```

### "Certificate error" during Rust installation

```bash
# Try with different certificate settings
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs --insecure | sh -s -- -y

# Or download directly
wget https://sh.rustup.rs -O rustup.sh
bash rustup.sh -y
```

## Updating

To update to the latest version:

```bash
git pull origin main
cargo build --release

# Or use the install script again
bash install.sh
```

## Uninstallation

To remove the observatory:

```bash
# Remove the repository
rm -rf /path/to/deterministic-forensic-observatory

# (Optional) Remove Rust if you don't need it
rustup self uninstall
```

## Next Steps

After installation:

1. **Run Quick Start:** See [QUICKSTART.md](QUICKSTART.md)
2. **Read Documentation:** See [README.md](README.md)
3. **View Examples:** `cargo run --release -- observatory-gui replay.json`
4. **Check Architecture:** See [docs/ARCHITECTURE_DETAILED.md](docs/ARCHITECTURE_DETAILED.md)

## Support

- **Installation Help:** Check [Troubleshooting](#troubleshooting) section
- **Rust Help:** https://www.rust-lang.org/en-US/learn
- **Issues:** GitHub Issues
- **Email:** bigdilly95@gmail.com

---

**Installed?** Start with:
```bash
cargo run --release -- observatory-gui replay.json
```
