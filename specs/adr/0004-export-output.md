# ADR 0004 — Export output and app settings

- **Status**: accepted
- **Date**: 2026-10-02

## Context
M3 writes developed photos to files and must remember export settings between
launches. Open before M3: the color encoding of exported files (`overview.md`
§4, "ICC profiles"), how a full-resolution render gets out of the GPU, and
where app settings live.

Facts checked on this machine, 2026-10-02:
- `image` 0.25 (already a dependency, features `jpeg`, `png`) encodes JPEG with
  a quality setting and PNG from RGB8 buffers.
- The engine's display stage renders any source region at any output size
  (`DisplayRequest`), the same code path the viewport uses.
- `eframe` 0.36 has a `persistence` feature: a key-value store handed to the
  app at creation and saved on exit and periodically.
- The A7 IV photo is 7008 × 4672; `wgpu`'s default maximum texture dimension is
  8192, which larger sensors exceed.

## Decision
| Concern | Choice |
|---------|--------|
| Color encoding | sRGB: the engine's display transform, 8 bits per channel |
| Color profile | none embedded; an untagged file is read as sRGB by viewers |
| Render | the display stage, in horizontal strips of the output assembled on the CPU, so no texture exceeds the device limit |
| Resize | the display stage renders directly at the output size from the source's mip chain |
| Encoding | `image` crate encoders: JPEG (quality 1–100), PNG |
| Thread | one export worker; decode, upload, render, encode and write all off the UI thread |
| App settings | `eframe` persistence, one serialized settings value under one key |

## Alternatives rejected
- **Embedding an sRGB ICC profile** — more robust with color-managed software,
  but needs a profile file to ship or a hand-built one; a malformed profile is
  worse than none. Left to the wide-gamut / ICC decision still open.
- **A separate CPU export path** — a second implementation of the pipeline,
  against `image-pipeline` invariant 6.
- **One full-size render target** — fails on sensors larger than the device's
  texture limit.
- **A settings file of our own** (`~/Library/Application Support/ziv`) — what
  `eframe` persistence already does, with one more dependency for the folder.

Quality of the mip-based downscale against a dedicated resampling filter was
not evaluated.

## Consequences
- Exports are limited to the sRGB gamut and 8 bits; colors outside it clip as
  they do on screen.
- 16-bit and wide-gamut export need a second output format in the display
  stage: a later decision.
- The settings store is unavailable in headless tests; the app must work with
  defaults when it is absent.
