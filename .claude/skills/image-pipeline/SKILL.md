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

This file holds the invariants every engine task must respect.

## Where things are
- `src/color/domain/` — primaries, matrices, sRGB transfer, `SrgbInput`, `DisplayTransform`.
- `src/develop/domain/edit.rs` — `Edit`, the adjustments as the sliders show them.
- `src/color/domain/illuminant.rs` — `Illuminant` (kelvin + tint), Bradford
  adaptation between two of them.
- `src/develop/domain/white_balance.rs` — `WhiteBalance` as the sliders show it
  (kelvin on a RAW, relative on a standard image) → a working-space matrix
  relative to the as-shot balance baked in at decode; `None` in the edit = as shot.
- `src/develop/domain/tone.rs` — `Tone`: contrast, highlights, shadows, whites,
  blacks as math on luminance in stops around the photo's middle grey; mirrored
  by `develop()` in `display_stage.wgsl`.
- `src/develop/domain/base_rendering.rs` — `BaseRendering`: the tone curve a RAW
  gets (none for a standard image), mirrored by `base_rendered()` in the shader.
- `src/develop/domain/development.rs` — `Development` = photo kind + edit,
  what a `DisplayRequest` carries. The photo's middle grey (the tone pivot) comes
  from its base rendering. `Development::to_display` is the CPU
  reference of the whole stage and `tests/it/develop_golden.rs` asserts the
  shader agrees with it.
- `src/photo/domain/working_image.rs` — `WorkingImage`, the engine's input.
- `src/photo/domain/camera_calibration.rs` — camera RGB → working space. The one
  clamp before the display transform lives here: white-balanced camera values
  are clipped at sensor saturation so blown highlights stay neutral.
- `src/photo/infrastructure/raw_file.rs` — `rawler` up to demosaic + crop only;
  its `Calibrate` / `SRgb` steps are never used (ADR 0002).
- `src/engine/infrastructure/` — `Engine` (upload + mipmaps, `render_display`,
  readback), `display_stage.wgsl`; shader constants come from the domain through
  uniforms. A `DisplayRequest` names the source region (a `PictureRegion`: a
  parallelogram of the picture), the output size in pixels and the sampling:
  the viewport renders only what is on screen, at screen resolution.
- `src/develop/domain/framing.rs` — `Framing`: what a photo keeps of its
  picture. It is not a stage: `Framing::region` says which region a render
  asks for (ADR 0015) — frame, angle, then its `Turn` (mirror, quarter turns)
  last; masks stay in photo units of the picture. Zone detection, enhancement
  and thumbnails ask for the whole source; the viewport, the histogram and
  the export ask for the framed region. `Framing::share_of_framed_photo` is
  the way back, from the picture to the framed photo (mask handles on screen).
- `src/develop/domain/mask.rs` and the shape files beside it — `Mask` = shape +
  `Adjustments` + inverted + hidden; every shape answers `coverage(point)` in
  photo units (pixels ÷ long edge, `photo_extent`). `Development::edited` blends
  each visible mask's adjustments by its coverage, after the photo's own.
- `display_stage.wgsl` mirrors it: `adjusted()` is the one adjustment function,
  called for the photo then for each mask; gradients, rectangle and polygon are
  computed per pixel from uniforms; a brush or zone mask is a layer of a coverage
  texture (`coverage_layers.rs`, long edge ≤ 2048): a brush is rasterized by
  `BrushCoverage` from its strokes, a zone carries its `CoverageImage`. The overlay veil is mixed in after encoding (ADR 0005).
- `src/enhance/` — the enhancement of a photo (ADR 0008, 0009, 0010): computed once by a
  model, kept beside the photo, uploaded as a second working texture
  (`Engine::upload_enhancement`). The stage mixes it with the original by
  `Edit::enhancement_intensity` before anything else (`Development::enhanced`,
  `enhanced()` in the shader); a source without enhancement mixes nothing.
- `src/develop/domain/tone_curve.rs` — `ToneCurve`: points on display tones,
  monotone cubic between them; `ToneCurves::lookup` is what the stage samples
  (`curve_lookup.rs`, one row of 256 texels, `tone_curved()` in the shader).
  Applied after encoding, to the photo with its masks, never per mask (ADR 0014).
