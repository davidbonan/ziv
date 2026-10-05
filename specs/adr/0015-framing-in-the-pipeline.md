# ADR 0015 — Framing in the pipeline

- **Status**: accepted
- **Date**: 2026-10-05

## Context
M12 adds the framing of a photo (`specs/crop-and-straighten.md`): a crop
frame, a straighten angle, quarter turns and a mirror. What was left to settle:
where the framing applies in the pipeline, in which coordinates it and the
masks live, how a turned picture is resampled, and how it is stored.

Facts: the display stage is one fragment shader working pixel by pixel; it
samples the source at a texture coordinate and computes every mask from that
same coordinate (`point = uv × photo extent`). A `DisplayRequest` already names
the part of the source to render, as an upright rectangle. Masks are in photo
units of the picture (ADR 0005). Zone detection and enhancement render or read
the whole picture. The source has mipmaps and is sampled trilinearly.

No spike was run: the sharpness of a straightened photo and the frame cost of
a turned sampling are **not evaluated**; criteria F9 and F20 check both.

## Decision
| Concern | Choice |
|---------|--------|
| Where | the framing is not a stage: it is the part of the source a render asks for. The region of a `DisplayRequest` becomes a parallelogram of the source — an origin and two axes in texture coordinates — and the framing of the edit, with the picture's size, says which one |
| Order | every stage is per pixel, so sampling the picture through the framing gives what framing the developed picture would give: the framing "comes last" without a second pass |
| Masks | unchanged: photo units of the picture. The stage keeps computing them from the sampled coordinate, so they follow the picture under any framing |
| Whole-picture readers | zone detection, enhancement and thumbnails keep asking for the whole source: the framing never reaches them |
| Frame | centre and size as fractions of the picture's width and height, so that the default — the whole picture — is the same for every photo |
| Angle | degrees, the picture turned clockwise under the frame around the frame's centre, in the picture's own orientation |
| Quarter turns and mirror | one of the eight orientations — a horizontal mirror, then 0 to 3 clockwise quarter turns — applied to the framed result. Frame, angle and masks are stored before it: a turn or a mirror changes that one value and nothing else |
| Size of the framed photo | the frame's size in pixels of the picture, rounded, sides swapped by an odd number of quarter turns |
| Resampling | the source's trilinear sampler, for the viewport and the export alike. A photo with an angle is never shown pixelated: its pixels are not the source's |
| Outside the picture | a pixel sampled more than a texel outside the source is transparent: crop mode shows the turned picture on the window's background |
| Before | the region comes from the photo's framing while the development is the one without edit: Before keeps the framing with no special case |
| Storage | a `framing` object in the sidecar edit; the sidecar `version` goes up by one, because an earlier reader would drop the framing and overwrite the file |

## Alternatives rejected
- **A second pass that frames the developed picture** — needs an intermediate
  target at picture size (7008 × 4672 in float or 8 bits) for every viewport
  frame, and a second code path for the strips of the export.
- **Frame and masks stored in the turned picture's coordinates** — a quarter
  turn or a mirror would rewrite the frame, the angle and every mask, brush
  strokes and zone coverages included.
- **Baking quarter turns into the decoded picture** — every turn would decode
  and upload the photo again, and invalidate zone masks and the enhancement
  file, which match the picture's size.
- **Bicubic or Lanczos resampling in the shader** — 16 taps or more on two
  textures per pixel; bilinear is what every other view of the photo already
  uses. Not measured; to revisit if F9 shows a soft result.

## Consequences
- `DisplayRequest` carries three vectors instead of two; the strip renderer of
  the export cuts the rows of the framed photo, not of the source.
- The viewport's `Viewport::photo` is the framed photo's size; pointer
  positions on the photo go through the framing to reach photo units, and mask
  handles come back through it.
- A rectangle or a gradient drawn on a straightened photo is aligned with the
  picture, not with the screen: shapes have no angle of their own beyond the
  radial gradient's.
- The histogram and the export render the framed region; nothing else changes
  in them.
- A straightened photo is one bilinear resampling away from the source: a
  slight softening at 100 %, accepted.
