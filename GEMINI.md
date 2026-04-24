# rawrii - AI Agent Guidelines

This file serves as the primary system context and set of rules for Gemini CLI interacting with the `rawrii` project.

## Single Source of Truth
The canonical source of truth for the project's goals, architecture, tech stack, and testing guardrails is located at:
**`docs/prd.md`**

## High-Level Directives
1. **Read the PRD First:** Before making any architectural decisions, adding new dependencies, or fundamentally changing core models (like `VideoAsset` or `RecordingPair`), review `docs/prd.md`.
2. **Tech Stack Constraints:**
   - **Tauri + React/TS + Rust:** Ensure boundaries are respected. State and data processing logic belong in Rust; the UI acts reactively on IPC commands and events.
   - **Windows 11 Target:** The playback architecture explicitly relies on Win32 API child-window surfaces for `mpv`.
3. **Dependencies:** Adhere to the established Tauri compatibility policy. Do not force numerical sync across crates; use valid configurations from `Cargo.lock`.
4. **Testing Mandate:** If modifying parser or pairing heuristics, ensure changes are verified using the fixture samples defined via `.test_examples` criteria. Do not disable or bypass failing test cases without user confirmation.
5. **Metadata & AI Pipeline:** 
   - Media metadata extraction via `ffprobe` is authoritative over filename hints.
   - All machine learning analysis must be local-first (ONNX Runtime).
   - Analysis observations are timestamped and linked to the `RecordingPair` timeline.