- `src/color/domain/oklab.rs` — Oklab of the working space (scene-referred,
  nothing clipped); its four matrices reach the shader through uniforms.
- `src/develop/domain/color_mixer.rs` — `ColorMixer`: Hue, Saturation and
  Luminance of eight color ranges, computed in Oklch. A color gets a share of
  the two ranges its hue sits between; Hue and Luminance fade out as chroma
  goes to zero, so a grey never moves. `Development::edited` applies it after
  the masks, `color_mixed()` in the shader (ADR 0014). Skipped at its default.
- `src/develop/domain/color_grading.rs` — `ColorGrading`: a tint and a
  luminance per tonal zone, on display-encoded values after the clamp. Shadows,
  midtones and highlights share every tone by its Rec.709 luma (Balance bends
  the luma, Blending widens the passages); Global weighs 1. `GradingFactors`
  is what the shader gets, `color_graded()` there (ADR 0014). Skipped when no
  zone holds a saturation or a luminance.
- `Engine::render_pixels` — the whole developed photo at any size, as pixels:
  what export writes. Same display stage, rendered in strips.
- `tests/it/gpu.rs`, `tests/it/golden.rs` — headless engine and golden compare.

## Invariants

1. **Scene-referred linear float.** From RAW decode to the last adjustment,
   pixels are linear-light floating point. No clamp to [0, 1] and no gamma
   before the display transform — clipping early destroys highlight recovery.
2. **The display transform is last and separate.** Encoding to the screen's or
   the export's color space is one final stage, never mixed into an adjustment.
3. **Edits are data.** An edit is a serializable set of parameters; the source
   file is never written. Default parameters produce the identity.
4. **Fixed stage order.** Stages run in one documented order, independent of the
   order the user touched the sliders: enhancement mixed in by its intensity → white balance → exposure → contrast → highlights and
   shadows → whites → blacks → vibrance and saturation → masks → color mixer →
   working to display primaries → base rendering →
   clamp and encode → color grading → tone curve. Extend this list with each
   adjustment; adjustments go before the conversion to display primaries,
   except color grading and the tone curve, which map display-encoded values
   (ADR 0014).
5. **A mask is a coverage in [0, 1].** A local adjustment is the same adjustment
   as the global one, blended by the mask. No second implementation of the math.
   Brush, gradient, shape and segmentation all produce the same kind of mask.
6. **One code path.** Preview and export run the same stages; preview only
   changes resolution. The engine takes a `wgpu::Device`, never a window.
7. **Color math is in the domain.** Constants and formulas (matrices, transfer
   functions) are pure Rust, unit-tested, and the shader mirrors them — never a
   magic number only in WGSL.
8. **Nothing heavy on the UI thread**: decode, inference, full-resolution export.

Working space: linear Rec.2020, D65. Display transform: Rec.2020 → Rec.709,
clamp, sRGB OETF (`specs/adr/0002-working-color-space.md`).

## Adding an adjustment
1. Spec says what the slider does and its range (`/spec`), with the Lightroom
   reference behavior when parity is wanted.
2. Parameter in `Edit`, default = identity.
3. Math as a pure function in the domain + unit tests on known values
   (identity at default, monotonicity, a hand-computed point).
4. Shader stage mirroring that function.
5. Golden test: small synthetic input, the parameter at a non-default value
   (`specs/testing.md` §5). Look at the image before committing it.
6. UI control with an accessible label; `Eu` test that moving it emits the intent.
7. `/headless-verify` on the real app.

## Adding a mask type
Produce a coverage; reuse the existing blend. A variant of `MaskShape` with
`coverage`, `handles`, `with_handle_at`; its rows in `shape_rows`
(`display_stage.rs`) and its branch in `shape_coverage` (WGSL), or a raster
layer like the brush. Tests: `U` on the geometry (coverage at known points),
`G` on one adjustment through the mask, with a feather: a sharp edge differs
by a pixel between CPU and GPU.

## Review checklist
- Any `clamp`, `saturate` or 8-bit conversion before the display transform?
- Any formula living only in WGSL?
- Does default = identity, proven by a test?
- Does export produce what the preview showed?
