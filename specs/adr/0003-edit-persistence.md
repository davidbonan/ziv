# ADR 0003 — Edit persistence

- **Status**: accepted
- **Date**: 2026-10-02

## Context
M2 makes edits that must survive closing the app. Edits are data and the source
file is never written (ADR 0001). The author chose **sidecar files next to the
photos, no catalog**: a folder of photos carries its own edits and can be moved,
copied or backed up as plain files. What was left to settle: the file format,
its name, and how it evolves.

Crates checked on crates.io, 2026-10-02: `serde` 1.0.229 and `serde_json`
1.0.151 (MIT OR Apache-2.0; `serde` is already in `Cargo.lock`), `toml` 1.1.6,
`quick-xml` 0.42.0.

## Decision
| Concern | Choice |
|---------|--------|
| Where | one sidecar per photo, in the photo's folder |
| Name | the photo's full file name + `.ziv.json` — `DSC07070.ARW.ziv.json` |
| Format | JSON, UTF-8, pretty-printed, through `serde` + `serde_json` |
| Version | a top-level integer `version`, starting at 1 |
| Content | the edit parameters only; a parameter at its default is still written |
| Reading | a missing sidecar means no edit; a missing field means that parameter's default; an unknown field is ignored; a `version` newer than the app knows, or a file that does not parse, is reported and never overwritten |
| Writing | to a temporary file in the same folder, then renamed over the sidecar |
| No edit | a photo whose every parameter is at its default has no sidecar: the file is removed |

## Alternatives rejected
- **Catalog (SQLite or similar)** — the author's choice is sidecars. A catalog
  ties edits to absolute paths and to one machine's database.
- **XMP (`DSC07070.xmp`)** — the file Lightroom and others write. ziv's
  adjustments are not Adobe's: storing them under `crs:` would be wrong, and
  sharing the file means rewriting XML another application owns. Reading
  Lightroom's settings is not a goal.
- **Name without the photo's extension (`DSC07070.ziv.json`)** — a RAW and its
  JPEG shot together would share one sidecar.
- **TOML** — as readable, but nested arrays of points (curves, later masks)
  are heavier to write and read than in JSON.
- **A binary format** — not diffable, not repairable by hand; sidecar size is
  not a concern at this scale.

Write latency and behavior on network or read-only folders were not evaluated.

## Consequences
- Edits follow the photo only if the sidecar is moved with it; renaming a photo
  outside ziv orphans its sidecar.
- A read-only folder cannot hold edits: the failure has to be shown to the user,
  the spec of M2 says how.
- Every later feature that persists something per photo (masks, crop) extends
  this file and bumps `version` only when an old reader would misread it.
- The edit model derives `Serialize` / `Deserialize`; `serde` stays out of the
  image math itself.
