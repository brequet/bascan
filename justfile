# Bascan — Local Manga Reader

set shell := ["pwsh", "-NoProfile", "-Command"]

# Build everything: frontend + Rust binary
build:
    pnpm --prefix packages/frontend build; cargo build --release --manifest-path packages/backend-rs/Cargo.toml

# Run the production binary
run:
    packages/backend-rs/target/release/bascan-backend.exe

# Dev mode: Rust backend + Vite frontend (with proxy)
dev:
    Start-Process -NoNewWindow -FilePath "cargo" -ArgumentList "run","--manifest-path","packages/backend-rs/Cargo.toml"; pnpm --prefix packages/frontend dev

# Install frontend dependencies
install:
    pnpm --prefix packages/frontend install

# Type-check frontend
check:
    pnpm --prefix packages/frontend check
