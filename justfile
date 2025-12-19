# Shell for Linux
set shell := ["sh", "-c"]

# Run Powershell 7
set windows-shell := ["pwsh", "-NoLogo", "-Command"]

check:
    cargo check --release
    cargo clippy -- -A clippy::style -A clippy::complexity

format:
    cargo-workspace-lints workspace-lints
    taplo fmt
    cargo +nightly fmt
