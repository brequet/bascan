# Bascan — Local Manga Reader

set shell := ["pwsh", "-NoProfile", "-Command"]

# Default library path (override with: just run C:\path\to\library)
library := "library"

# Build everything: frontend + Rust binary
build:
    pnpm --prefix packages/frontend build; cargo build --release --manifest-path packages/backend-rs/Cargo.toml

# Run the production binary (optionally pass library path)
run path=library:
    packages/backend-rs/target/release/bascan-backend.exe {{path}}

# Dev mode: Rust backend + Vite frontend (with proxy)
dev path=library:
    Start-Process -NoNewWindow -FilePath "cargo" -ArgumentList "run","--manifest-path","packages/backend-rs/Cargo.toml","--","{{path}}"; pnpm --prefix packages/frontend dev

# Install frontend dependencies
install:
    pnpm --prefix packages/frontend install

# Type-check frontend
check:
    pnpm --prefix packages/frontend check
