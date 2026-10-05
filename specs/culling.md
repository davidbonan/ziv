# ziv — Culling

## 1. Goal
Sort a series inside ziv instead of Adobe Bridge: look at its photos in a grid
beside a fast preview, rate them, reject them, remove or trash the bad ones,
then develop only the ones worth it.

## 2. Scope
**In**
- A Cull mode beside the Develop mode: series sidebar, grid, preview above the
  photo details; a resizable separator between grid and preview.
- A preview of the photo as ziv develops it without edit; the picture a RAW
  embeds stands in for it while the RAW is developed.
- Ratings from 0 to 5 stars and a rejected mark, set by keyboard, on one or
  several photos.
- A filter per series, "N stars or more", shared by the grid and the filmstrip.
- Removing photos from a series; moving photos to the macOS Trash.

**Out**
- Reading or writing XMP: ratings made in Bridge are not read, ratings made in
  ziv are not seen by Bridge or Lightroom.
- Color labels, keywords, collections, stacks, a compare view of two photos.
- Filters on anything but the rating: date, camera, lens, edited or not.
- Sorting the grid by anything but the series order.
- A thumbnail size setting; a grid in Develop mode.
- Moving the selection by itself after a mark.
- Renaming, moving or copying files; importing from a memory card.
- Undo of a mark, of a removal or of a move to the Trash.
- Showing edits in the preview of Cull mode.
- Developing ahead more than the two photos next to the selected one.
- Edited or rating figures in the sidebar other than today's edited count.

## 3. Vocabulary
- **Cull mode** — the window laid out to sort a series: grid and preview.
- **Develop mode** — the window as `shell.md` lays it out: viewport, filmstrip,
  develop panel.
- **Grid** — the photos of the open series as rows of thumbnails.
- **Preview** — the area of Cull mode showing the selected photo.
- **Embedded preview** — the picture the camera stored inside a RAW file. For
  a JPEG, PNG or TIFF, the image itself.
- **Photo details** — the lines under the preview describing the selected
  photo and its file.
- **Selected photo** — as in `import.md`: the one photo shown large.
- **Selection** — the photos a mark, a removal or a move to the Trash acts on.
  It always holds the selected photo; in Develop mode it is that photo alone.
- **Rating** — 0 to 5 stars on a photo; 0 is unrated.
- **Rejected photo** — a photo carrying the rejected mark.
- **Mark** — a rating or the rejected mark.
- **Series filter** — the lowest rating a photo needs to be shown, or All.
- **Shown photo** — a photo of the open series that passes its series filter.

## 4. Behavior

### Modes
1. The window is in Cull mode or in Develop mode. The top bar carries a switch
   between the two, after the names; `G` goes to Cull, `D` to Develop.
2. Develop mode is the window of `shell.md` rule 7. Cull mode keeps the top bar
   and the series sidebar; then come the grid and, on its right, the preview
   above the photo details. It has no filmstrip and no develop panel.
3. Both modes share the open series, the selected photo and the series filter:
   switching changes none of them.
4. The mode is remembered between launches. Importing a new series switches to
   Cull mode.
5. In Cull mode, Before is disabled, Export… works as in Develop mode, and the
   zoom readout is that of the preview.
6. With no open series, Cull mode shows the empty state of `import.md` rule 8
   in place of the grid and the preview.
7. A key of this spec does nothing while a text field has the focus.

### Grid
8. The grid shows the shown photos of the open series, in series order, in
   rows that fill its width, scrolling vertically.
9. A cell shows the thumbnail (`catalog.md` rules 16–19), the file name, the
   rating, the rejected mark and the edited marker (`shell.md` rule 9). A
   missing photo is marked as not found.
10. Clicking a cell selects that photo. The selected photo is highlighted and
    kept visible.
11. `←` / `→` select the previous / next shown photo, `↑` / `↓` the one in the
    row above / below; they stop at the ends.
12. `Cmd`+click adds a photo to the selection or takes it out. `Shift`+click
    selects every shown photo from the selected photo to the clicked one, which
    becomes the selected photo. `Cmd+A` selects every shown photo.
13. A plain click or an arrow key brings the selection back to one photo.
14. A double click on a cell, or `Enter`, switches to Develop mode on that
    photo.
15. The separator between the grid and the preview is dragged to resize them.
    Each keeps a minimum width. The position is remembered between launches.

### Preview
16. The preview shows the selected photo as Develop mode shows it without
    edit, at fit, upright: edits are not applied.
17. Until a RAW is developed, the picture it embeds is shown in its place; it
    appears without waiting for the RAW to be developed.
