# Local Fixture Data

This folder is used for local validation of scanner/parser/pairing behavior with real INNOVV K6 copied media.

Expected structure example:

- `.test_examples/INNOVVK6/VIDEO/*.MP4`
- `.test_examples/INNOVVK6/Photo/*.JPG`

Expected K6 video name shape:

- `YYYYMMDD_HHMMSS_<sequence>_F.MP4`
- `YYYYMMDD_HHMMSS_<sequence>_R.MP4`

Notes:

- Keep sensitive or large raw media local.
- Unit tests in Rust should include synthetic filename fixtures derived from these real examples.
- Open this folder from the app to validate pair behavior and warning rendering.
