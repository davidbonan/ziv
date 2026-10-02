# ADR 0005 — Mask geometry and rendering

- **Status**: accepted
- **Date**: 2026-10-02

## Context
M4 adds masks: gradients, shapes and a brush, each carrying the adjustments of
the develop panel (`specs/local-adjustments.md`). The pipeline's invariants
already say a mask is a coverage in [0, 1] and that a local adjustment is the
global one blended by it, on one code path for preview and export. What was
left to settle: where coverage is computed, in which coordinates masks live,
how several masks reach the display stage, and how they are stored.

Facts: the display stage is one fragment shader fed by one uniform block; an
A7 IV photo is 7008 × 4672; export renders the same stage in strips. `wgpu` 30
default limits allow a 64 KiB uniform buffer and 2D array textures of 256
layers. No performance spike was run: frame cost with 16 masks is **not
evaluated** and is checked by the milestone's demo.

## Decision
| Concern | Choice |
|---------|--------|
| Coordinates | photo units: pixels of the oriented photo divided by its long edge. Isotropic, independent of the render size |
| Gradients and shapes | coverage computed analytically in the display stage, per pixel, from parameters in the uniform block — exact at any zoom and export size |
| Polygon | at most 16 corners, distance to the outline evaluated in the shader |
| Brush | strokes are the stored data; they are rasterized on the CPU into an 8-bit coverage image whose long edge is at most 2048, uploaded as one layer of a 2D array texture, sampled bilinearly |
| Blend | after the photo's adjustments and before the conversion to display primaries, for each visible mask in order: `colour = mix(colour, developed(colour, mask adjustments), coverage)`, with the same shader functions as the global stage |
| Limit | 16 masks per photo: a fixed-size array in the uniform block |
| CPU reference | the domain computes the same coverage and blend; golden tests compare the shader to it |
| Overlay | drawn by the display stage from the same coverage, after encoding; never requested by export |
| Storage | a `masks` array in the sidecar edit; the sidecar `version` becomes 2, because a version 1 reader would drop the masks and overwrite the file |

## Alternatives rejected
- **Every mask rasterized to a texture** — one path for all types, but a hard
  rectangle edge stored at 2048 pixels is blurred in a 7008-pixel export, and
  each handle drag re-rasterizes and re-uploads an image.
- **Brush coverage at full photo resolution** — 33 MB per brush mask for an
  A7 IV photo; a brush edge is soft by nature and survives bilinear upsampling.
- **One render pass per mask (ping-pong)** — lifts the limit of 16, at the cost
  of float intermediate targets at output size and a second code path for the
  strip renderer. Not needed at this scale.
- **Storing the rasterized brush coverage** — binary data in a JSON sidecar,
  and strokes could no longer be re-rendered at another resolution.

## Consequences
- The uniform block grows by the mask array; a photo without mask loops zero
  times.
- A brush stroke re-rasterizes only the dabs it adds (erasing included: strokes
  are replayed in order when a stroke is undone).
- Segmentation masks (M5) arrive as another raster coverage layer, like the
  brush.
- Sidecars with thousands of stroke points grow to tens of kilobytes; accepted.
