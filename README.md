# Bascan

Local manga/scan reader. Rust backend with embedded SvelteKit frontend — single binary, zero dependencies at runtime.

## Quick Start

```bash
# Install frontend dependencies
just install

# Build everything (frontend + Rust binary)
just build

# Run
just run
# or with a custom library path:
just run /path/to/your/library
```

Then open [http://localhost:3001](http://localhost:3001).

## Library Structure

Bascan expects a `library/` folder (or custom path) with this structure:

```
library/
├── my-manga/
│   ├── Volume 01/
│   │   ├── 000.jpg        ← cover
│   │   ├── 001.jpg
│   │   ├── 002.jpg
│   │   ├── 006-007.jpg    ← double-page spread (detected automatically)
│   │   └── ...
│   ├── Volume 02/
│   │   ├── 000.jpg
│   │   └── ...
│   └── ...
├── another-series/
│   ├── Chapter 01/
│   └── ...
└── ...
```

- **Top-level folders** = series (shown on the home page)
- **Second-level folders** = volumes/chapters
- **Image files** inside volumes: `.jpg`, `.jpeg`, `.png`, `.webp`
- Files named like `006-007.jpg` are treated as double-page spreads

## Library Path

The library path is resolved in order:

1. CLI argument: `bascan-backend.exe /path/to/library`
2. Environment variable: `BASCAN_LIBRARY=/path/to/library`
3. Default: `./library` relative to the working directory

## Features

- **Page mode** (single page, click/arrow to navigate) and **scroll mode** (vertical long-strip)
- **Double-page spread detection** — wider display for spread images
- **Reading progress** saved per volume (localStorage, survives refresh)
- **Page position in URL** (`?p=42`) — bookmarkable, shareable
- **Next/previous chapter** navigation at volume boundaries
- **Fullscreen** with auto-hiding controls and cursor
- **Zoom** controls (+/−/reset)
- **Keyboard shortcuts**: `←`/`→` navigate, `F` fullscreen, `M` toggle mode, `+`/`−`/`0` zoom, `Esc` back
- **Dark theme** throughout

## Always-on (Windows, current user)

Run Bascan in the background, hidden, auto-started at logon — no terminal needed.

```pwsh
# One-time setup: register a hidden Scheduled Task as the current user
just install-autostart
# or with a custom library path:
just install-autostart C:\path\to\library

# Inspect task + process + logs
just status
just logs            # tail -f equivalent

# Rebuild and restart in one go
just update

# Remove it
just uninstall-autostart
```

Notes:

- Runs as the **current user**, scope: user only. No admin/elevation needed.
- Writes a launcher at `%LOCALAPPDATA%\Bascan\launch.cmd` and logs to `%LOCALAPPDATA%\Bascan\bascan.log`.
- Not reachable while logged out — by design. Bascan is a personal reader, not a server.
- `just update` kills the running process (the `.exe` is locked while running), rebuilds, then restarts the task.

## Development

```bash
# Dev mode: Rust backend + Vite dev server with hot reload
just dev

# Type-check frontend
just check
```

## Tech Stack

- **Backend**: Rust + Axum + include_dir (frontend embedded in binary)
- **Frontend**: SvelteKit + TypeScript (built as static SPA)
