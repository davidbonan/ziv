# ziv — Crop and straighten

## 1. Goal
Reframe a photo without leaving ziv: keep a part of the picture, level a tilted
horizon, turn a photo shot sideways, mirror it — as Lightroom's crop tool does,
the source file never being touched.

## 2. Scope
**In**
- A **crop mode** in Develop: the whole picture, a crop frame with handles, the
  outside dimmed, a grid of thirds.
- Aspect ratios: Original, 1 : 1, 4 : 5, 3 : 2, 16 : 9, Custom, Free; a lock;
  landscape / portrait swap.
- A straighten angle of ±45°: by slider, by dragging outside the frame, by a
  level tool drawn on the picture.
- Quarter turns left and right; horizontal and vertical mirror.
- The framing as part of the photo's edit: viewport, masks, histogram, sidecar,
  undo and redo, export.

**Out**
- Perspective and lens corrections: upright, keystone, distortion, vignette.
- Automatic straightening; automatic or content-aware crop; filling empty
  corners.
- Other overlays than the thirds (golden ratio, diagonals, spiral); typed
  ratios (`7 : 5` entered by hand); a crop given in pixels; the size of the
  result shown while cropping.
- Holding a key to constrain a ratio or to resize from the centre.
- Zoom and pan inside crop mode.
- Framing in Cull mode, on thumbnails, grid, filmstrip and series covers: they
  keep showing the source (`develop.md` scope, `culling.md` scope).
- Framing several photos at once; pasting a framing (rule 38).

## 3. Vocabulary
- **Picture** — the whole photo as M1 shows it: upright, uncropped.
- **Framing** — what a photo keeps of its picture and how it is turned: crop
  frame, angle, quarter turns, mirror. Its default keeps the whole picture.
- **Crop frame** — the rectangle of the picture that is kept.
- **Angle** — the straighten angle, −45.0° … +45.0°: how much the picture is
  turned under the frame.
- **Aspect ratio** — the proportion of the frame's two sides, whatever its
  orientation: 4 : 5 names a 5 : 4 landscape frame as well.
- **Locked ratio** — a ratio the frame keeps through every gesture.
- **Crop mode** — the state of Develop in which the framing is edited.
- **Level tool** — a line drawn on the picture along something that should be
  horizontal or vertical.
- **Quarter turn** — the picture turned by 90°, left or right.
- **Mirror** — the picture flipped, horizontally or vertically.
- **Framed photo** — the picture after its framing: what the viewport, the
  histogram and the export work on.

## 4. Behavior

### Crop mode
1. The tool bar starts with a **Crop** icon button, then a separator, then the
   mask tools (`shell.md` rule 11). The button, or `R`, enters crop mode on a
   photo that is ready; it shows as on while crop mode lasts. Crop mode exists
   in Develop mode only.
2. Entering crop mode leaves Before, deselects the mask and disarms the mask
   tool. In crop mode Before and the mask tools are disabled, and neither mask
   handles nor a mask overlay are drawn.
3. In crop mode the viewport shows the whole picture, developed with its edit,
   at fit: zoom and pan are off. The frame is always upright on screen: the
   picture is the one that turns with the angle.
4. The part of the picture outside the frame is dimmed. Inside the frame a
   grid of thirds is drawn; the frame has a handle on each corner and on each
   side.
5. In place of the photo's sections the panel shows the **Crop** section:
   aspect ratio and lock, Angle, Level, Rotate left, Rotate right, Flip
   horizontal, Flip vertical, Reset crop, Done. The histogram and the panel
   foot stay.
6. **Done**, `Enter`, `R`, or a double click inside the frame leaves crop mode
   and keeps the framing. Selecting another photo or switching to Cull mode
   does the same.
7. `Esc` leaves crop mode and gives the photo back the framing it had when
   crop mode was entered.
8. Leaving crop mode shows the framed photo at fit.

### Crop frame
9. Dragging a corner handle moves the two sides meeting there; dragging a side
   handle moves that side. With a locked ratio the frame keeps it: the other
   sides follow, around the opposite corner or the middle of the opposite side.
10. Dragging inside the frame moves the frame over the picture without
    resizing it.
11. The frame never leaves the picture: no gesture makes it keep a point
    outside the picture, whatever the angle. A drag that would is stopped at
    the edge.
12. A side of the frame is never shorter than a fiftieth of the picture's long
    edge.
13. Every gesture updates the viewport at every frame of the drag
    (`develop.md` rule 12).

