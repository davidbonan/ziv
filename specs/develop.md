# ziv — Global development

## 1. Goal
Develop the selected photo with the sliders of Lightroom's Basic panel, see the
result live in the viewport, and find the edits again the next time the photo
is opened — the source file never being touched.

## 2. Scope
**In**
- A develop panel beside the viewport with the adjustments of §4.
- A base rendering for RAW files, so an untouched RAW looks close to the
  camera's JPEG.
- Live preview while a slider moves.
- Edits saved automatically in a sidecar next to the photo (ADR 0003).
- Reset of one adjustment and of all of them.
- Undo / redo, before / after, copy / paste of the edits.
- A visual design held to the standard of §5, M1's screens included.

**Out**
- Tone curve, HSL, color grading — next milestone.
- Texture, clarity, dehaze, sharpening, noise reduction, lens corrections, crop.
- Histogram, clipping warnings.
- Presets, pasting a chosen subset of the edits, pasting onto several photos.
- Filmstrip thumbnails reflecting the edits: they keep showing the source.
- Camera-specific looks: one base rendering serves every camera.
- Export (M3).

## 3. Vocabulary
- **Adjustment** — one slider's parameter. Its default leaves the photo unchanged.
- **Edit** — the set of all adjustments of one photo.
- **Base rendering** — what ziv applies to every RAW before the edit so it looks
  finished: brightness and contrast close to the camera's JPEG. Not an
  adjustment: it has no control and is not stored.
- **Develop panel** — the panel holding the adjustments of the selected photo.
- **Sidecar** — the file next to a photo that stores its edit.
- **History** — the successive states of a photo's edit during this run of the app.
- **Before** — the photo with its base rendering and no edit.
- **Copied edit** — the edit kept in memory by Copy, waiting for Paste.

## 4. Behavior

### Adjustments
Names and ranges are Lightroom's. The same number is not promised to give the
same picture as Lightroom.

| Group | Adjustment | Range | Default |
|-------|-----------|-------|---------|
| White balance | Temp (RAW) | 2000 … 50000 K | as shot |
| White balance | Tint (RAW) | −150 … +150 | as shot |
| White balance | Temp (JPEG, PNG, TIFF) | −100 … +100 | 0 |
| White balance | Tint (JPEG, PNG, TIFF) | −100 … +100 | 0 |
| Tone | Exposure | −5.00 … +5.00 EV | 0 |
| Tone | Contrast | −100 … +100 | 0 |
| Tone | Highlights | −100 … +100 | 0 |
| Tone | Shadows | −100 … +100 | 0 |
| Tone | Whites | −100 … +100 | 0 |
| Tone | Blacks | −100 … +100 | 0 |
| Presence | Vibrance | −100 … +100 | 0 |
| Presence | Saturation | −100 … +100 | 0 |

1. Every adjustment at its default shows the photo exactly as M1 showed it,
   plus the base rendering for a RAW.
2. **Temp** warms the photo toward yellow when raised, cools it toward blue when
   lowered; **Tint** goes to magenta when raised, to green when lowered. On a
   RAW, the defaults are the camera's as-shot values and a neutral grey stays
   neutral at those values.
3. **Exposure** +1.00 doubles the light of the whole photo; −1.00 halves it.
4. **Contrast** spreads tones away from middle grey when raised, pulls them
   toward it when lowered; middle grey does not move.
5. **Highlights** and **Shadows** brighten (raised) or darken (lowered) the
   bright, respectively dark, part of the photo and leave the other end alone.
   Highlights lowered brings back detail brighter than display white when the
   file holds it.
6. **Whites** and **Blacks** move the brightest, respectively darkest, end of
   the range: the point that reaches pure white, pure black.
7. **Saturation** scales the intensity of every color; −100 gives a
   black-and-white photo. **Vibrance** does the same but acts mostly on muted
   colors and holds back on already saturated ones.
8. The result does not depend on the order in which the sliders were touched.

### Base rendering
9. An untouched RAW is close to its camera JPEG in overall brightness and
   contrast; highlights roll off smoothly instead of clipping hard.
10. JPEG, PNG and TIFF get no base rendering.

### Develop panel
11. The panel is shown beside the viewport when the selected photo is ready; it
    is disabled, not hidden, while the photo loads or when it failed.
12. Dragging a slider updates the viewport at every frame of the drag.
13. Each adjustment shows its value; the value can be typed. A typed value
    outside the range is brought back into it.
14. Double-clicking a slider or its name resets that adjustment to its default.
15. **Reset** returns every adjustment of the photo to its default.
16. An adjustment away from its default is visibly distinct from one at default.
17. Zoom, pan and the filmstrip keep working while developing; the view is not
    reset by an edit.

### Persistence
18. An edit is saved by itself shortly after each change; there is no Save
    command and no unsaved state to manage.
