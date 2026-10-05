# ziv — Tone curve and color

## 1. Goal
Finish the global development with Lightroom's three color tools: shape the
tones with a point curve, move each color of the photo with the color mixer,
and tint shadows, midtones and highlights with color grading.

## 2. Scope
**In**
- A **Tone curve** section: a point curve for RGB and one per channel (Red,
  Green, Blue), edited on a graph, the photo's histogram behind it.
- A **Color mixer** section: Hue, Saturation and Luminance of eight color
  ranges.
- A **Color grading** section: Shadows, Midtones, Highlights and Global, each
  with a color wheel and a luminance; Blending and Balance.
- The three as part of the photo's edit: live preview, sidecar, undo and redo,
  Before, Reset, copy and paste, export, histogram.

**Out**
- These three tools on a mask: a mask keeps its nine adjustments
  (`local-adjustments.md` rule 6); the tools act on the whole photo.
- The parametric curve (Highlights, Lights, Darks, Shadows regions).
- Typed coordinates for a curve point; moving a point with the keyboard.
- The targeted adjustment tool (dragging on the photo), eyedroppers.
- The point color tool, black-and-white mixing, curve or grading presets.
- Per-channel histograms behind the graph (`histogram.md` scope).
- Promise of the same picture as Lightroom for the same numbers
  (`develop.md` §4).

## 3. Vocabulary
- **Tone curve** — a mapping from the tone a pixel has to the tone it gets,
  drawn from black (left, bottom) to white (right, top).
- **Curve channel** — what a curve acts on: RGB (the three channels alike),
  Red, Green or Blue. A photo has one curve per channel.
- **Curve point** — a point the curve goes through. The two **end points**
  always exist.
- **Identity curve** — the straight diagonal: it changes nothing.
- **Color mixer** — the Hue, Saturation and Luminance adjustments of the color
  ranges.
- **Color range** — one of Red, Orange, Yellow, Green, Aqua, Blue, Purple,
  Magenta: the pixels whose hue is near that color.
- **Color grading** — a tint and a luminance given to a tonal zone.
- **Tonal zone** — Shadows, Midtones, Highlights (the dark, middle and bright
  tones of the photo) or Global (all of them).
- **Color wheel** — the disc setting a zone's tint: the angle is the hue, the
  distance from the centre the saturation.
- **Balance** — where the photo's tones pass from shadows to highlights.
- **Blending** — how much the tonal zones overlap.

## 4. Behavior

### Sections
1. The develop panel's sections of a photo are, in order: Masks, White
   balance, Tone, Presence, Tone curve, Color mixer, Color grading, Detail.
   The three new ones fold like the others (`shell.md` rule 13).
2. With a mask selected the three new sections are not shown: they belong to
   the photo only. They apply after the masks, to the whole photo: a masked
   area gets them like the rest (ADR 0014).
3. With the three tools at their defaults the photo is exactly what it was
   before this milestone.
4. The result does not depend on the order in which the controls were touched
   (`develop.md` rule 8).

### Tone curve
5. The section shows a square graph: the tone before the curve from black on
   the left to white on the right, the tone after it from black at the bottom
   to white at the top, as the screen shows them. A grid of quarters and the
   identity diagonal are drawn under the curve.
6. A selector above the graph chooses the curve channel shown: RGB, Red,
   Green, Blue. A channel whose curve is not the identity is marked on the
   selector.
7. The curve of a channel is smooth, goes through every one of its points and
   never leaves the graph. Left of the first point and right of the last one
   it is flat.
8. Clicking on the graph away from a point adds a point there and the same
   press can drag it. Dragging a point moves it; it cannot pass its
   neighbours horizontally nor leave the graph.
9. Double-clicking a point, or dragging it out of the graph, removes it. The
   end points cannot be removed; they are dragged like the others: raising the
   left one lifts black, lowering the right one dims white.
10. The **RGB** curve remaps the three channels alike: a neutral grey stays
    neutral. The **Red**, **Green** and **Blue** curves remap their channel
    only: raising the middle of Red warms a grey toward red, lowering it
    toward cyan. The four curves combine.
11. Dragging a point updates the viewport at every frame of the drag
    (`develop.md` rule 12).
