# ziv — Export

## 1. Goal
Write the developed photo — or every photo of the session — to JPEG or PNG
files in a folder chosen once and remembered.

## 2. Scope
**In**
- Export of the selected photo, or of all the photos of the session.
- JPEG with a quality setting, and PNG.
- Full resolution, or a chosen long edge.
- A destination folder asked the first time and remembered between launches.
- Progress, cancellation and a result message.

**Out**
- TIFF, 16-bit output, HEIF, AVIF.
- Color profiles: files are sRGB, untagged (ADR 0004).
- Metadata (EXIF, IPTC) copied to the exported file.
- Watermark, output sharpening, renaming templates, export presets.
- Exporting a chosen subset of the session.

## 3. Vocabulary
- **Export** — writing a photo, developed with its edit, as a new image file.
- **Export settings** — format, JPEG quality, size and destination folder.
- **Destination** — the folder exported files are written to.
- **Long edge** — the larger of an image's width and height.

## 4. Behavior

### Starting an export
1. `Cmd+E`, or the "Export…" button, opens the export dialog for the selected
   photo; `Cmd+Shift+E` opens it for all the photos of the session.
2. The dialog says what will be exported: the photo's name, or the number of
   photos.
3. The dialog shows the export settings: format (JPEG or PNG), quality (JPEG
   only, 1 … 100, default 90), size (full resolution, or a long edge in pixels,
   default 2048) and the destination.
4. With no destination yet, the dialog asks for one; Export is disabled until
   a folder is chosen. "Change…" picks another folder at any time.
5. The export settings are remembered between launches.
6. Cancel, or `Esc`, closes the dialog and exports nothing.
7. Export is unavailable with an empty session, and while an export is running.

### What is written
8. A photo is exported as the viewport shows it outside Before: its edit and,
   for a RAW, the base rendering.
9. A photo never opened in this session is exported with the edit of its
   sidecar; a photo whose sidecar cannot be used is exported without edit.
10. At full resolution the file has the photo's size. With a long edge, the
    file's long edge is that value and its aspect ratio the photo's; a photo
    smaller than the long edge is not enlarged. Amended by
    `crop-and-straighten.md` rule 37: the photo is the framed photo.
11. The file is named after the photo, with the format's extension:
    `DSC07070.ARW` → `DSC07070.jpg`. If that name exists in the destination a
    number is appended — `DSC07070-1.jpg` — so that nothing is ever overwritten.
12. The source photo and its sidecar are never modified.

### While it runs
13. The export runs in the background: the window stays usable, photos can be
    browsed and developed.
14. Progress is shown — "Exporting 3 of 12" — with a Cancel button. Cancel stops
    after the photo being written; files already written stay.
15. A photo that cannot be decoded or written is skipped; the others are still
    exported.
16. At the end a message gives the number of photos exported and the
    destination, and the number of photos that failed when there are any.

## 5. Acceptance criteria
- [x] X1 — The exported pixels equal the engine's display render of the
      developed photo, at full resolution (rules 8, 10).
- [x] X2 — Long edge: size and aspect ratio as described, no enlargement
      (rule 10).
- [x] X3 — File naming, and no overwrite of an existing file (rule 11).
- [x] X4 — JPEG and PNG files are written and read back as images of the
      expected size (rule 3).
- [x] X5 — A session export writes one file per photo, uses each photo's
      stored edit, skips a failing photo and reports the counts
      (rules 9, 15, 16).
- [x] X6 — The dialog shows the scope and the settings; Export is disabled
      without a destination; Cancel exports nothing (rules 2–4, 6).
- [x] X7 — Export settings survive a restart (rule 5).
- [x] X8 — Progress and Cancel behave as described; the app keeps producing
      frames during an export (rules 13, 14).
- [x] X9 — In the real app, exporting the developed A7 IV RAW writes a file
      that looks like the viewport (rule 8).

## 6. Test plan
| Criterion | Level | Fixture |
|-----------|-------|---------|
| X1, X2 | U + Eb | synthetic image through the headless engine |
| X3 | U + Eb | temp folder |
| X4 | Eb | `patches.png`, temp folder |
| X5 | Eb | temp folder with fixtures, a sidecar and a broken file |
| X6 | Eu | none |
| X7 | U | settings serialized and read back |
| X8 | U + HV | slow exporter stub; real app |
| X9 | HV | local `DSC07070.ARW` |

The native folder picker is never driven by a test.

## 7. Open questions
None.
