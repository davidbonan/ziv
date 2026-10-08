# ziv — AI presets

## 1. Goal
Retouch a zone of the photo in one click: a preset finds its zone with the
zone models, masks it and sets the adjustments of the mask — whiter teeth, an
enhanced sky, bright eyes, glowing skin, a subject that stands out.

## 2. Scope
**In**
- A Presets section in the develop panel with nine presets in two groups.
  Portrait: Whiter teeth, Bright eyes, Skin glow, Lip color, Hair shine.
  Scene: Enhanced sky, Dramatic sky, Golden sky, Subject pop.
- A new zone, teeth, used by Whiter teeth.
- An applied preset: the masks a preset made on a photo, kept together in the
  masks list, with one Intensity for all of them.
- Applied presets saved with the edit, undone and redone, copied and pasted,
  exported.

**Out**
- Presets without a model (a named edit applied to the whole photo).
- Presets made, changed, renamed or removed by the user; import of presets.
- Applying a preset to several photos at once, or from Cull mode.
- Choosing the persons a preset applies to: it takes every person found.
- Teeth as a part in the People picker.
- A preview of a preset before it is applied.
- Grouping masks by hand; moving a mask into or out of an applied preset.
- Detecting again when the photo or its edit changes.

## 3. Vocabulary
- **Preset** — a named recipe: the zones it masks and, for each, the local
  adjustments it sets.
- **Applied preset** — what a preset made on one photo: its masks and its
  intensity.
- **Intensity** — how much of an applied preset's effect is kept, from 0
  (none) to 100 (all).
- **Teeth** — the zone covering the teeth of the persons of the photo.

## 4. Behavior

### Presets section
1. The develop panel has a "Presets" section above Masks, folding like the
   others, with one button per preset, two a row, under the name of its
   group. Portrait: Whiter teeth, Bright eyes, Skin glow, Lip color, Hair
   shine. Scene: Enhanced sky, Dramatic sky, Golden sky, Subject pop.
2. The buttons are disabled where the zone tools are (photo not ready, a
   detection running, crop mode), and a preset is disabled when the photo has
   fewer masks left under the limit of 16 than the preset makes.

### Applying a preset
3. Clicking a preset starts the detection of its zones at once. The window
   stays usable; a message says what is being detected.
4. When the detection ends, the applied preset is added to the masks list and
   selected: a row named after the preset and its rank — "Whiter teeth 1" —
   with its masks listed under it, named like any zone mask — "Teeth 1".
5. The masks of an applied preset carry the preset's adjustments:

   | Preset | Zone | Adjustments |
   |--------|------|-------------|
   | Whiter teeth | Teeth, every person | Exposure +0.40, Saturation −60 |
   | Enhanced sky | Sky | Exposure −0.20, Contrast +20, Highlights −40, Vibrance +30 |
   | Bright eyes | Eyes, every person | Exposure +0.25, Contrast +15, Whites +10 |
   | Subject pop | Subject | Exposure +0.20, Contrast +10 |
   | Subject pop | Background | Exposure −0.30, Saturation −15 |
   | Skin glow | Skin, every person | Exposure +0.15, Contrast −10 |
   | Lip color | Lips, every person | Vibrance +20, Saturation +20 |
   | Hair shine | Hair, every person | Contrast +15, Whites +15 |
   | Dramatic sky | Sky | Exposure −0.60, Contrast +40, Highlights −60, Blacks −15 |
   | Golden sky | Sky | Temp +30, Tint +5, Exposure −0.10, Vibrance +25, Saturation +10 |

6. The Portrait presets take every person found in the photo, without a
   picker, in one mask. Teeth are what is bright and not red of the inside
   of a mouth: the tongue, the gums and the dark behind the teeth are left.
7. A preset whose zone is not found — no sky, no person, no teeth showing —
   adds nothing and says so. Subject pop adds its two masks or none.
8. Only one detection runs at a time, zone tools and presets together.
   Selecting another photo abandons it.
9. The same preset can be applied again: it adds another applied preset, whose
   effect adds up with the first.
10. Models are downloaded at first use, with the progress and the failures of
    the zone masks (`specs/zone-masks.md` rules 11, 12).

