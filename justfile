# Bascan — Local Manga Reader

set shell := ["pwsh", "-NoProfile", "-Command"]

# Build everything and produce the single exe
build:
    pnpm --filter @bascan/frontend build; cargo build --release --manifest-path packages/backend-rs/Cargo.toml

# Run the production binary (build first if needed)
run:
    packages/backend-rs/target/release/bascan-backend.exe

# Dev mode: Bun backend + Vite frontend (with proxy)
dev:
    Start-Process -NoNewWindow -FilePath "bun" -ArgumentList "run","src/index.ts" -WorkingDirectory "packages/backend"; pnpm --filter @bascan/frontend dev

# Install all dependencies
install:
    pnpm install
