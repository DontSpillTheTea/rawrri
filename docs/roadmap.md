# Roadmap

## Milestone 0: bootstrap

- [x] Tauri + React + TypeScript + Rust structure
- [x] Initial command bridge
- [x] Project docs (`README`, `prd`, `architecture`, `roadmap`)
- [ ] Toolchain CI and packaging setup

## Milestone 1: scan + pair + browse

- [x] Folder scan command
- [x] K6-compatible filename profile parser
- [x] Deterministic pairing heuristic with threshold
- [x] Logical pair list UI
- [x] Missing-side warnings
- [x] Basic persistent cache by folder
- [ ] Incremental scan progress events
- [ ] File-change invalidation strategy

## Milestone 1.5: validated browse foundation

- [x] Real sample data pass against `.test_examples` naming realities
- [x] Parser and pairing tests include fixture-derived examples
- [x] Scan diagnostics surfaced in backend and UI
- [x] Stable deterministic IDs for assets/pairs
- [x] Explicit missing-side badges and pairing reason visibility
- [ ] Refresh invalidation to avoid stale cache when folder contents change

## Milestone 2: dual playback (mpv)

- [x] Integrate mpv process control via IPC (two instances)
- [x] Render front/rear panes embedded in app window via Win32 child surfaces
- [x] Shared logical playhead model
- [x] Synchronized play/pause/seek
- [x] Keyboard transport shortcuts
- [ ] Optional one-click resync

## Milestone 3: speed polish

- [x] SQLite-backed asynchronous job queue foundation
- [x] Async ffprobe metadata worker
- [ ] Warm-open cache validation optimization
- [ ] Virtualized pair list for large libraries
- [ ] Adjacent pair preloading
- [ ] Interaction latency profiling/tuning

## Milestone 3.5: AI Pipeline Core

- [x] Robust frame extraction foundation (ffmpeg PPM stream)
- [ ] Model downloader (YOLOv8, OCR models)
- [ ] ONNX Runtime integration (ort)
- [ ] Real-time inference implementation
- [ ] Observation debouncing and IoU tracking
- [ ] Search and filtering UI

## Milestone 4: kept segments

- [ ] Mark in/out controls
- [ ] Add kept segment decision list entries
- [ ] Segment bin UI with jump/delete
- [ ] Persist kept segments

## Milestone 5: export

- [ ] ffmpeg export job queue
- [ ] Side-by-side / front-only / rear-only modes
- [ ] Progress reporting + cancellation
- [ ] Deterministic output naming
- [ ] Error reporting and logs
