# K6-Compatible Filename Notes (`.test_examples`)

Observed from local copied sample set:

- Video files are under `INNOVVK6/VIDEO`.
- Photo files (`.JPG`) are present under `INNOVVK6/Photo` and must be ignored by video scan.
- Video pattern in samples is consistent with:
  - `YYYYMMDD_HHMMSS_<sequence>_F.MP4`
  - `YYYYMMDD_HHMMSS_<sequence>_R.MP4`
- Extension and side marker casing can vary in general, so parser remains case-insensitive.
- Front/rear timestamps differ by about one second in many adjacent pairs.
- Some historical-looking timestamps exist (for example `20000101_*`) and should still parse/pair deterministically.

Implementation implications:

- Parser extracts `side`, timestamp, sequence, raw timestamp string, extension.
- Pairing is nearest-neighbor within configurable threshold and never double-assigns a side.
- Partial pairs are preserved and surfaced with explicit warnings.
