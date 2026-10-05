# ziv — Series catalog

## 1. Goal
Quit ziv, launch it again and carry on: every import stays listed as a series in
a sidebar, and the app reopens on the series and the photo last looked at.

## 2. Scope
**In**
- A catalog of series that survives a quit (ADR 0011).
- A sidebar listing the series; selecting one opens it.
- Importing adds a series instead of replacing the open photos.
- Importing a folder again brings its new photos into its series.
- The launch reopens the last series on its last photo.
- Missing photos and missing series; Locate for a series.
- Removing a series from the catalog, renaming it, showing it in the Finder.
- A thumbnail cache, so a reopened series shows without decoding its photos.

**Out**
- Series made by hand: mixing photos of several series, moving a photo to
  another series.
- Search, sub-folder recursion. Ratings, the rejected mark, the filter, the
  grid and removing one photo from a series came later: `culling.md`.
- Locating one missing photo of a series that still has others.
- Cleaning the thumbnail cache.
- Remembering the zoom, the pan or the undo history between launches.
- A catalog shared between machines.
- The visual overhaul of the rest of the window (top bar, histogram, tool
  icons, folding sections): a later milestone. The sidebar follows direction A
  of `specs/mockups/shell.html`; nothing else of the mockup is built here.

## 3. Vocabulary
- **Catalog** — the series ziv remembers on this machine. It holds no edit.
- **Series** — what one import brought in, with a name and an import date.
- **Folder series** — a series imported as one folder; it remembers that folder.
- **Photo set** — a series imported as anything else: several files, or a mix.
- **Import** — opening photos through the picker or a drop (`import.md` rules
  1–3); it now ends in the catalog.
- **Open series** — the series whose photos are in the filmstrip. The session
  (`import.md`) is the open series and its selected photo.
- **Missing photo** — a photo of a series whose file is no longer at its path.
- **Missing series** — a series whose every photo is missing.
- **Edited photo** — a photo that has a sidecar (`develop.md`).
- **Series cover** — the first four photos of a series, as a small mosaic.
- **Thumbnail cache** — thumbnails kept on disk between launches.

## 4. Behavior

### Importing
1. An import of exactly one folder makes a folder series named after the
   folder. Any other import makes a photo set named after its size and its
   date: "23 photos, 14 Sep".
2. The photos of a series are those the import found (`import.md` rules 3, 7),
   in natural name order. The list is frozen: a file added to the folder later
   is not part of the series.
3. A new series is added to the catalog, becomes the open series, and its first
   photo is selected. The series open before stays in the catalog.
4. Importing a folder that already is a folder series opens that series and
   adds the photos of the folder it does not have yet. Its name, its import
   date and its place in the list do not change; its selected photo is kept.
5. An import that finds no photo adds nothing and shows the message of
   `import.md` rule 7.

### Sidebar
6. A sidebar on the left of the window lists the series, the most recently
   imported first. It has an "Import…" button, which does what `Cmd+O` does.
7. A row shows the series cover, the name, the count "37 of 148 edited" and the
   import date.
8. Clicking a row opens that series. The open series is highlighted.
9. Opening a series selects the photo that was selected the last time it was
   open, the first one the first time.
10. A button and `Cmd+L` hide and show the sidebar; hidden or shown is
    remembered between launches. Hiding it gives its width to the viewport.
11. With no series, the sidebar is empty and the viewport shows the empty state
    of `import.md` rule 8.
12. The edited count follows the edits: it changes when a photo of the open
    series gets its first edit or goes back to no edit.

### Between launches
13. Every change to the catalog is written when it happens: an import, a
    rename, a removal, a relocation, another series opened, another photo
    selected.
14. At launch the series are listed as they were, the series open at the last
    quit is open, and its selected photo is selected.
15. Edits come back from the sidecars as they already do (`develop.md` rules
    18–22): the catalog adds nothing to them.

### Thumbnails
16. A thumbnail decoded for the filmstrip or a series cover is kept in the
    thumbnail cache.
17. A thumbnail found in the cache is shown without decoding the photo.
18. A photo whose file changed since its thumbnail was kept is decoded again.
19. A thumbnail shows the photo as shot, with or without the cache.

### Missing photos
20. Whether a photo is missing is checked at launch and when its series is
    opened.
21. A missing photo keeps its place in the filmstrip, marked as not found;
    selecting it shows its file name and "not found" in the viewport. It
    counts in the series' total.
22. A missing series is dimmed in the sidebar and says "Folder not found" in
    place of its count. It can still be opened.
