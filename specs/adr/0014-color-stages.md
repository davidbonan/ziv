# ADR 0014 — Color stages: tone curve, color mixer, color grading

- **Status**: accepted
- **Date**: 2026-10-05

## Context
M11 adds three tools to the photo's edit (`specs/curve-and-color.md`): a point
curve per channel, the color mixer and color grading. What was left to settle:
where each sits in the display stage, in which values it computes, and how a
curve of any number of points reaches the shader.

Facts: the display stage is one fragment shader; adjustments and masks run in
the working space, then display primaries, base rendering, clamp and encoding
(`display_stage.wgsl`). The pipeline's invariant 4 says adjustments go before
the conversion to display primaries. The curve's graph is drawn over the
histogram, which counts display levels (`histogram.md` rule 2); the base
rendering of a RAW sits between the working values and those levels. A mask
carries `Adjustments` only, applied by the one `adjusted()` function.

No spike was run: the frame cost of the three stages and the look of the color
ranges are **not evaluated**; the milestone's demo (T19) checks both.

## Decision
| Concern | Choice |
|---------|--------|
| Order | enhancement → adjustments → masks → **color mixer** → display primaries → base rendering → clamp → encoding → **color grading** → **tone curve** → overlay |
| Scope | the three stages run once, on the photo after its masks: a masked area gets them like the rest. Never part of `Adjustments`, so never of a mask |
| Tone curve values | display-encoded values in [0, 1], one channel at a time: the graph's axes are the histogram's levels |
| Tone curve shape | monotone cubic interpolation between points (Fritsch–Carlson): through every point, no overshoot, so it stays in the graph without a clamp; flat outside the end points |
| Tone curve in the shader | the domain samples the four curves into one 256-texel RGBA float texture, each channel holding its own curve applied after the RGB curve; sampled linearly. An edit whose curves are all the identity skips the stage |
| Color mixer values | Oklch of the working-space pixel, before the display primaries: scene-referred, unbounded |
| Color ranges | eight centres at the Oklch hues of the named colors; a pixel's share of the two nearest ranges is a smooth partition of unity over hue. Hue and Luminance fade to nothing as chroma goes to zero; Saturation needs no fade, scaling a chroma of zero changes nothing |
| Color mixer effect | Hue turns the Oklch hue toward the neighbouring centre, 30 % of the way at ±100: under a third, hues keep their order when two neighbours turn toward each other. Saturation scales chroma, Luminance scales the light of the pixel |
| Color grading values | display-encoded values, after clamp: a zone's weight comes from the Rec.709 luma the histogram counts; Balance moves the pivot, Blending the width of the overlap |
| Color grading effect | each zone adds its tint — the wheel's color minus its luma, scaled by Saturation and the zone's weight — and its Luminance; Global weighs every tone 1 |
| CPU reference | `Development::to_display` computes the same three stages; goldens compare the shader to it |
| Storage | curves, color mixer and color grading as fields of the sidecar edit beside `adjustments`; the sidecar `version` goes up by one |

## Alternatives rejected
- **The three tools in the working space, before the masks** — keeps
  invariant 4 untouched, but the curve would no longer act on the tones the
  histogram shows: on a RAW the graph's white would not be display white.
- **Curve points in the uniform block, evaluated per pixel** — forces a limit
  on the number of points and a spline evaluation per pixel and per channel; a
  256-texel lookup costs one sample.
- **Natural or Catmull-Rom splines** — smoother at a point, but overshoot
  between two close points and leave the graph; clamping them flattens the
  curve where the user did not ask for it.
- **HSV hue on working RGB for the color mixer** — less math, but turning a
  hue changes the perceived lightness (blue toward purple darkens), to be
  compensated by hand.
- **Color grading in the working space** — zones would be cut on scene light,
  not on the tones seen: "shadows" of a RAW would not be the left of the
  histogram.

## Consequences
- Invariant 4 is amended: color grading and the tone curve are the two
  adjustments that run on display-encoded values, after the clamp. They cannot
  bring back light the base rendering clipped; Highlights and Whites still do,
  upstream.
- `specs/curve-and-color.md` rule 2 is reworded: the three tools apply after
  the masks, not under them.
- The display stage gets one more texture binding (the curve lookup) and a few
  rows of uniforms; no limit on the number of curve points.
- A curve edit re-samples and re-uploads 256 texels, per change.
- The histogram and the export need nothing: they read the same stage.
- Oklab needs the working space → LMS matrices in the domain, mirrored in the
  shader through uniforms.
