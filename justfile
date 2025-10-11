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

# Install the info binary
install:
    cargo install --path .

# Run the info CLI tool
info *args:
    cargo run --bin info -- {{args}}