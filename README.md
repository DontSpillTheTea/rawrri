# rawrii

Fast desktop viewer for paired front/rear dashcam footage, with basic clip export.

## What problem this solves

Many dashcam systems store front and rear streams as separate files. Reviewing footage manually in Windows file explorer is slow and painful. `rawrii` groups those files into logical recordings so browsing and playback feel like one workflow.

## Current status

Early prototype (Milestone 0/1 foundation):

- Tauri + React + TypeScript + Rust project layout
- Folder scan command in Rust
- K6-compatible filename parsing and deterministic front/rear pairing heuristic
- Pair list UI with selection and missing-side warnings
- Keyboard navigation (`J/K` or `Up/Down`) for previous/next pair
- Dual mpv playback control via two IPC-driven player instances (external windows in v0)

## Naming assumptions (current profile)

Current built-in profile is K6-compatible naming:

- `YYYYMMDD_HHMMSS_<sequence>_F.MP4`
- `YYYYMMDD_HHMMSS_<sequence>_R.MP4`

Examples:

- `20260323_114324_000023_F.MP4`
- `20260323_114325_000024_R.MP4`

## Tech stack

- Desktop shell: Tauri
- Frontend: React + TypeScript + Vite
- Backend/core: Rust
- Planned playback engine: mpv
- Planned export engine: ffmpeg / ffprobe

## Setup

### Windows prerequisites

`rawrii` targets Windows 11 first and uses Rust MSVC builds under Tauri.

Install:

- Node.js (LTS)
- Rust via `rustup` (`stable-x86_64-pc-windows-msvc`)
- Visual Studio 2022 Build Tools (or Visual Studio 2022) with:
  - Desktop development with C++
  - MSVC v143 toolset
  - Windows 10/11 SDK
- Microsoft Edge WebView2 runtime
- mpv (required for Milestone 2 playback)

Quick verification:

```bash
node --version
npm --version
cargo --version
rustup show
mpv --version
```

Install mpv on Windows with Scoop:

```bash
scoop install mpv
```

If you see `link.exe not found` during `npm run tauri:dev`, install/repair Visual Studio Build Tools with the C++ workload above and restart the terminal so `link.exe` is on PATH.

### Tauri dependency compatibility policy

Tauri Rust crates and npm packages do not always share identical version numbers. Use published, mutually compatible versions from the current Tauri ecosystem rather than forcing numeric sameness.

Current known-good set in this repo:

- npm: `@tauri-apps/cli` `^2.10.1`
- npm: `@tauri-apps/api` `^2.10.1`
- npm: `@tauri-apps/plugin-dialog` `^2.6.0`
- Rust: `tauri` `2.10.3`
- Rust: `tauri-build` `2.5.6`
- Rust: `tauri-plugin-dialog` `2.6.0`

If Cargo reports a version does not exist (for example `tauri-build = "^2.10.0"`), switch to a published compatible crate version instead of trying to numerically align all packages.

`src-tauri/Cargo.lock` is kept in the repo for deterministic Tauri/Rust resolution and should remain committed.

### Install and run

```bash
npm install
npm run tauri:dev
```

Build:

```bash
npm run tauri:build
```

## Roadmap

- Milestone 0: bootstrap and docs
- Milestone 1: scan + parse + pair + browse
- Milestone 2: dual mpv playback with synchronized controls
- Milestone 3: speed polish (cache warm-open, virtualization, async metadata)
- Milestone 4: kept segments (in/out decision list)
- Milestone 5: ffmpeg export (side-by-side/front/rear modes)

See `docs/roadmap.md` for details.

Planned direction: expand camera profiles beyond the current K6-compatible baseline.

## Real sample validation (`.test_examples`)

The project now treats `.test_examples` as a first-class local validation dataset for scan/parser/pairing behavior. During development:

- open `.test_examples` from the app
- verify pair counts and warnings against expected file reality
- run parser/pairing unit tests that include examples derived from this folder

Policy for now: keep `.test_examples` as local developer fixture data unless otherwise decided for repository sharing size/privacy.

## Current limitations

- mpv playback currently uses external native windows (embedding in Tauri UI comes later)
- No ffprobe metadata extraction yet (duration/resolution/codec placeholders)
- Cache is basic and currently keyed by folder path
- No export pipeline yet

## Contributing

Contributions are welcome. Please:

1. Open an issue describing bug/feature intent.
2. Keep PRs focused and small.
3. Include tests for parser/pairing logic changes.
4. Keep playback/export logic isolated behind modules.

## License

MIT (`LICENSE`)
