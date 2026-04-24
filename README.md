# rawrii

Fast desktop viewer for paired front/rear dashcam footage, with basic clip export.

## What problem this solves

Many dashcam systems store front and rear streams as separate files. Reviewing footage manually in Windows file explorer is slow and painful. `rawrii` groups those files into logical recordings so browsing and playback feel like one workflow.

## Current status

Early prototype (Milestone 0/1 foundation):

- Tauri + React + TypeScript + Rust project layout
- Folder scan command in Rust with SQLite-backed async metadata extraction
- K6-compatible filename parsing and deterministic front/rear pairing heuristic
- Pair list UI with selection and missing-side warnings
- Keyboard navigation (`J/K` or `Up/Down`) for previous/next pair
- Dual mpv playback control via two IPC-driven player instances embedded in the main app window
- Local-first AI foundation (ONNX Runtime) for vehicle and license plate discovery

## Tech stack

- Desktop shell: Tauri
- Frontend: React + TypeScript + Vite
- Backend/core: Rust
- Playback engine: mpv
- Export/Metadata: ffmpeg / ffprobe
- Database: SQLite (via rusqlite)
- AI Inference: ONNX Runtime (via ort)

## Setup

`rawrii` is a Windows-first application, but development is frequently performed under **WSL2**.

### Windows prerequisites

Targeting Windows 11 using Rust MSVC builds.

Install:

- Node.js (LTS)
- Rust via `rustup` (`stable-x86_64-pc-windows-msvc`)
- Visual Studio 2022 Build Tools (or Visual Studio 2022) with:
  - Desktop development with C++
  - MSVC v143 toolset
  - Windows 10/11 SDK
- Microsoft Edge WebView2 runtime
- mpv (required for playback)
- ffmpeg & ffprobe (required for metadata ingestion and AI analysis)

Quick verification:

```bash
node --version
npm --version
cargo --version
rustup show
mpv --version
ffmpeg -version
ffprobe -version
```

Install dependencies on Windows with Scoop:

```bash
scoop install mpv ffmpeg
```

### WSL2 Development (Linux)

If developing under WSL2, you need to install the following Linux system dependencies to compile the Tauri backend and run tests:

```bash
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev ffmpeg
```

*Note: While the app builds on Linux/WSL2, the embedded mpv playback surfaces currently rely on Win32 child-window integration and are supported on Windows only.*

## Real sample validation (`.test_examples`)

The project now treats `.test_examples` as a first-class local validation dataset for scan/parser/pairing behavior. During development:

- open `.test_examples` from the app
- verify pair counts and warnings against expected file reality
- run parser/pairing unit tests that include examples derived from this folder

## Documentation

Detailed goals and architecture can be found in:
- [Product Requirements Document (PRD)](docs/prd.md)
- [Roadmap](docs/roadmap.md)

## Contributing

Contributions are welcome. Please:

1. Open an issue describing bug/feature intent.
2. Keep PRs focused and small.
3. Include tests for parser/pairing logic changes.
4. Keep playback/export logic isolated behind modules.

## License

MIT (`LICENSE`)