23. "Locate…" on a missing series asks for a folder. Every photo of the series
    whose file name is in that folder now points there; the others stay
    missing. A folder series remembers the new folder.
24. A folder that holds none of the series' photos changes nothing and says so.
25. Sidecars are found again with their photos: they sit beside them.

### Managing a series
26. A menu on a row offers Rename, Show in Finder, Locate… (missing series
    only) and Remove.
27. Rename edits the name in place; `Enter` confirms, `Esc` cancels, an empty
    name cancels. A renamed folder series keeps its name when its folder is
    imported again.
28. Show in Finder opens the folder of a folder series, or the folder of the
    first photo of a photo set. It is disabled on a missing series.
29. Remove takes the series out of the catalog at once, without confirmation.
    No photo, sidecar or enhancement file is touched: importing the same photos
    again brings the series back with its edits.
30. Removing the open series opens the one below it, else the one above it,
    else leaves the empty state.

### Failures
31. A catalog that cannot be read, or that a newer ziv wrote, is reported by a
    message naming the file. The app starts with no series, and the file is
    never overwritten: series imported in that run are not remembered, and the
    message says so.
32. A catalog that cannot be written is reported; the app keeps working, and
    the next change tries again.
33. A thumbnail cache that cannot be read or written is never reported:
    thumbnails are decoded from the photos.

## 5. Acceptance criteria
- [x] C1 — A folder import makes a series named after the folder; an import of
      files makes one named by size and date; photos in natural order
      (rules 1, 2).
- [x] C2 — An import adds a series, opens it on its first photo, and keeps the
      earlier series (rule 3).
- [x] C3 — Importing a folder again adds only its new photos to its series and
      keeps name, date, place and selected photo (rule 4).
- [x] C4 — A catalog written and read back gives the same series, open series
      and selected photos (rules 13, 14).
- [x] C5 — The app relaunched on an existing catalog shows the sidebar with its
      series, the last series open and its last photo selected, edits included
      (rules 14, 15).
- [x] C6 — The sidebar lists the series most recent first, each with cover,
      name, edited count and date; clicking a row opens the series on its
      remembered photo (rules 6–9).
- [x] C7 — The sidebar hides and shows by button and by `Cmd+L`, and the choice
      survives a relaunch (rule 10).
- [x] C8 — The edited count is the number of photos with a sidecar and follows
      a first edit and a reset (rules 7, 12).
- [x] C9 — A second launch shows the thumbnails of a series without decoding
      its photos; a photo changed on disk is decoded again (rules 16–18).
- [x] C10 — A missing photo is marked in the filmstrip and in the viewport; a
      series with every photo missing is dimmed and says "Folder not found"
      (rules 20–22).
- [x] C11 — Locate on a moved folder brings the series back with its edits;
      a folder without its photos changes nothing and says so (rules 23–25).
- [x] C12 — Rename, with `Enter`, `Esc` and an empty name; the name survives a
      re-import of the folder (rule 27).
- [x] C13 — Remove takes the series out, leaves photos and sidecars on disk,
      and opens the neighbor or the empty state (rules 29, 30).
- [x] C14 — An unreadable or newer catalog is reported and left untouched; a
      write failure is reported and retried at the next change (rules 31, 32).
- [x] C15 — Demo: import a folder and a set of files, edit a photo, quit,
      relaunch and find both series, the open one, the photo and its edit
      (rules 3, 14, 15).

## 6. Test plan
| Criterion | Level | Fixture |
|-----------|-------|---------|
| C1, C2, C3 | U + Eb | temp folder with empty files of mixed extensions |
| C4 | U + Eb | catalog in a temp data folder |
| C5 | HV | catalog and photos injected through the constructor seam |
| C6 | Eu | catalog of three stub series |
| C7 | Eu + HV | none |
| C8 | U + Eb | temp folder with photos, some with a sidecar |
| C9 | U + Eb | counting decoder stub, temp cache folder, a photo rewritten on disk |
| C10 | U + Eu | series whose files were deleted |
| C11 | Eb + Eu | temp folder moved; chosen folder injected through the intent |
| C12, C13 | U + Eu | catalog of three stub series; temp folder for the files left on disk |
| C14 | U + Eb + Eu | malformed catalog, catalog with a higher version, read-only data folder |
| C15 | HV | two temp imports, app built twice on the same data folder |

The native picker and the Finder are never driven by a test
(`specs/testing.md` §7): the chosen folder and the Finder request are intents.

## 7. Open questions
None.
