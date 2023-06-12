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

check-fmt:
  cargo fmt --check

# Runs tests
test:
  cargo test --locked --all-features
