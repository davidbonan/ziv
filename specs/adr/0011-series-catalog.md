# ADR 0011 — Series catalog

- **Status**: accepted
- **Date**: 2026-10-02

## Context
The open photos live in memory only: `Session` is a list of paths that the next
import replaces, and nothing but the export settings survives a quit. Closing
the app means importing the same photos again to keep working on them.

The author wants the imports kept as **series** listed in a sidebar. A series is
what one import brought in: a folder, or several photos picked at once.

ADR 0003 chose sidecars and rejected a catalog *for the edits*, because a
catalog ties edits to absolute paths and to one machine. That reason does not
apply to remembering which photos were imported: this ADR adds an index and
leaves the edits where ADR 0003 put them.

Already in `Cargo.toml`: `dirs` 6 (the models folder is
`dirs::data_dir()/ziv/models`), `serde` + `serde_json` 1, `sha2` 0.10, `image`
0.25 with its JPEG codec. Checked on crates.io, 2026-10-02: `rusqlite` 0.40.2
(MIT).

Thumbnails are decoded again at every import, 320 px on the long side, from the
embedded preview for a RAW. The time this takes for a large series was not
measured.

## Decision
| Concern | Choice |
|---------|--------|
| What the catalog holds | the series, the photos of each as absolute paths, the series and the photo last looked at; never an edit |
| Where | `dirs::data_dir()/ziv/catalog.json` |
| Format | JSON, UTF-8, through `serde` + `serde_json`, a top-level integer `version` starting at 1 |
| Writing | to a temporary file in the same folder, then renamed over the catalog |
| Reading | a missing catalog means no series; a `version` newer than the app knows, or a file that does not parse, is reported and never overwritten |
| Content of a series | the list of photos as imported, frozen: a file added to the folder later is not part of it |
| Missing photo or folder | stays in the series, shown as not found |
| Import date | the local calendar day, through `chrono` 0.4 (already built: `rawler` depends on it) |
| Edited count of a series | derived from the sidecars (ADR 0003: no sidecar means no edit), not stored |
| Thumbnail cache | one JPEG per photo, 320 px on the long side, in `dirs::cache_dir()/ziv/thumbnails` |
| Thumbnail key | SHA-256 of the photo's path, size and modification time: a changed source misses the cache |
| Losing the cache | harmless: a thumbnail not found is decoded from the photo and stored again |

## Alternatives rejected
- **`eframe` persistence** (what ADR 0004 uses for settings) — no code for the
  file, but the catalog would sit inside the settings value, be written when
  `eframe` autosaves, and have no version or unreadable-file behavior of its own.
- **SQLite (`rusqlite`)** — pays off for search and filters over thousands of
  photos, which nothing asks for; a new dependency carrying a C library. Build
  time not evaluated.
- **Rescanning the folder when a series is opened** — new files would appear by
  themselves, but the series would stop being what the author imported; the
  author chose the frozen list.
- **No thumbnail cache until the cost is measured** — the author chose to have
  it from the start, so that a reopened series shows at once.
- **Thumbnails inside the catalog file** — one file rewritten at every selection
  would carry megabytes of pictures.

Thumbnail file size, growth of the cache folder and write latency of the
catalog were not evaluated.

## Consequences
- The catalog is tied to this machine's absolute paths: moving a folder leaves
  its series not found until it is located again. The edits still travel with
  the folder (ADR 0003).
- A thumbnail shows the photo as shot, as today: it does not depend on the edit,
  so an edit never invalidates the cache.
- A changed or deleted photo leaves its old thumbnail in the cache folder;
  nothing removes it yet.
- `Session` stops being the only place that knows the open photos: it becomes
  the view of one series of the catalog.
- `specs/overview.md` §3: the edit persistence row no longer says "no catalog";
  it says the catalog holds no edit.
