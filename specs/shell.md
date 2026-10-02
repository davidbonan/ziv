# ziv — Window shell

## 1. Goal
Give the window the layout and the controls of direction A of
`specs/mockups/shell.html`: a top bar, the series on the left, the photo in the
middle above its filmstrip, the develop panel on the right — with tools as
icons, sections that fold and white balance sliders that show their colors.

## 2. Scope
**In**
- A top bar: sidebar button, series and photo names, zoom, Before, Export….
- The layout of direction A; a filmstrip without buttons, with the position in
  the series and a marker on edited photos.
- Develop panel: mask tools as icons, folding sections, a fixed foot with
  Copy, Paste and Reset.
- Temp and Tint sliders with colored tracks.

**Out**
- Histogram and shooting data (ISO, focal length, aperture, speed): shown in
  the mockup, framed in `histogram.md`.
- Directions B and C of the mockup.
- Resizable or detachable panels, a light theme, configurable shortcuts.
- Any change to what an adjustment, a mask or an export does.

## 3. Vocabulary
- **Top bar** — the band across the top of the window.
- **Zoom readout** — the zoom of the viewport, as text in the top bar.
- **Edited marker** — the dot on the thumbnail of an edited photo
  (`catalog.md`: a photo that has a sidecar).
- **Tool bar** — the row of mask tools at the top of the develop panel.
- **Section** — a titled group of the develop panel: Masks, White balance,
  Tone, Presence, Detail.
- **Panel foot** — the band at the bottom of the develop panel.

## 4. Behavior

### Top bar
1. The top bar is always shown, across the whole window.
2. On its left, a button hides and shows the series sidebar, as `Cmd+L` does;
   it shows as on while the sidebar is shown.
3. Next to it, the name of the open series and of the selected photo:
   "Lofoten / DSC07070.ARW". With no open series, nothing.
4. On its right, the zoom readout of a photo that is ready: "Fit · 24 %" at
   fit, "150 %" otherwise. Nothing while the photo loads or when it failed.
5. Then **Before**: what `B` does (`develop.md` rule 25); it shows as on while
   Before is shown. Disabled when no photo is ready.
6. Then **Export…**: what `Cmd+E` does (`export.md` rule 1). It is the one
   button of the window filled with the accent. Disabled with no open series.

### Layout
7. Under the top bar: the sidebar on the left and the develop panel on the
   right, both down to the bottom of the window; between them the viewport
   above the filmstrip.
8. The filmstrip has no button. It starts with the position of the selected
   photo in the series: "3 / 148".
9. The thumbnail of an edited photo carries the edited marker; the marker
   follows the edits as the count of the sidebar does (`catalog.md` rule 12).
10. With no open series there is no filmstrip and no develop panel; the
    sidebar and the top bar stay.

### Develop panel
11. The tool bar is one row of icon buttons: Linear, Radial, Rectangle,
    Polygon, Brush, a separator, Subject, Background, Sky, People. Each keeps
    its name for assistive technology and shows it when hovered.
12. The armed tool shows as on. A tool is disabled when it was before: no room
    for a mask, or a detection running (`local-adjustments.md`, `zone-masks.md`).
13. Each section has a title; clicking it folds or unfolds the section. Every
    section starts unfolded. Folding is kept when another photo is selected.
14. The Masks title carries the number of masks of the photo when it has some.
15. With a mask selected, its name, Done, Invert, Overlay and its sliders are
    shown under the list of masks, in place of the photo's sections, as today.
16. The panel foot stays at the bottom of the panel whatever is scrolled:
    **Copy** and **Paste** do what `Cmd+Shift+C` and `Cmd+Shift+V` do
    (`develop.md` rules 27–29), **Reset** what it did (rule 15). Paste is
    disabled until an edit was copied, Reset when nothing is edited.
17. The panel no longer has a title, nor Before and Reset at its top.

### White balance sliders
18. The Temp track goes from blue through neutral to yellow, the Tint track
    from green through neutral to magenta: each end shows where the slider
    takes the photo. This holds for a photo's and for a mask's white balance.
19. These two tracks have no accent fill; a value away from its default is
    still told by its accent value (`develop.md` rule 16).

### Design
20. `develop.md` rule 30 is amended: besides the accent, hue appears on the
    Temp and Tint tracks only. Rules 31–35 hold for everything above.

## 5. Acceptance criteria
- [x] S1 — The top bar shows the series and photo names, and its sidebar
      button hides and shows the sidebar (rules 1–3).
- [x] S2 — The zoom readout says "Fit · N %" at fit and "N %" otherwise, and
      nothing without a ready photo (rule 4).
- [x] S3 — Before and Export… of the top bar do what their shortcuts do and
      are disabled as said (rules 5, 6).
- [x] S4 — In the real app the sidebar and the develop panel reach the bottom
      of the window and the filmstrip sits between them (rule 7).
- [x] S5 — The filmstrip shows the position and no button; an edited photo has
      its marker (rules 8, 9).
- [x] S6 — Every mask tool is an icon button found by its name; clicking one
      arms it or asks for its detection; disabled states hold (rules 11, 12).
- [x] S7 — A section folds and unfolds by its title; the Masks title carries
      the count (rules 13, 14).
- [x] S8 — The foot's Copy, Paste and Reset act as their shortcuts and are
      disabled as said; the panel has no title (rules 16, 17).
- [x] S9 — Temp and Tint show colored tracks, on a photo and on a mask, and
      still work as sliders (rules 18, 19).
- [x] S10 — Demo: the real app on a series looks like direction A of the
      mockup, histogram and shooting data aside (rules 1–19).

## 6. Test plan
| Criterion | Level | Fixture |
|-----------|-------|---------|
| S1, S3 | Eu + HV | top bar state injected; catalog of one series |
| S2 | U + Eu | views at fit and zoomed |
| S4 | HV | one series injected through the constructor seam |
| S5 | Eu + HV | strip of three photos, one edited |
| S6, S7, S8 | Eu | develop panel state injected |
| S9 | Eu + HV | sliders of a RAW and of a standard image |
| S10 | HV | a folder of the fixtures, one photo edited, one mask |

## 7. Open questions
None.