18. The developed RAW then replaces it, the same part of the photo staying on
    screen. A RAW that embeds no picture is shown once developed; one that
    fails to develop keeps its embedded picture.
19. Zoom and pan follow `import.md` rules 17–23; 100 % is one screen pixel for
    one pixel of the photo once developed.
45. The shown photos just after and just before the selected one are developed
    ahead, the one after first: selecting one of them shows it developed at
    once, without its embedded picture. The photo that was selected stays
    developed while it is next to the new one. No other photo is kept.
20. While it loads, when it failed or when the photo is missing, the preview
    shows what the viewport shows (`import.md` rules 15, 16; `catalog.md`
    rule 21).

### Photo details
21. Under the preview, the photo details give, for the selected photo: the file
    name, the rating and the rejected mark, the shooting data line
    (`histogram.md` rules 9–12), the size of the photo in pixels
    "7008 × 4672", the date it was shot "14 Sep 2026, 18:42" and the size of
    its file "34.1 MB".
22. A value the file does not carry is left out. The size in pixels is that of
    the photo, not of its embedded preview.
23. With several photos in the selection, a line says how many: "5 selected".

### Marks
24. A photo has a rating from 0 to 5; a photo never rated is at 0. `0` to `5`
    give that rating to every photo of the selection. Clicking a star of the
    photo details does the same.
25. `X` rejects every photo of the selection; when all of them are already
    rejected it clears the mark instead. A rejected photo keeps its rating.
    Amended by `crop-and-straighten.md` rule 17: not in crop mode.
26. Marks work in both modes. The thumbnails of the filmstrip show the rating
    and the rejected mark as the cells of the grid do.
27. A mark never moves the selection.
28. A mark is written when it is made and is found again at the next launch. It
    belongs to the photo: a photo present in two series has one rating.
29. A mark is not an edit: it does not make a photo an edited photo, and Undo,
    Redo, Reset, Copy and Paste leave it alone.

### Series filter
30. Each series has a filter: **All**, or **N stars or more** with N from 1
    to 5. The top bar shows it and changes it, in both modes.
31. With All, every photo of the series is shown; rejected photos are dimmed.
    With N stars or more, the photos rated N or more and not rejected are
    shown.
32. The filter applies to the grid and to the filmstrip alike. The position of
    the filmstrip (`shell.md` rule 8) counts shown photos: "3 / 12".
33. A photo that stops being shown — by a mark or by a change of the filter —
    leaves the grid and the filmstrip at once. When it was the selected photo,
    the next shown photo is selected, else the previous one, else none.
34. When no photo is shown, the grid in Cull mode and the viewport in Develop
    mode say "No photo at N stars or more" with a **Show all** button, which
    sets the filter to All.
35. The filter of a series is remembered between launches. A new series starts
    at All.
36. The filter changes nothing in the sidebar: cover, edited count and total
    are those of the whole series.
37. The export of all the photos of the session (`export.md` rule 1) writes the
    shown photos; the dialog gives their number.

### Removing and trashing
38. In Cull mode, `Delete` or **Remove from series** in the menu of a cell
    takes the selection out of the open series at once, without confirmation.
    No file is touched. A photo removed from a folder series does not come
    back when its folder is imported again (`catalog.md` rule 4); removing the
    series and importing the folder again brings every photo back.
39. In Cull mode, `Cmd+Delete` or **Move to Trash…** in the menu of a cell asks
    for a confirmation that says how many photos. Confirmed, each photo's file
    and the files ziv keeps beside it (sidecar, enhancement file) go to the
    macOS Trash, and the photo leaves the series.
40. **Trash rejected…**, above the grid, does what rule 39 does for every
    rejected photo of the open series, whatever the filter. It is disabled when
    the series has no rejected photo.
41. After a removal or a move to the Trash, the next shown photo is selected,
    else the previous one, else none.
42. A series left without any photo leaves the catalog as by Remove
    (`catalog.md` rules 29, 30).
43. A file that cannot be moved to the Trash stays where it is and its photo
    stays in the series; a message says how many photos could not be moved.
44. A photo moved to the Trash that belongs to another series is a missing
    photo there (`catalog.md` rule 21).

## 5. Acceptance criteria
- [x] K1 — The top bar switch, `G` and `D` change the mode; Cull mode shows the
      sidebar, the grid, the preview and the photo details, and neither
      filmstrip nor develop panel; open series and selected photo are kept
      (rules 1–3, 5, 6).
- [ ] K2 — The mode survives a relaunch; an import switches to Cull mode
      (rule 4).
