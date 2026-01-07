# Installation

## Prerequisites

- **Rust** 1.70 or later (install via [rustup](https://rustup.rs/))
- **Git** for cloning the repository

### Platform-Specific Requirements

#### Linux

```bash
# Debian/Ubuntu - for SQLite bundled compilation
sudo apt-get install build-essential

# Fedora/RHEL
sudo dnf install gcc
```

#### macOS

```bash
# Xcode command line tools
xcode-select --install
```

#### Windows

- Visual Studio Build Tools with C++ workload
- Or MinGW-w64

## Clone and Build

```bash
# Clone the repository
git clone https://github.com/GeorgeQLe/gtui-suite.git
cd gtui-suite

# Build the entire workspace
cargo build --workspace

# Build in release mode for better performance
cargo build --workspace --release
```

## Build a Specific App

```bash
# Build only the habit-tracker app
cargo build -p habit-tracker

# Build with release optimizations
cargo build -p habit-tracker --release
```

## Verify Installation

```bash
# Run tests to verify everything works
cargo test --workspace

# Run a simple app
cargo run -p habit-tracker
```

## Development Setup

For development, you may also want:

```bash
# Install cargo-watch for auto-rebuild on changes
cargo install cargo-watch

# Watch and run an app
cargo watch -x "run -p habit-tracker"
```
