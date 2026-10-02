# ziv — Import & viewport

## 1. Goal
Open a folder or a set of photos — RAW included — browse them through a
filmstrip, and look at the selected one in a viewport with zoom and pan.

## 2. Scope
**In**
- Opening photos with the native file picker and by drag-and-drop.
- Formats: RAW files `rawler` decodes, JPEG, PNG, TIFF.
- A filmstrip of thumbnails for the open photos; one selected photo.
- A viewport showing the selected photo: fit, 100 %, free zoom, pan.
- RAW shown neutral: camera white balance and correct colors, no tone curve.

**Out**
- Remembering the open photos between two launches: `catalog.md`.
- Sub-folder recursion, grid view, ratings, flags, metadata panel, deletion.
- Any adjustment, including the base tone curve (M2), and export (M3).
- ICC profiles, wide-gamut display (ADR 0002).
- "Open with" from the Finder and command-line arguments.

## 3. Vocabulary
- **Photo** — one source file on disk that ziv can show. Never written.
- **Session** — the ordered set of photos currently open, plus the selected one.
- **Selected photo** — the photo shown in the viewport. At most one.
- **Filmstrip** — horizontal band of thumbnails, one per photo of the session.
- **Thumbnail** — small picture standing for a photo in the filmstrip.
- **Viewport** — the area showing the selected photo.
- **Fit** — the zoom level at which the whole photo is visible in the viewport;
  a photo smaller than the viewport stays at 100 %, it is never enlarged to fit.
- **Zoom** — screen pixels per photo pixel; 100 % is one for one.
- **Pan** — the offset of the photo in the viewport when it is larger than it.

## 4. Behavior

### Opening
1. `Cmd+O` and an "Open…" button open the native picker; it accepts one folder
   or several files.
2. Dropping files or a folder on the window opens them the same way.
3. A folder contributes its direct children with a supported extension;
   sub-folders are not entered. Hidden files are skipped.
4. Opening **replaces** the session: the previous photos leave the filmstrip.
   They stay reachable as a series (`catalog.md` rule 3).
5. Photos are ordered by file name, natural order (`DSC2` before `DSC10`),
   case-insensitive.
6. After opening, the first photo is selected.
7. Files with an unsupported extension are ignored; if nothing supported is
   left, the session is unchanged and a message says no photo was found.
8. With an empty session, the empty state is shown with the "Open…" button.

### Filmstrip
9. One thumbnail per photo, in session order, scrollable horizontally.
10. Clicking a thumbnail selects that photo. The selected thumbnail is
    highlighted and kept visible.
11. `←` / `→` select the previous / next photo; they stop at the ends.
12. A thumbnail appears as soon as it is ready; until then a placeholder with
    the file name holds its place. The window never freezes while they load.
13. A photo that fails to decode keeps its place, marked as in error.

### Viewport
14. The selected photo is shown at fit, centered, upright: the orientation
    recorded by the camera is applied.
15. While the selected photo is loading, the viewport shows a loading
    indicator; the rest of the window stays usable.
16. If it fails to decode, the viewport shows the file name and the reason.
17. Scrolling or pinching zooms around the pointer: the photo point under the
    pointer stays under it.
18. Zoom is bounded: never smaller than fit, never larger than 800 %.
19. A click toggles between fit and 100 %; going to 100 % centers on the
    clicked point. `Cmd+0` is fit, `Cmd+1` is 100 %.
20. Dragging pans when the photo is larger than the viewport. The photo cannot
    be dragged away: no empty margin on a side where the photo overflows; a
    dimension smaller than the viewport stays centered.
21. At fit, resizing the window keeps the photo at fit.
22. Selecting another photo resets the view to fit.
23. Above 100 % photo pixels are shown as sharp squares, not blurred.

### Rendering
24. A JPEG, PNG or TIFF looks the same as in Preview.app on an sRGB screen.
25. A RAW is shown with the camera's white balance and calibrated colors, no
    tone curve: flatter and darker than the camera JPEG. Clipped highlights are
    neutral, not tinted.

## 5. Acceptance criteria
- [x] A1 — Opening a folder puts its supported photos in the session in natural
      name order, first one selected (rules 3, 5, 6).
- [x] A2 — Opening replaces the previous session (rule 4).
- [x] A3 — Unsupported files are ignored; an all-unsupported selection leaves
      the session unchanged and shows the message (rule 7).
- [x] A4 — The picker and a drop lead to the same session (rules 1, 2).
- [x] A5 — A Sony A7 IV `.ARW` decodes to an upright image of the camera's
      default crop size, with finite values and a neutral grey patch staying
      neutral (rules 14, 25).
- [x] A6 — A JPEG, a PNG and a TIFF decode; an sRGB value round-trips through
      the working space and the display transform within 1/255 (rule 24).
- [x] A7 — EXIF orientations are applied (rule 14).
- [x] A8 — The engine output of a synthetic image matches its golden (rule 24).
- [x] A9 — Clicking a thumbnail and `←` / `→` change the selected photo;
      navigation stops at the ends (rules 10, 11).
- [x] A10 — Loading, ready and error states are each shown by the viewport and
      the filmstrip (rules 12, 13, 15, 16).
- [x] A11 — Decoding runs off the UI thread: the app keeps producing frames
      while a photo loads (rules 12, 15).
- [x] A12 — Zoom keeps the point under the pointer, stays within fit…800 %
      (rules 17, 18).
- [x] A13 — Click toggles fit / 100 %; `Cmd+0` and `Cmd+1` work (rule 19).
- [x] A14 — Pan is clamped as described; resize keeps fit; selection resets the
      view (rules 20, 21, 22).
- [x] A15 — In the real app, a session with photos shows the filmstrip and the
      selected photo in the viewport (rules 9, 14).

## 6. Test plan
| Criterion | Level | Fixture |
|-----------|-------|---------|
| A1, A2, A3 | U + Eb | temp folder with empty files of mixed extensions |
| A4 | Eu | picker result and drop injected through the same intent |
| A5 | Eb | the author's `DSC07070.ARW`, local and not committed (not redistributable) |
| A6 | U + Eb | tiny generated JPEG / PNG / TIFF in `tests/fixtures/` |
| A7 | Eb | tiny JPEGs carrying EXIF orientations |
| A8 | G | synthetic gradient + color patches |
| A9 | Eu | session of three stub photos |
| A10 | Eu | photo states injected |
| A11 | U + HV | slow decoder stub; frames counted while loading |
| A12, A13, A14 | U (view geometry) + Eu (gestures emit the intents) | none |
| A15 | HV | session injected through the constructor seam |

The native picker is never driven by a test (`specs/testing.md` §7).

## 7. Open questions
None.
