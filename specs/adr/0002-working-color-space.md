# ADR 0002 — Working color space and display transform

- **Status**: accepted
- **Date**: 2026-10-02

## Context
The engine works on scene-referred linear float pixels (`image-pipeline`
invariant 1). Two things were still open and block M1: which RGB primaries those
pixels are expressed in, and how they are encoded for the screen.

Facts gathered on `rawler` 0.8.0 (source in the cargo registry):
- `RawDevelop`'s `Calibrate` step converts camera RGB straight to **linear sRGB**
  (`src/imgop/raw.rs:194`, `SRGB_TO_XYZ_D65` is hardcoded) and clips every pixel
  (`clip_euclidean_norm_avg`, `src/imgop/raw.rs:212`). Negative components —
  colors outside sRGB — are discarded there.
- The steps are selectable (`RawDevelop { steps }`, `ProcessingStep`), and the
  camera matrix is reachable (`RawImage::color_matrix_find_first`, `wb_coeffs`).
  Demosaiced, uncalibrated camera RGB is therefore available.
- `rawler` ships `XYZ_TO_SRGB_D65`, `XYZ_TO_PROFOTORGB_D50` and a Bradford
  adaptation, but no Rec.2020 matrix.

## Decision
| Concern | Choice |
|---------|--------|
| Working primaries | Rec.2020 (ITU-R BT.2020), white point D65 |
| Working encoding | linear light, `f32`, unbounded — no clamp |
| RAW input | `rawler` up to demosaic + crop, **without** its `Calibrate` and `SRgb` steps; ziv applies white balance and camera → XYZ(D65) → Rec.2020 itself |
| Standard image input | decoded as sRGB, linearized with the sRGB EOTF, converted Rec.709 → Rec.2020. Embedded ICC profiles are ignored for now |
| Display transform | last stage: Rec.2020 → Rec.709 primaries, clamp to [0, 1], sRGB OETF (IEC 61966-2-1 piecewise) |

Matrices and transfer functions are pure Rust in the domain, derived from the
published chromaticities, unit-tested; the shader mirrors them
(`image-pipeline` invariant 7).

## Alternatives rejected
- **Linear sRGB / Rec.709** — what `rawler` produces out of the box, but camera
  colors outside the gamut go negative and are clipped before any adjustment
  can use them.
- **Linear ProPhoto RGB (ROMM), D50** — Lightroom's internal space. Two of its
  primaries are not physical colors, and its D50 white point adds a chromatic
  adaptation on both the input and the display side. Parity with Lightroom's
  internal numbers is not a goal.
- **ACEScg (AP1)** — comparable gamut, white point ~D60: an adaptation on every
  path, for an ecosystem (ACES) ziv does not use.
- **A color-management crate (`moxcms` 0.9.1, `palette` 0.7.7)** — not needed for
  two fixed 3×3 matrices and one transfer function; revisited when ICC input or
  output profiles are specified.

Image quality differences between these spaces were not evaluated by a spike.

## Consequences
- ziv owns the camera → working conversion; `rawler` is used as a decoder and
  demosaicer only.
- D65 everywhere on the display side: no chromatic adaptation between working
  space and sRGB / Display P3.
- The display transform has no tone curve: a RAW opened in M1 is shown as
  linear scene values encoded to sRGB, highlights above 1.0 clip. The base tone
  curve is M2's subject.
- The screen is treated as sRGB. How the macOS surface is tagged on a wide-gamut
  panel, and a Display P3 output, were not evaluated; a later ADR settles it.
- Out-of-sRGB colors are clipped per channel at display, with no gamut mapping.
