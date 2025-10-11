# Rust Library Project Setup Guide

## Installation

### 1. Install Rust via rustup
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env  # Add cargo to PATH
```

### 2. Add official components
```bash
rustup component add clippy rustfmt rust-analyzer
```

### 3. Install useful tools
```bash
cargo install just          # Task runner (recommended)
cargo install cargo-watch   # Auto-rebuild on changes
cargo install cargo-edit    # Add/remove deps from CLI
```

### 4. Update everything
```bash
rustup update
```

## Project Structure

```
my_rust_lib/
├── Cargo.toml              # Package manifest & dependencies
├── Cargo.lock              # Lock file (commit for apps, optional for libs)
├── .gitignore
├── README.md
├── LICENSE
├── src/
│   ├── lib.rs              # Library root - public API entry point
│   ├── module1.rs          # Top-level module (option 1)
│   ├── module2/            # Top-level module (option 2 - folder)
│   │   ├── mod.rs          # Module root (or use module2.rs instead)
│   │   ├── submod1.rs
│   │   └── submod2.rs
│   └── bin/                # Optional executables
│       └── cli.rs          # Creates binary: cargo run --bin cli
├── tests/                  # Integration tests
│   └── integration_test.rs
├── benches/                # Benchmarks
│   └── benchmark.rs
├── examples/               # Usage examples
│   └── basic_usage.rs      # Run: cargo run --example basic_usage
├── rustfmt.toml            # Code formatter config (optional)
├── clippy.toml             # Linter config (optional)
├── justfile                # Task automation (if using just)
└── .github/
    └── workflows/
        └── ci.yml          # CI/CD configuration
```

## Configuration Files

### Cargo.toml (basic)
```toml
[package]
name = "my_rust_lib"
version = "0.1.0"
edition = "2021"

[dependencies]
# Add dependencies: cargo add serde

[dev-dependencies]
# Test-only dependencies

[build-dependencies]
# Build script dependencies
```

### rustfmt.toml (optional)
```toml
max_width = 100
edition = "2021"
```

### justfile (recommended)
```just
# Default recipe
default: check test

# Run all checks
check:
    cargo fmt -- --check
    cargo clippy --all-targets -- -D warnings

# Format code
fmt:
    cargo fmt

# Run tests
test:
    cargo test

# Build release
release:
    cargo build --release

# Run CI checks locally
ci: check test
    cargo doc --no-deps

# Watch and auto-test
watch:
    cargo watch -x test
```

### .gitignore
```
/target/
Cargo.lock  # Remove this line if building an application
**/*.rs.bk
```

## Common Commands

### Project Creation
```bash
cargo new my_lib --lib      # Create new library
cargo new my_app            # Create new binary/application
```

### Building & Running
```bash
cargo build                 # Debug build
cargo build --release       # Optimized build
cargo run                   # Run default binary
cargo run --bin cli         # Run specific binary
cargo run --example basic   # Run example
```

### Testing
```bash
cargo test                  # Run all tests
cargo test --lib            # Only unit tests
cargo test integration      # Tests matching "integration"
cargo test -- --nocapture   # Show println! output
```

### Code Quality
```bash
cargo fmt                   # Format code
cargo fmt -- --check        # Check formatting (CI)
cargo clippy                # Lint code
cargo clippy -- -D warnings # Treat warnings as errors (CI)
```

### Documentation
```bash
cargo doc --open            # Generate & open docs
cargo doc --no-deps         # Docs for your crate only
```

### Dependencies
```bash
cargo add serde             # Add dependency
cargo add serde --features derive
cargo add tokio --dev       # Add dev dependency
cargo remove serde          # Remove dependency
cargo update                # Update dependencies
```

### Utilities
```bash
cargo check                 # Fast check (no codegen)
cargo clean                 # Remove build artifacts
cargo tree                  # Show dependency tree
```

### With just
```bash
just                        # Run default recipe
just test                   # Run specific recipe
just ci                     # Run CI checks locally
just watch                  # Auto-run tests on changes
```

## Editor Setup (VS Code + rust-analyzer)

### Install Extension
Search for "rust-analyzer" in VS Code extensions

### Settings (.vscode/settings.json)
```json
{
  "rust-analyzer.check.command": "clippy",
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

## Neovim Setup

### With native LSP
```lua
-- Install rust-analyzer via rustup component add rust-analyzer
require('lspconfig').rust_analyzer.setup({
  settings = {
    ['rust-analyzer'] = {
      checkOnSave = {
        command = "clippy"
      }
    }
  }
})
```

## CI Setup (GitHub Actions)

### .github/workflows/ci.yml
```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: actions/cache@v4
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
      - run: cargo fmt -- --check
      - run: cargo clippy -- -D warnings
      - run: cargo test
      - run: cargo build --release
```

## Quick Start Checklist

1. [ ] Install rustup
2. [ ] Add components: `rustup component add clippy rustfmt rust-analyzer`
3. [ ] Install tools: `cargo install just cargo-watch`
4. [ ] Create project: `cargo new my_lib --lib`
5. [ ] Create justfile for common tasks
6. [ ] Setup editor with rust-analyzer
7. [ ] Create .github/workflows/ci.yml
8. [ ] Start coding in src/lib.rs

## Tips

- Run `just ci` before committing to catch issues early
- Use `cargo watch -x test` during development for instant feedback
- Commit Cargo.lock for applications, usually don't commit for libraries
- Read compiler errors carefully - they're actually helpful in Rust
- Use `cargo doc --open` to browse documentation for your dependencies