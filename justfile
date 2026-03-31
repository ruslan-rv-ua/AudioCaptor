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

# Clean Rust build artifacts
clean:
    cargo clean --manifest-path src-tauri/Cargo.toml

# Install JS dependencies
install:
    pnpm install
