# ziv — Zone masks

## 1. Goal
Mask a zone of the photo without drawing it: the subject, the background, the
sky, or a part — skin, hair, eyes, lips, clothes — of the persons chosen.

## 2. Scope
**In**
- Zone tools in the Masks section: Subject, Background, Sky, People.
- People: the persons of the photo are found; the user picks persons and
  parts; one mask is made per part.
- Models downloaded the first time a zone is used, with progress.
- Zone masks behave like any mask: adjustments, invert, hide, delete, overlay,
  undo, copy and paste, sidecar, export.

**Out**
- Retouching a zone mask with the brush; combining masks.
- Clicking any object to mask it; zones other than those listed (teeth,
  eyebrows, iris, landscape elements).
- Choosing a person by clicking them on the photo.
- Detecting again when the photo's adjustments change: a zone mask is made
  once, from the photo without its edit.
- Choosing where models are stored; removing them from the app.
- Distribution: two of the models are for personal use only (ADR 0007).

## 3. Vocabulary
- **Zone** — what a zone mask covers: subject, sky, or a part of persons.
- **Zone mask** — a mask whose coverage was computed by a model.
- **Detection** — the computation of a zone mask's coverage.
- **Person** — one human found in the photo, numbered from left to right.
- **Part** — skin, hair, eyes, lips or clothes of a person.
- **Model** — a file a detection needs, downloaded once.

## 4. Behavior

### Making a zone mask
1. The Masks section has a second row of tools: Subject, Background, Sky,
   People. They are disabled where the mask tools are (16 masks, photo not
   ready).
2. Subject, Background and Sky start a detection at once. The window stays
   usable; a message says what is being detected.
3. When the detection ends, the mask is added to the list and selected, with
   its overlay shown. Background covers what Subject leaves.
4. A zone mask is named after its zone and rank — "Subject 1", "Background 1",
   "Sky 1", "Skin 1".
5. A detection that finds nothing — no sky in the photo — adds no mask and
   says so.
6. Only one detection runs at a time; the zone tools are disabled meanwhile.
   Selecting another photo abandons the detection.

### People
7. People finds the persons of the photo, then shows a picker: one toggle per
   person — "Person 1", "Person 2"… from left to right, each outlined and
   numbered on the photo — and one toggle per part. While it is open no mask
   is drawn or detected.
8. All persons are chosen at first; no part is. "Create masks" (or `Enter`) is
   enabled once a person and a part are chosen, and makes one mask per chosen
   part, covering that part on every chosen person; the last one is selected.
   Cancel (or `Esc`) makes nothing. Skin leaves out the eyes, the eyebrows,
   the lips and the inside of the mouth.
9. A part found on none of the chosen persons makes no mask and is named in a
   message; the other parts are still made.
10. A photo without any person says so and shows no picker.

### Models
11. The first use of a zone downloads its model: the message shows the
    progress — "Downloading the subject model, 120 of 224 MB". Later uses
    start at once, without network.
12. A download that fails — no network, interrupted, corrupted file — says so;
    no mask is made and nothing is kept of the partial file. The next attempt
    starts again.

### With the rest of the app
13. A zone mask is computed from the photo without its edit, and does not
    change when the adjustments change.
14. A zone mask has no handle and no feather; Invert, hide, delete, overlay
    and its adjustments work as for any mask.
15. Zone masks are saved with the edit, their coverage inside the sidecar, and
    restored without any model; undo, redo, Before, Reset, paste and export
    handle them like the other masks. Such a sidecar is not understood by an
    earlier version of ziv, which leaves it alone (ADR 0003).
16. Pasted onto a photo of another size or framing, a zone mask keeps its
    coverage stretched to the photo: it no longer matches the picture.

## 5. Acceptance criteria
- [x] Z1 — A model is downloaded once, checked, and reused; a failed or
      corrupted download leaves nothing behind and is reported (rules 11, 12).
- [x] Z2 — Subject detection on a photo with an obvious subject covers it and
      leaves the background (rule 2).
- [x] Z3 — A zone mask renders its adjustments by its coverage, in the
      viewport and in an export of another size (rules 14, 15).
- [x] Z4 — A zone mask survives closing and reopening the photo with no model
      present (rule 15).
- [x] Z5 — The zone tools start a detection, show its message, add and select
      the mask; they are disabled during a detection and at 16 masks
      (rules 1–4, 6).
- [x] Z6 — Background covers what Subject leaves (rule 3).
- [x] Z7 — Sky covers the sky of a landscape; a photo without sky makes no
      mask and says so (rules 2, 5).
- [x] Z8 — People lists the persons from left to right; Create makes one mask
      per chosen part for the chosen persons; a part not found is named
      (rules 7–10).
- [x] Z9 — Parts of a person cover what they name on a full-length person and
      on a portrait (rule 8).
- [x] Z10 — In the real app, on a photo of several persons: mask the subject,
      the sky, the skin of one person; the window keeps producing frames
      during detection; reopen and export (rules 2, 6, 15).

## 6. Test plan
| Criterion | Level | Fixture |
|-----------|-------|---------|
| Z1 | U + Eb | model source stub; temp folder |
| Z2, Z7, Z9 | Eb | local models and local photos (skipped when absent) |
| Z3 | U + G | synthetic coverage through the headless engine |
| Z4 | U + Eb | temp folder |
| Z5, Z6, Z8 | U + Eu | detector stub |
| Z10 | HV | local models, a local photo of several persons |

Tests never download a model: those that need one read it from a local folder
and are skipped when it is absent, like the RAW sample.

## 7. Open questions
None.
