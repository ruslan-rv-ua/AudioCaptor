# AudioCaptor — Tauri v2 app

default:
    @just --list

# Start Tauri dev server
dev:
    pnpm tauri dev

# Production build — minimal exe, slow compile
build:
    pnpm tauri build --no-bundle

# Fast build — larger exe, quick compile
build-fast:
    pnpm tauri build --no-bundle -- --profile release-fast

# Frontend-only dev server
vite-dev:
    pnpm vite dev --port 1420

# All local checks; stops at the first failure
check:
    pnpm vite:build
    pnpm check
    cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
    cargo test --manifest-path src-tauri/Cargo.toml
    cargo fmt --manifest-path src-tauri/Cargo.toml -- --check

# NVDA smoke test of the release exe (just build first); `just smoke <scenario>` runs one: launch, record, dialogs, tray
smoke *scenario:
    powershell -NoProfile -File scripts/smoke-nvda.ps1 {{scenario}}

# Clean Rust build artifacts
clean:
    cargo clean --manifest-path src-tauri/Cargo.toml

# Install JS dependencies
install:
    pnpm install
