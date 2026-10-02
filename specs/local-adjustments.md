# ziv — Local adjustments

## 1. Goal
Develop a part of the photo differently from the rest: draw a mask — a
gradient, a shape or brush strokes — and move, for that mask only, the same
adjustments as the develop panel.

## 2. Scope
**In**
- Five mask types: linear gradient, radial gradient, rectangle, polygon, brush.
- Per mask: the nine adjustments of the develop panel, applied on top of the
  photo's edit; invert; hide; delete.
- A red overlay showing what a mask covers.
- Masks stored in the sidecar, undone and redone, copied and pasted, exported.

**Out**
- Combining masks (add, subtract, intersect): each mask stands alone.
- Zone masks made by segmentation (M5); luminance and color range masks.
- Renaming, reordering, duplicating a mask.
- A greyscale view of the mask.
- Pressure-sensitive brush (tablet), auto-mask (edge-aware brush).
- Local adjustments other than the nine of the panel (sharpness, noise…).

## 3. Vocabulary
- **Mask** — a region of the photo, its own adjustments, and whether it is
  inverted and visible.
- **Coverage** — how much a mask applies at a point, from 0 (not at all) to 1
  (fully).
- **Feather** — how progressively the coverage of a mask falls at its edge.
- **Linear gradient** — full coverage on one side of a band, none on the other,
  progressive across the band.
- **Radial gradient** — an ellipse covered inside, feathered at its edge.
- **Rectangle**, **polygon** — a shape covered inside, feathered at its edge.
- **Brush mask** — the strokes painted on the photo, minus the erased ones.
- **Stroke** — one press-drag-release of the brush, with its size, feather and
  flow.
- **Overlay** — the red veil drawn where the selected mask covers the photo.
- **Local adjustments** — the adjustments of one mask.

## 4. Behavior

### Masks of a photo
1. The develop panel has a "Masks" section: one button per mask type, and the
   list of the photo's masks. A mask is named after its type and rank —
   "Linear gradient 1", "Brush 2".
2. A photo holds at most 16 masks; beyond, the buttons are disabled.
3. Clicking a mask in the list selects it. While a mask is selected the panel
   shows that mask's adjustments under its name, instead of the photo's;
   "Done", `Esc`, or clicking the selected mask again returns to the photo's.
4. Each mask has an eye button hiding it: a hidden mask changes nothing in the
   photo and stays in the list. `Delete` or `Backspace`, or the mask's remove
   button, deletes the selected mask.
5. "Invert" swaps what the selected mask covers and what it leaves.

### Local adjustments
6. A mask carries the nine adjustments of the develop panel, with the same
   names, ranges and defaults. Temp and Tint are relative (−100 … +100) on
   every photo, RAW included.
7. A mask's adjustments apply on top of the photo's edit, in proportion to the
   coverage: at coverage 1 the result is what the same slider would give
   globally, at coverage 0 the photo is untouched.
8. Where masks overlap, their effects add up, in the order the masks were
   created.
9. A mask with every adjustment at its default changes nothing.

### Drawing a mask
10. A mask type button arms that tool: the next drag on the photo draws the
    mask, which is then added to the list and selected. `Esc` disarms.
11. **Linear gradient**: the drag goes from full coverage to none. Afterwards
    the two ends are handles that can be dragged, and the middle handle moves
    the whole gradient.
12. **Radial gradient**: the drag starts at the centre and sets the two radii.
    Afterwards: the centre handle moves it, four edge handles resize it, a
    handle outside the ellipse rotates it. Feather 0 … 100, default 50.
13. **Rectangle**: the drag sets two opposite corners. Afterwards: the centre
    handle moves it, the corner handles resize it. Feather 0 … 100, default 0.
14. **Polygon**: each click adds a corner; clicking the first corner, a
    double-click or `Enter` closes it (three corners at least). Afterwards each
    corner is a handle, the centre handle moves it. Feather 0 … 100, default 0.
15. **Brush**: every drag on the photo adds a stroke to the selected brush
    mask. Size, feather (0 … 100, default 50) and flow (1 … 100, default 100)
    are those of the brush when the stroke is painted. Holding `Alt` erases
    instead. `[` and `]` change the size. A circle follows the pointer and
    shows the size.
16. While a mask is selected or a tool is armed, a drag on the photo belongs to
    the mask; holding `Space` pans instead. Zooming is unchanged.
17. Masks follow the photo: zooming, panning, exporting at another size keep
    them at the same place on the picture.

### Overlay
18. The overlay shows the coverage of the selected mask as a red veil.
19. It is shown as long as the selected mask has all its adjustments at their
    default, so that a new mask is visible while it is drawn; `O`, or the
    "Overlay" button, shows or hides it at any time.
20. Handles are drawn only for the selected mask, and never in an export.

### With the rest of the app
21. Masks are part of the edit: saved in the sidecar, restored when the photo
    is opened again, written into exports.
22. Every change to a mask — drawn, moved, one stroke, an adjustment, invert,
    hide, delete — is one step of undo and redo.
23. Before shows the photo without its masks. Reset removes them.
24. Paste copies the masks along with the rest of the edit, at the same
    relative place in the picture.
25. A sidecar written with masks is not understood by an earlier version of
    ziv, which leaves it alone (ADR 0003).

## 5. Acceptance criteria
- [x] L1 — Coverage of each mask type at known points: full, none, and the
      feathered transition (rules 11–15).
- [x] L2 — A mask's adjustment renders, at coverage 1, what the global
      adjustment renders, and nothing at coverage 0; default local adjustments
      are the identity (rules 7, 9).
- [x] L3 — Overlapping masks add up in creation order; a hidden mask changes
      nothing; Invert swaps the coverage (rules 4, 5, 8).
- [x] L4 — The Masks section adds, lists, selects, hides and deletes masks,
      and stops at 16 (rules 1–4).
- [x] L5 — With a mask selected the panel edits that mask's adjustments, and
      returns to the photo's on Done (rules 3, 6).
- [x] L6 — Each tool draws its mask from pointer input, and its handles edit
      it (rules 10–16).
- [x] L7 — The overlay appears on a mask without adjustment and follows `O`
      (rules 18, 19).
- [x] L8 — Masks survive closing and reopening the photo; an earlier sidecar
      still opens (rules 21, 25).
- [x] L9 — Undo, redo, Before, Reset and paste handle masks as described
      (rules 22–24).
- [x] L10 — The export of a photo with masks equals the display render, at any
      size (rules 17, 21).
- [x] L11 — In the real app: darken the sky of the A7 IV RAW with a linear
      gradient, brighten a face with a radial gradient, paint with the brush;
      the viewport stays fluid while dragging (rules 7, 11, 12, 15).

## 6. Test plan
| Criterion | Level | Fixture |
|-----------|-------|---------|
| L1 | U | none |
| L2, L3 | U + G | synthetic ramp and patches through the headless engine |
| L4, L5, L7 | Eu | none |
| L6 | Eu + HV | none; real app |
| L8 | U + Eb | temp folder, a version 1 sidecar |
| L9 | U + Eu | none |
| L10 | Eb | synthetic image, export at two sizes |
| L11 | HV | local `DSC07070.ARW` |

## 7. Open questions
None.