12. A **Reset curve** button returns the shown channel to the identity; it is
    disabled when that curve is the identity.
13. Behind the graph, faint, the histogram of the photo as developed
    (`histogram.md` rules 2–5), the same whatever the channel shown. It
    follows the edit, the curve included.

### Color mixer
| Adjustment | Per color range | Range | Default |
|-----------|-----------------|-------|---------|
| Hue | Red, Orange, Yellow, Green, Aqua, Blue, Purple, Magenta | −100 … +100 | 0 |
| Saturation | the same eight | −100 … +100 | 0 |
| Luminance | the same eight | −100 … +100 | 0 |

14. A selector chooses Hue, Saturation or Luminance; the section then shows
    the eight sliders of that adjustment, one per color range, in the order of
    the table. A choice holding a value away from its default is marked on
    the selector.
15. **Hue** turns the colors of a range toward the neighbouring range: lowered,
    toward the one before it in the list (Red toward Magenta); raised, toward
    the one after it (Red toward Orange, Magenta toward Red).
16. **Saturation** scales the intensity of the colors of a range; −100 makes
    them grey. **Luminance** darkens (lowered) or brightens (raised) them.
17. A color between two ranges gets a share of each: a smooth gradient of hues
    stays smooth, with no step, whatever the sliders.
18. A neutral grey is never changed by the color mixer.
19. Each slider's track shows its effect: for Hue, the hues the range turns
    to; for Saturation, from grey to the range's color; for Luminance, from
    its dark to its bright shade. No accent fill; a value away from its
    default is told by its accent value (`shell.md` rule 19).
20. The sliders are the develop panel's: typed value, clamping, double-click
    reset (`develop.md` rules 13, 14, 16).

### Color grading
| Adjustment | Range | Default |
|-----------|-------|---------|
| Hue (per zone) | 0 … 360° | 0 |
| Saturation (per zone) | 0 … 100 | 0 |
| Luminance (per zone) | −100 … +100 | 0 |
| Blending | 0 … 100 | 50 |
| Balance | −100 … +100 | 0 |

21. A selector chooses the view: **3-way**, Shadows, Midtones, Highlights,
    Global. It starts on 3-way. A zone away from its defaults is marked on the
    selector.
22. The 3-way view shows the three wheels of Shadows, Midtones and Highlights
    together, each with its Luminance slider.
23. A zone's view shows one large wheel and that zone's Hue, Saturation and
    Luminance sliders. The wheel and the sliders show the same values.
24. Blending and Balance are shown under every view.
25. A wheel is the disc of all hues, grey at its centre. Pressing or dragging
    in it places its point: the angle sets the zone's Hue, the distance from
    the centre its Saturation, the rim being 100. Double-clicking a wheel
    returns Hue and Saturation of its zone to 0.
26. A zone tints its tones toward its Hue, the more the higher its
    Saturation; at Saturation 0 the Hue changes nothing. **Shadows** acts on
    the dark tones and leaves the bright ones alone, **Highlights** the
    reverse, **Midtones** on the middle and fades toward both ends,
    **Global** on every tone alike.
27. A zone's **Luminance** brightens (raised) or darkens (lowered) the tones
    the zone acts on.
28. **Balance** moves the passage between shadows and highlights: raised, more
    of the photo counts as highlights; lowered, more as shadows.
29. **Blending** sets how wide the zones overlap: at 0 each zone keeps to its
    own tones, at 100 each spreads far into the others.
30. With every Saturation and Luminance at 0, Blending and Balance change
    nothing.
31. Dragging in a wheel updates the viewport at every frame of the drag; one
    drag is one step of undo.

### With the rest of the app
32. The three tools are part of the edit: saved in the sidecar, restored when
    the photo is opened again, written into exports, counted by the histogram
    and by the edited marker.
33. One point added, moved or removed, one Reset curve, one slider change, one
    wheel drag: each is one step of undo and redo (`develop.md` rule 23).
34. Before shows the photo without them. Reset returns the three tools to
    their defaults with the rest of the edit.
35. Paste copies them with the rest of the edit, between a RAW and a standard
    image as well: their scales are the same on both.
