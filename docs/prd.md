# PRD: k6player

## Mission

Build a Windows-first desktop app for very fast browsing and reviewing INNOVV K6 front/rear footage. Front and rear files are treated as one logical recording in the UI.

## Primary goals

1. Auto-detect and pair front/rear clips.
2. Browse paired clips as a single logical unit.
3. Maintain smooth keyboard-first navigation and seeking.
4. Provide synchronized dual-pane playback with mpv.
5. Keep UI highly responsive on large folders.

## Secondary goals

1. Mark in/out and keep segments.
2. Maintain a kept-segments decision list.
3. Export via ffmpeg in minimal layout modes.

## Non-goals (v0/v1)

- Full nonlinear editor
- Effects/transitions/color/audio workflows
- Cloud/mobile/collab features
- Custom media decode engine

## Mandatory stack

- Tauri desktop shell
- React + TypeScript frontend
- Rust backend
- mpv for playback
- ffmpeg/ffprobe for export and metadata
- Windows 11 target first

## Functional requirements (abridged)

### Scan + ingest

- Choose folder
- Parse K6 file names
- Detect side (`_F` / `_R`)
- Pair with deterministic nearest-neighbor timestamp heuristic
- Handle missing/ambiguous/corrupt files gracefully

### Metadata

- Path, filename, side, parsed timestamp
- Duration/resolution/codec (via ffprobe)
- Size, mtime, health warnings

### Logical pair model

- Front/rear references
- Canonical start time
- Estimated duration
- Pairing confidence + reason + warnings

### Playback

- Side-by-side front/rear
- Shared logical playhead
- Synchronized play/pause/seek
- Re-sync mechanism for drift

### Keyboard-first controls

- `Space`: play/pause
- `Left/Right`: small seek
- `Shift+Left/Right`: large seek
- `Up/Down` or `J/K`: previous/next pair
- `[` and `]`: mark in/out
- `Enter`: add kept segment
- `Home/End`: pair start/end

### Performance

- Non-blocking incremental scan
- Metadata extraction in background
- Persistent cache
- List virtualization
- Fast adjacent navigation

### Kept segments + export (phase 2)

- Decision-list model (not timeline NLE)
- Segment bin with in/out ranges
- ffmpeg export modes:
  - side-by-side
  - front-only
  - rear-only

## Acceptance criteria

### v0

- Select folder and auto-group pairs
- Select pair from list
- View front/rear together
- Fast play/pause/seek
- No UI freeze
- Clear missing/malformed file warnings

### v1

- Mark in/out and collect kept segments
- Export combined output
- Clear export error UX