19. Selecting a photo that has a sidecar shows it with its edit and the sliders
    at the saved values.
20. A photo whose edit is back to all defaults has no sidecar.
21. If the sidecar cannot be written (read-only folder, full disk), a message
    says so; the edit stays in effect for this run of the app.
22. If a sidecar cannot be read or comes from a newer ziv, the photo is shown
    without edit, the panel is disabled with the reason, and the sidecar is
    left untouched.

### Undo / redo
23. `Cmd+Z` undoes the last change of the selected photo's edit, `Cmd+Shift+Z`
    redoes it. One slider drag is one step, as are a typed value, a reset, a
    Reset and a paste.
24. Each photo has its own history; it lasts until another session is opened or
    the app quits. A new change after an undo discards what could be redone.

### Before / after
25. `B`, or the Before button of the panel, switches the viewport between the
    edited photo and Before; a "Before" badge is shown while Before is on.
    Amended by `crop-and-straighten.md` rule 34: Before keeps the framing.
26. Changing an adjustment, or selecting another photo, leaves Before.

### Copy / paste
27. `Cmd+Shift+C` copies the selected photo's edit; `Cmd+Shift+V` replaces the
    selected photo's edit with the copied one. Amended by
    `crop-and-straighten.md` rule 38: the framing is not pasted.
28. Pasting between a RAW and a JPEG / PNG / TIFF leaves white balance as it
    was: their scales are not the same.
29. Paste does nothing when no edit was copied. The copied edit lasts until the
    app quits.

## 5. Design
The interface is part of the product, not a wrapper around it.

30. Dark surfaces sharing one faint night-blue cast, kept low enough in
    saturation that the photo stays the only strong color on screen. One accent
    color, a glacier blue, used only for selection and for values away from
    default. Amended by `shell.md` rule 20: the Temp and Tint tracks show their
    hues.
31. One spacing scale, one type scale, tabular digits for values: nothing
    shifts by a pixel when a value changes.
32. A slider whose default is in the middle fills from the middle; its handle
    and track have hover, pressed and disabled states.
33. Every control has a visible keyboard focus and an accessible name.
34. Layout is stable: showing a message, a badge or the panel never moves the
    photo under the pointer unexpectedly.
35. M1's screens — empty state, filmstrip, loading and error states — follow
    the same rules.

## 6. Acceptance criteria
- [x] D1 — Default edit is the identity: the engine output equals M1's for a
      standard image (rule 1).
- [x] D2 — Each adjustment, at a non-default value, matches its golden and its
      domain math (rules 2–7).
- [x] D3 — Exposure +1 doubles linear values; Contrast keeps middle grey;
      Saturation −100 yields equal channels; Highlights / Shadows leave the
      opposite end within tolerance (rules 3–7).
- [x] D4 — A RAW neutral grey stays neutral at as-shot white balance (rule 2).
- [x] D5 — Untouched, the A7 IV RAW's mean display brightness is within 15 % of
      its embedded camera preview's (rule 9).
- [x] D6 — A standard image gets no base rendering (rule 10).
- [x] D7 — Moving a slider changes the rendered viewport in the same frame
      (rule 12).
- [x] D8 — Typed values, range clamping, double-click reset and Reset behave as
      described; a non-default adjustment is marked (rules 13–16).
- [x] D9 — Panel disabled while loading, on failure and on an unreadable
      sidecar (rules 11, 22).
- [x] D10 — An edit written, then the photo reopened in a new app: same slider
      values, same render (rules 18, 19).
- [x] D11 — All-default edit removes the sidecar; write failure is reported and
      the edit kept; unreadable or newer sidecar is never overwritten
      (rules 20–22).
- [x] D12 — Undo / redo steps and per-photo histories behave as described
      (rules 23, 24).
- [x] D13 — Before / after switches the render, shows the badge, and is left by
      an edit or a selection (rules 25, 26).
- [x] D14 — Copy / paste transfers the edit, white balance excepted across
      kinds (rules 27–29).
- [x] D15 — The result is independent of the order of slider changes (rule 8).
- [x] D16 — Every screen, captured from the real app, has been looked at
      against rules 30–35.

## 7. Test plan
| Criterion | Level | Fixture |
|-----------|-------|---------|
| D1 | G | M1's synthetic ramp and patches |
| D2 | U + G | synthetic input, one golden per adjustment |
| D3, D4, D15 | U | none |
| D5 | Eb | local `DSC07070.ARW` and its embedded preview |
| D6 | U + G | `patches.png` |
| D7 | HV | local RAW; capture before and after a slider change |
| D8, D9 | Eu | none |
| D10, D11 | U + Eb + HV | photo copied to a temp folder; read-only folder |
| D12, D14 | U + Eu | none |
| D13 | Eu + HV | local RAW |
| D16 | HV | every state of the app |

## 8. Open questions
None.