### Aspect ratio
14. The ratio selector offers Free, Original (the picture's own ratio), 1 : 1,
    4 : 5, 3 : 2 and 16 : 9. Choosing a ratio locks it and replaces the frame
    with the largest frame of that ratio, of the same orientation and centre,
    held inside the current frame. Choosing Free unlocks the ratio and leaves
    the frame as it is.
15. The lock button shows as on whenever the ratio is locked. Clicked on Free,
    it locks the ratio the frame has; the selector then shows the name of that
    ratio, or **Custom** when it has none. Clicked while locked, it goes back
    to Free.
16. The choice of ratio is not part of the edit. Entering crop mode, the ratio
    is locked on the one the frame has: Original for a photo never cropped.
17. `X` turns a landscape frame into a portrait one and back: same centre,
    sides swapped, shrunk as much as needed to stay inside the picture. In
    crop mode `X` no longer rejects the photo (`culling.md` rule 25); ratings
    by `0` to `5` keep working.

### Straighten
18. The **Angle** slider goes from −45.0° to +45.0°, default 0, with one
    decimal. Raised, the picture turns clockwise under the frame. It is a panel
    slider: typed value, clamping, double-click reset (`develop.md` rules 13,
    14, 16).
19. Dragging outside the frame turns the picture: the angle follows the turn of
    the pointer around the centre of the frame, within the slider's range.
    While the angle changes a finer grid replaces the thirds.
20. When the angle changes, the frame keeps its centre and its ratio and is
    shrunk just as much as needed to stay inside the picture (rule 11). During
    one drag the frame is worked out from the frame the drag started with:
    coming back to the starting angle gives the starting frame back.
21. **Level** arms the level tool; it shows as on while armed and `Esc`
    disarms it without leaving crop mode. The next drag on the picture draws a
    line; at its release the angle becomes the one that makes that line
    horizontal or vertical, whichever needs the smaller turn, and the tool is
    disarmed. A line shorter than a hundredth of the picture's long edge is
    ignored.

### Quarter turns and mirror
22. **Rotate left** and **Rotate right** turn the photo by a quarter turn.
    `Cmd+[` and `Cmd+]` do the same in Develop mode, in crop mode or not.
23. **Flip horizontal** and **Flip vertical** mirror the photo.
24. A quarter turn or a mirror carries everything with the picture: the frame
    keeps the same part of the picture, the angle keeps levelling the same
    line, every mask stays on what it covered. Four quarter turns the same
    way, or the same mirror twice, give back exactly the photo there was.

### The framed photo
25. With the default framing the photo is exactly what it was before this
    milestone, on screen and in an export.
26. Outside crop mode the viewport shows the framed photo only. Fit, 100 %, the
    zoom bounds, the zoom readout and the pan limits (`import.md` rules 17–23)
    are those of the framed photo.
27. The framing comes last: the picture is developed first, then framed. The
    adjustments, the masks, the color tools and the enhancement give each
    point of the picture the same result whatever the framing.
28. A mask belongs to the picture, not to the frame: changing the framing
    never moves it on the picture. A mask drawn on the framed photo lands
    where it was drawn; the part of a mask outside the frame is not shown and
    is found again when the frame is widened.
29. A zone detection (`zone-masks.md`) and an enhancement (`enhance.md`) work
    on the whole picture, framed or not: neither is lost or computed again
    when the framing changes.
30. The histogram is that of the framed photo (`histogram.md` rule 2): the
    part outside the frame does not count. In crop mode it follows the frame.
31. A straightened photo has no step along its tilted lines and no blur beyond
    what turning a picture needs: a fine line stays a fine line.

### With the rest of the app
32. The framing is part of the edit: saved in the sidecar, restored when the
    photo is opened again, counted by the edited marker and the edited count.
    A photo whose only change is its framing is an edited photo.
33. One handle drag, one move of the frame, one drag or typed value of the
    angle, one level line, one choice of ratio that changes the frame, one
    `X`, one quarter turn, one mirror, one Reset crop: each is one step of
    undo and redo (`develop.md` rule 23), in crop mode as outside it. The
    `Esc` of rule 7 is one step too.
34. `develop.md` rule 25 is amended: Before keeps the framing. It shows the
    framed photo without the rest of its edit, so that the same picture is
    compared.
35. **Reset crop** returns the frame to the whole picture and the angle to 0;
    quarter turns and mirrors stay. It is disabled when both are at their
    default.
36. **Reset** of the panel foot returns the whole framing to its default with
    the rest of the edit (`develop.md` rule 15), quarter turns and mirrors
    included.
37. `export.md` rule 10 is amended: an export writes the framed photo. At full
    resolution the file has the frame's size in pixels of the picture, rounded
    to whole pixels, whatever the angle; a long edge applies to the framed
    photo.
38. `develop.md` rule 27 is amended: Paste does not carry the framing. The
    photo pasted onto keeps its own; pasted masks keep the place they had on
    the picture (`local-adjustments.md` rule 24).
39. A sidecar written by an earlier ziv still opens, with the default framing.
    A sidecar holding a framing is not understood by an earlier ziv, which
    leaves it alone (ADR 0003).

### Design
40. The frame, its handles and its grids are neutral light lines readable on
    any picture; no accent on the picture. `develop.md` rules 30–35 hold: every
    control of the Crop section has a visible keyboard focus and an accessible
    name, and so has the crop frame.

## 5. Acceptance criteria
- [x] F1 — With the default framing the engine output and the export equal
      the ones before this milestone, on a standard image and on a RAW
      (rule 25).
- [x] F2 — A frame renders exactly the pixels of the picture it holds, and
      matches its golden (rules 26, 27).
- [x] F3 — The Crop button and `R` enter crop mode; the Crop section replaces
      the photo's sections; Before and the mask tools are disabled; Done,
      `Enter`, `R` and a double click keep the framing, `Esc` gives the
      earlier one back (rules 1, 2, 5–7).
- [x] F4 — In crop mode the whole picture is shown at fit with the frame, its
      eight handles, the dimmed outside and the thirds; leaving shows the
      framed photo at fit (rules 3, 4, 8).
- [x] F5 — Corner and side handles resize the frame, a drag inside moves it,
      with and without a locked ratio; the frame never leaves the picture and
      never gets smaller than its minimum (rules 9–12).
- [x] F6 — Each ratio of the selector gives the largest frame of that ratio
      inside the current one; Free unlocks; the lock locks the current ratio
      and names it, Custom included; entering crop mode locks the frame's
      ratio (rules 14–16).
- [x] F7 — `X` swaps landscape and portrait and stays inside the picture; in
      crop mode it does not reject the photo, and `0` to `5` still rate
      (rule 17).
- [x] F8 — For any angle and any frame, the four corners of the frame are
      inside the picture; an angle changed and brought back within one drag
      gives the starting frame back (rules 11, 20).
- [x] F9 — A straightened picture matches its golden: the frame's content
      turned by the angle, a tilted fine line kept fine and without steps
      (rules 18, 31).
- [x] F10 — The Angle slider, a drag outside the frame and a typed value set
      the angle within ±45°; the finer grid shows while it changes (rules 18,
      19).
- [x] F11 — A level line gives the angle that makes it horizontal or vertical,
      whichever is nearer; a too short line changes nothing; `Esc` disarms the
      tool only (rule 21).
- [x] F12 — Quarter turns and mirrors render as their goldens; the frame, the
      angle and a mask keep the same part of the picture; four turns or two
      identical mirrors are the identity; `Cmd+[` and `Cmd+]` work outside
      crop mode (rules 22–24).
- [x] F13 — Fit, 100 %, the zoom readout and the pan limits are those of the
      framed photo (rule 26).
- [x] F14 — A mask keeps its place on the picture when the framing changes, a
      mask drawn on a framed, straightened and turned photo covers what was
      drawn over, and an adjustment gives a point of the picture the same
      value framed or not (rules 27, 28).
- [x] F15 — A zone mask and an enhancement made before a framing still match
      the picture after it, and the other way round (rule 29).
- [x] F16 — The histogram counts the framed photo only (rule 30).
- [x] F17 — A framing survives closing and reopening the photo and makes it an
      edited photo; an earlier sidecar still opens (rules 32, 39).
- [x] F18 — Undo and redo step through the framing gestures one by one;
      Before keeps the framing; Reset crop and Reset reset what they say;
      Paste leaves the framing of the photo pasted onto (rules 33–36, 38).
- [x] F19 — The export of a framed photo equals the display render of the
      framed photo, at the frame's size and at a long edge (rule 37).
- [x] F20 — Demo, in the real app on the A7 IV RAW: level the horizon with the
      level tool, crop to 4 : 5 portrait, turn a photo shot sideways, darken
      the sky with a gradient drawn on the framed photo; the viewport stays
      fluid while dragging; quit, relaunch, find the framing again; export and
      get the framed file (rules 13, 21, 32, 37, 40).

## 6. Test plan
| Criterion | Level | Fixture |
|-----------|-------|---------|
| F1 | G | M1's synthetic ramp and patches; M2's RAW golden |
| F2 | U + G | a synthetic image whose pixels tell their position |
| F3 | Eu + HV | develop panel state injected; the keys in the real app |
| F4 | Eu + HV | crop canvas state injected; real app |
| F5 | U + Eu | none; pointer input on the crop canvas |
| F7 | U + HV | none; the keys in the real app |
| F6 | U + Eu | frames of several ratios; Crop section state injected |
| F8 | U | angles and frames swept over their ranges |
| F9 | U + G | a synthetic image with a fine tilted line |
| F10 | U + Eu + HV | crop canvas and Crop section state injected; the slider in the real app |
| F11 | U + Eu | crop canvas state injected; pointer input |
| F12 | U + G + Eu | the position image, one golden per turn and mirror; one mask |
| F13 | U + Eu | a view on a framed photo |
| F14 | U + G | the position image with one mask, framed and not |
| F15 | Eb | synthetic image with a stored zone mask and an enhancement file |
| F16 | Eb | an image half black, half white, framed on one half |
| F17 | U + Eb | temp folder, a sidecar of the current version |
| F18 | U + Eu | none |
| F19 | Eb | the position image, export at two sizes, with and without angle |
| F20 | HV | local `DSC07070.ARW` |

## 7. Open questions
None.