36. A sidecar written by an earlier ziv still opens, the three tools at their
    defaults. A sidecar holding them is not understood by an earlier ziv,
    which leaves it alone (ADR 0003).

### Design
37. `develop.md` rule 30 is amended again: hue also appears on the color mixer
    tracks, the color wheels and the Red, Green and Blue curves. Rules 31–35
    hold: the graph and each wheel have a visible keyboard focus and an
    accessible name.

## 5. Acceptance criteria
- [x] T1 — With the three tools at their defaults the engine output equals
      the one before this milestone, on a standard image and on a RAW
      (rule 3).
- [x] T2 — A curve goes through its points, stays in the graph, is flat
      outside its end points; the identity curve changes nothing (rules 7, 9).
- [x] T3 — The RGB curve keeps a neutral grey neutral and matches its golden;
      a Red, Green or Blue curve changes its channel only; the four combine
      (rule 10).
- [x] T4 — On the graph: a click adds a point, a drag moves it between its
      neighbours, a double click or a drag outside removes it, the end points
      stay (rules 8, 9).
- [x] T5 — The channel selector shows each channel's curve and marks the
      edited ones; Reset curve resets the shown channel only (rules 6, 12).
- [x] T6 — The graph shows the photo's histogram behind the curve, following
      the edit (rule 13).
- [x] T7 — Each color mixer adjustment acts on its range, leaves the opposite
      hue and a neutral grey untouched, and matches its golden
      (rules 15, 16, 18).
- [x] T8 — A hue gradient developed with the color mixer has no step between
      two neighbouring pixels larger than a stated bound (rule 17).
- [x] T9 — The color mixer section shows eight sliders per choice, with
      colored tracks, the edited choices marked; they behave as panel sliders
      (rules 14, 19, 20).
- [x] T10 — Each tonal zone tints and brightens the tones it acts on and
      leaves the far end within tolerance; Global acts on all; Saturation 0
      is the identity; each matches its golden (rules 26, 27, 30).
- [x] T11 — Balance and Blending move and widen the zones as described
      (rules 28, 29).
- [x] T12 — A wheel sets Hue and Saturation from a press or a drag, resets on
      a double click, and agrees with the zone's sliders (rules 23, 25).
- [x] T13 — The grading selector shows the 3-way view and the four zone
      views, Blending and Balance under each, the edited zones marked
      (rules 21–24).
- [x] T14 — The three sections sit in the panel's order, fold, and are absent
      while a mask is selected; a masked area gets the three tools like the
      rest (rules 1, 2).
- [x] T15 — A curve, a color mixer value and a grading survive closing and
      reopening the photo; an earlier sidecar still opens (rules 32, 36).
- [x] T16 — Undo, redo, Before, Reset and paste handle the three tools as
      described (rules 33–35).
- [x] T17 — The export of a photo developed with the three tools equals the
      display render (rule 32).
- [x] T18 — The result is independent of the order of the changes (rule 4).
- [x] T19 — Demo, in the real app on the A7 IV RAW: an S-curve on RGB, lifted
      blacks on Blue, greens desaturated, blues darkened, teal shadows and
      orange highlights; the viewport stays fluid while dragging; quit,
      relaunch, find the edit again; export (rules 11, 31, 32, 37).

## 6. Test plan
| Criterion | Level | Fixture |
|-----------|-------|---------|
| T1 | G | M1's synthetic ramp and patches; M2's RAW golden |
| T2, T18 | U | none |
| T3 | U + G | synthetic ramp, one golden per curve channel |
| T4, T5 | Eu | graph state injected; pointer input |
| T6 | Eu + HV | a histogram injected; local RAW |
| T7 | U + G | hue wheel patches, one golden per adjustment |
| T8 | U | a computed hue gradient |
| T9 | Eu + HV | color mixer state injected |
| T10 | U + G | synthetic ramp, one golden per zone |
| T11 | U | none |
| T12, T13 | Eu | grading state injected; pointer input |
| T14 | Eu + G | develop panel state injected; ramp with one mask |
| T15 | U + Eb | temp folder, a sidecar of the current version |
| T16 | U + Eu | none |
| T17 | Eb | synthetic image, export at two sizes |
| T19 | HV | local `DSC07070.ARW` |

## 7. Open questions
None.
