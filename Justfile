# just manual: https://github.com/casey/just/#readme

_default:
    @just --list

# Runs clippy on the source code
check: 
  cargo clippy --locked --all-features --all-targets -- -D warnings

# Runs clippy on the source code, attempting to fix any errors
lint-fix:
  cargo clippy --fix --allow-staged --allow-dirty --locked --all-features --all-targets
  cargo fmt

# Formats source code
fmt:
  cargo fmt

# Checks that code is formatted correctly
check-fmt:
  cargo fmt --check

# Runs tests
test: test-examples
  cargo test --locked

check-all: 
    cargo hack --feature-powerset clippy --locked -- -D warnings

check-docs: 
    RUSTDOCFLAGS='-D warnings' RUSTFLAGS='-D warnings' cargo hack --all rustdoc --all-features --all-targets

# Test Substrate examples in the examples/ folder
test-examples: 
  @just examples/test