- [x] K3 — The grid shows the shown photos in series order with name and
      markers; click and the four arrows select, stopping at the ends
      (rules 8–11).
- [x] K4 — `Cmd`+click, `Shift`+click and `Cmd+A` build the selection; a plain
      click or an arrow brings it back to one photo (rules 12, 13).
- [x] K5 — A double click or `Enter` opens the photo in Develop mode (rule 14).
- [ ] K6 — The separator resizes grid and preview within their minimum widths
      and its position survives a relaunch (rule 15).
- [x] K7 — The picture a RAW embeds is read without decoding the sensor data
      and shown first, upright; the developed RAW then replaces it keeping the
      part of the photo on screen; a JPEG, or a RAW embedding no picture, is
      shown once decoded (rules 16–18).
- [x] K8 — The preview zooms and pans, and shows its loading, failed and not
      found states (rules 19, 20).
- [x] K9 — The photo details give name, marks, shooting data, pixel size, date
      and file size, leave out what the file lacks, and count a selection of
      several photos (rules 21–23).
- [x] K10 — `0`–`5`, a clicked star and `X` mark the whole selection, in both
      modes, without moving it and not while a text field has the focus; the
      grid and the filmstrip show the marks (rules 7, 24–27).
- [x] K11 — Marks survive a relaunch, are shared by two series holding the
      same photo, and are untouched by Undo, Reset and Paste; a rated photo
      without edit is not an edited photo (rules 28, 29).
- [x] K12 — The filter set in the top bar shows the photos rated N or more and
      not rejected, in the grid and in the filmstrip, with the position counting
      shown photos; All dims rejected photos; the sidebar does not change; a
      session export writes the shown photos
      (rules 30–32, 36, 37).
- [x] K13 — A photo that stops being shown leaves at once and the selection
      moves to its neighbor; an empty result shows the message and Show all
      (rules 33, 34).
- [x] K14 — The filter of each series survives a relaunch; a new series starts
      at All (rule 35).
- [x] K15 — Remove from series takes the selection out without touching a file
      and selects the neighbor; the photo stays out when its folder is imported
      again; a series emptied leaves the catalog
      (rules 38, 41, 42).
- [x] K16 — Move to Trash asks, then moves the photo, its sidecar and its
      enhancement file to the Trash and takes the photo out of the series;
      cancelling changes nothing; Trash rejected… does it for the rejected
      photos (rules 39, 40).
- [x] K17 — A file that cannot be trashed stays in the series and is counted in
      a message; a trashed photo is missing in its other series (rules 43, 44).
- [x] K19 — Selecting the photo after or before the selected one shows it
      developed at once; photos that are no longer next to it are let go, and
      one that stopped being wanted before its turn is not developed (rule 45).
- [x] K18 — Demo: import a folder, rate and reject in Cull mode, trash the
      rejected, filter at 2 stars or more, switch to Develop and find only the
      kept photos in the filmstrip; quit, relaunch and find marks, filter and
      mode again (rules 1–44).

## 6. Test plan
| Criterion | Level | Fixture |
|-----------|-------|---------|
| K1 | Eu + HV | catalog of one series injected through the constructor seam |
| K2 | U + HV | app built twice on the same data folder; import injected through the intent |
| K3, K4, K5 | U + Eu | series of seven stub photos, grid three cells wide |
| K6 | U + Eu + HV | none |
| K7 | U + Eb + HV | the author's `DSC07070.ARW` (local), and the same converted to a DNG embedding no picture; fixture JPEG; a zoomed view on a picture replaced by a bigger one |
| K8 | Eu | preview states injected |
| K9 | U + Eb + Eu | fixture JPEG with EXIF; PNG without |
| K10 | U + Eu | series of three stub photos; a focused text field |
| K11 | U + Eb | temp data folder; two series over the same temp files, one with a sidecar |
| K12, K13 | U + Eu | series of five stub photos rated 0, 1, 2, 3 and one rejected |
| K14 | U + Eb | catalog in a temp data folder |
| K15 | U + Eb + Eu | temp folder; series of three photos, then of one |
| K16 | U + Eb + Eu | temp folder with a photo, its sidecar and its enhancement file; Trash behind a port, a temp folder standing for it; confirmation injected through the intent |
| K17 | U + Eb | Trash stub that refuses one file; two series over the same files |
| K19 | U + HV | stub loader held by the test; a folder of two RAW of the fixtures |
| K18 | HV | a folder of the fixtures, app built twice on the same data folder, Trash stub |

The macOS Trash is never driven by a test: a port stands for it, as the native
picker and the Finder do (`specs/testing.md` §7).

## 7. Open questions
None.