### Intensity
11. While an applied preset is selected the panel shows its name and one
    slider, Intensity, 0 … 100, default 100, instead of the photo's
    adjustments; "Done", `Esc`, or clicking it again returns to the photo's.
12. Intensity scales the effect of every mask of the applied preset: at 100
    each mask renders its adjustments in full, at 0 the photo is as without
    the applied preset, in between in proportion.

### Masks of an applied preset
13. A mask of an applied preset is a zone mask: selecting it shows its
    adjustments, which can be changed; Invert, hide, overlay work as for any
    mask. Its Intensity is the one of its applied preset.
14. Deleting a mask of an applied preset removes that mask only; the applied
    preset leaves the list with its last mask. Deleting the selected applied
    preset — `Delete`, `Backspace` or its remove button — removes all its
    masks.
15. A preset's zones are detected on the whole picture, without its edit and
    whatever its framing, like any zone mask.

### With the rest of the app
16. Applied presets are part of the edit: saved in the sidecar with their
    masks and intensity, restored without any model, written into exports.
    Such a sidecar is not understood by an earlier version of ziv, which
    leaves it alone (ADR 0003).
17. Applying a preset is one step of undo and redo; so is a change of
    Intensity, and the deletion of an applied preset.
18. Before shows the photo without its applied presets. Reset removes them.
19. Paste copies the applied presets with the rest of the edit; their masks
    keep their coverage stretched to the photo, like any zone mask
    (`specs/zone-masks.md` rule 16).

## 5. Acceptance criteria
- [x] R1 — A preset button starts its detection, shows its message, then adds
      and selects an applied preset named after it, with its masks under it
      carrying the adjustments of the table (rules 1, 3–5).
- [x] R2 — Presets are disabled during a detection, when the photo is not
      ready, and when the masks left are fewer than the preset makes
      (rules 2, 8).
- [x] R3 — Intensity 100 renders each mask's adjustments in full, 0 renders
      the photo without the applied preset, 50 half of the effect; in the
      viewport and in an export (rule 12).
- [x] R4 — A selected applied preset shows its name and Intensity only, and
      returns to the photo's adjustments on Done (rule 11).
- [x] R5 — Subject pop makes a Subject and a Background mask, or nothing when
      no subject is found (rules 5, 7).
- [x] R6 — Bright eyes covers the eyes of every person of a photo of several
      persons in one mask; a photo without person adds nothing and says so
      (rules 6, 7).
- [x] R7 — The teeth zone covers the teeth of a smiling portrait and leaves
      the lips and the skin; a closed mouth makes no mask and says so
      (rules 5–7).
- [x] R8 — Deleting a mask of an applied preset leaves the others; the last
      one takes the applied preset with it; deleting the applied preset
      removes all its masks (rule 14).
- [x] R9 — Applied presets survive closing and reopening the photo with no
      model present; an earlier sidecar still opens (rule 16).
- [x] R10 — Undo, redo, Before, Reset and paste handle applied presets as
      described (rules 17–19).
- [x] R12 — The section shows the presets two a row under "Portrait" and
      "Scene"; Skin glow, Lip color and Hair shine mask the skin, the lips
      and the hair of every person, Dramatic sky and Golden sky the sky, with
      the adjustments of the table (rules 1, 5, 6).
- [x] R11 — In the real app, on a portrait and on a landscape: Whiter teeth,
      Bright eyes, Enhanced sky, Subject pop; dose one with Intensity; the
      window keeps producing frames during detection; reopen and export
      (rules 3, 4, 12, 16).

## 6. Test plan
| Criterion | Level | Fixture |
|-----------|-------|---------|
| R1, R2 | U + Eu | detector stub |
| R3 | U + G | synthetic coverage through the headless engine |
| R4 | Eu | none |
| R5 | U + Eu | detector stub |
| R6, R7 | U + Eb | local models and local photos (skipped when absent) |
| R8 | U + Eu | none |
| R9 | U + Eb | temp folder, a sidecar of the version before |
| R10 | U + Eu | none |
| R11 | HV | local models, a local portrait and a local landscape |
| R12 | U + Eu + HV | none; local models and photos for HV |

Tests never download a model (`specs/zone-masks.md` §6).

## 7. Open questions
None.
