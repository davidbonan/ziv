# ziv — Histogram and shooting data

## 1. Goal
Read a photo's exposure at a glance while developing it: the histogram of the
developed photo at the top of the develop panel, and under it how the photo was
shot.

## 2. Scope
**In**
- The histogram of the selected photo as developed, following every edit.
- The shooting data of the photo: sensitivity, focal length, aperture, shutter
  speed.

**Out**
- One colored shape per channel; clipping warnings painted on the photo.
- The histogram of the part of the photo zoomed on.
- Editing by dragging the histogram; folding or hiding it.
- Camera and lens names, date, exposure compensation, any other metadata.
- Shooting data anywhere else: filmstrip, sidebar, exported files.

## 3. Vocabulary
- **Histogram** — how many pixels of the developed photo sit at each of the 256
  display levels, black on the left, white on the right.
- **Luminance shape** — the histogram of the pixels' luminance.
- **Channels shape** — for each level, the largest of its red, green and blue
  counts: where any channel sits.
- **Clipped level** — pure black or pure white.
- **Shooting data** — the settings the camera wrote in the file: sensitivity
  (ISO), focal length, aperture, shutter speed.

## 4. Behavior

### Histogram
1. The top of the develop panel shows the histogram of the selected photo,
   above the tool bar. It stays there whatever is scrolled, with a mask
   selected or not.
2. It is the histogram of the whole photo as developed, whatever the zoom, in
   the values sent to the screen (`overview.md` §3, display transform).
3. It draws two shapes on the same scale: the channels shape, and over it the
   lighter luminance shape. Luminance is Rec.709 luma of the display values.
4. Heights are relative to the tallest level that is not clipped. A clipped
   level taller than that fills the height: a spike on an edge says pixels are
   clipped without flattening the rest.
5. The histogram follows the edit: adjustments, masks, enhancement intensity,
   undo, redo, paste, reset. While Before is shown it is the histogram of the
   photo without its edit. The veil of a mask overlay never counts.
6. While the photo loads, when it failed or was not found, the histogram's
   place is kept and stays empty.
7. The histogram is not a control: nothing in it is clicked or dragged.
8. Following the edit never slows a slider: one histogram costs well under a
   frame. Measured on a 33 MP RAW, test build: about 4 ms, 2.7 ms of them
   waiting for the GPU.

### Shooting data
9. Under the histogram, one line gives the shooting data of a photo that is
   ready, in this order: "ISO 400", "35 mm", "f/2.8", "1/250 s".
10. They are read from the photo's file, RAW, JPEG, PNG or TIFF, when it
    carries them.
11. A value the file does not carry, or carries as zero, is left out; the
    others keep their order. With no value the line is empty; its place is kept.
12. Focal length is rounded to the millimeter. Aperture has one decimal, none
    when it is zero: "f/8". A speed shorter than 0.3 s is a fraction, "1/250 s";
    another is in seconds with one decimal at most: "0.5 s", "2 s", "30 s".

### Design
13. Greys only: `develop.md` rule 30 holds.

## 5. Acceptance criteria
- [x] H1 — Known pixels give the expected counts at the expected levels, for
      luminance and for channels; heights follow the tallest level that is not
      clipped (rules 3, 4).
- [x] H2 — The histogram of a photo rendered by the engine is that of its
      developed pixels, and moves with the edit (rules 2, 5).
- [x] H3 — The histogram is found by its name and keeps its place without a
      photo; in the real app it sits above the tool bar, with and without a
      mask selected (rules 1, 6, 7).
- [x] H4 — In the real app the histogram changes with an edit and with Before
      (rule 5).
- [x] H5 — The cost of one histogram of a 33 MP photo is measured and printed
      by a test; it asserts no bound, the suite runs in parallel (rule 8).
- [x] H6 — Shooting data are worded as said, values left out included
      (rules 9, 11, 12).
- [x] H7 — Shooting data are read from a RAW, a JPEG, a PNG and a TIFF; a file
      without them gives none (rules 10, 11).
- [x] H8 — The line shows under the histogram, in the panel and in the real
      app (rule 9).
- [x] H9 — Demo: the real app on a series shows the histogram and the shooting
      data of the selected photo as in direction A of the mockup.

## 6. Test plan
| Criterion | Level | Fixture |
|-----------|-------|---------|
| H1 | U | a handful of pixels |
| H2 | Eb | synthetic image on the headless engine, two edits |
| H3 | Eu + HV | histogram of known pixels; one photo with a mask |
| H4 | HV | one photo, an exposure change, Before |
| H5 | Eb | the local 33 MP RAW; skipped when absent |
| H6 | U | none |
| H7 | Eb | `patches_with_shooting_data.{jpg,png,tiff}`, `patches.png`, the local RAW |
| H8 | Eu + HV | shooting data injected; a fixture carrying them |
| H9 | HV | a folder of the fixtures |

## 7. Open questions
None.
