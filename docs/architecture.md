# Architecture Overview

## Design priorities

1. User-perceived speed over architectural purity.
2. Treat front/rear files as one logical recording.
3. Keep UI responsive while scan/metadata/export run in background.
4. Isolate media-engine specific logic behind backend module boundaries.

## Tauri dependency policy

Tauri v2 uses multiple Rust crates and npm packages with independent release cadences. Compatibility matters more than numeric sameness across package names.

Example: `tauri` and `tauri-build` may use different minor version lines while still being the correct compatible combination.

## High-level components

### Frontend (`src/`)

- React UI shell
- Recording browser and pair selection
- Keyboard shortcuts and transport controls
- Playback layout (mpv panes)
- Kept-segments and export UI (later milestones)

### Tauri bridge

- Frontend invokes backend commands
- Backend emits progress events for scan/export/playback status

### Backend (`src-tauri/src/`)

- `scanner`: folder traversal and file ingest
- `filename_parser`: deterministic dashcam filename parser (currently K6-compatible profile)
- `pairing`: nearest-neighbor pairing with configurable threshold
- `metadata`: ffprobe extraction pipeline (planned)
- `cache`: persistent folder scan cache
- `playback`: mpv orchestration boundary (planned)
- `export`: ffmpeg job queue boundary (planned)
- `settings`, `state`, `logging`: runtime coordination

## Data model

Core entities:

- `VideoAsset`
- `RecordingPair`
- `PlaybackSession`
- `KeptSegment`
- `ExportJob`

These align with the PRD data contracts in `docs/prd.md`.

## Pairing heuristic (current implementation)

1. Parse valid candidates for supported naming profiles (currently K6-compatible).
2. Split by side.
3. Sort by parsed timestamp.
4. For each front asset, choose nearest unused rear within threshold.
5. Keep unmatched assets as partial pairs with warnings.
6. Emit confidence and reason for each pair.

## Caching strategy (current baseline)

- Store scan results per folder in local app-data JSON cache.
- Reuse cache for warm-open speed.
- Planned improvement: mtime/signature invalidation and partial re-index.

## Planned playback architecture

Milestone 2 introduces dual mpv control:

- Shared logical playhead in app state
- Per-side offset mapping
- Synchronized play/pause/seek commands
- Drift correction via periodic sync

## Planned export architecture

Milestone 5 introduces ffmpeg decision-list export:

- Segment extraction from kept ranges
- Mode-based composition (side-by-side/front/rear)
- Ordered concatenation
- Progress + cancellation + failure reporting
