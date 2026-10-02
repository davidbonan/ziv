---
name: image-pipeline
description: >-
  Reference for ziv's image engine: pipeline invariants (linear float working
  data, edits as parameters, masks as coverage, one code path for preview and
  export) and the procedure to add an adjustment, a mask type or a shader with
  its golden test. Load before writing or reviewing anything in the engine, a
  WGSL shader, color math, RAW decoding output, masks, or export.
---

# image-pipeline

The engine does not exist yet (M1). This file holds the invariants every engine
task must respect; file paths and concrete types are added here as they land.

## Invariants

1. **Scene-referred linear float.** From RAW decode to the last adjustment,
   pixels are linear-light floating point. No clamp to [0, 1] and no gamma
   before the display transform — clipping early destroys highlight recovery.
2. **The display transform is last and separate.** Encoding to the screen's or
   the export's color space is one final stage, never mixed into an adjustment.
3. **Edits are data.** An edit is a serializable set of parameters; the source
   file is never written. Default parameters produce the identity.
4. **Fixed stage order.** Stages run in one documented order, independent of the
   order the user touched the sliders. The order is written here when M2 specifies it.
5. **A mask is a coverage in [0, 1].** A local adjustment is the same adjustment
   as the global one, blended by the mask. No second implementation of the math.
   Brush, gradient, shape and segmentation all produce the same kind of mask.
6. **One code path.** Preview and export run the same stages; preview only
   changes resolution. The engine takes a `wgpu::Device`, never a window.
7. **Color math is in the domain.** Constants and formulas (matrices, transfer
   functions) are pure Rust, unit-tested, and the shader mirrors them — never a
   magic number only in WGSL.
8. **Nothing heavy on the UI thread**: decode, inference, full-resolution export.

Working primaries and the display transform are an open decision
(`specs/overview.md` §4): settle it with `/adr` before the first engine task.

## Adding an adjustment
1. Spec says what the slider does and its range (`/spec`), with the Lightroom
   reference behavior when parity is wanted.
2. Parameter in the edit model, default = identity.
3. Math as a pure function in the domain + unit tests on known values
   (identity at default, monotonicity, a hand-computed point).
4. Shader stage mirroring that function.
5. Golden test: small synthetic input, the parameter at a non-default value
   (`specs/testing.md` §5). Look at the image before committing it.
6. UI control with an accessible label; `Eu` test that moving it emits the intent.
7. `/headless-verify` on the real app.

## Adding a mask type
Produce a coverage; reuse the existing blend. Tests: `U` on the geometry
(coverage at known points), `G` on one adjustment through the mask.

## Review checklist
- Any `clamp`, `saturate` or 8-bit conversion before the display transform?
- Any formula living only in WGSL?
- Does default = identity, proven by a test?
- Does export produce what the preview showed?
