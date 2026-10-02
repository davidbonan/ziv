# ADR 0008 — Enhancement model and pipeline

- **Status**: accepted; the format of the kept result is superseded by 0009, the model and its processor by 0010
- **Date**: 2026-10-02

## Context
M6 removes noise and strengthens detail on the whole photo
(`specs/enhance.md`). The author chose: one computation on demand, an
intensity slider afterwards, the result kept in a file beside the photo. ziv is
for personal use (ADR 0007). The runtime is ONNX Runtime on the CPU (ADR 0006).

Spike on this machine (Apple M5 Pro, `ort` 2.0.0-rc.13, release build), on a
crop of a night photo with fine lights, as it is and with added noise (σ = 12
of 255):

| Model | File | Licence | 512² tile, CPU | Result |
|-------|------|---------|----------------|--------|
| SCUNet real PSNR (`Heliosoph/scunet-onnx`) | 3.8 + 73.1 MB | Apache-2.0 | 2.1 s | noise gone, small lights and edges kept, nothing invented |
| SCUNet real GAN (same repository) | 3.8 + 73.1 MB | Apache-2.0 | 2.1–2.9 s | slightly crisper, texture invented |
| NAFNet deblurring (`opencv/deblurring_nafnet`) | 91.7 MB | MIT | 1.1 s | on the noisy crop: blocks of coloured garbage; on the clean one: a painterly smoothing |

SCUNet by tile size, CPU: 256² 0.53–0.75 s, 384² 1.15 s, 512² 2.1 s, 640²
3.2–3.8 s, 1024² 26–42 s — about 8 s per megapixel up to 640², far worse
beyond. CoreML execution provider: no result after 180 s for one 512² tile.
SwinIR denoising (σ = 25 only), Restormer: not evaluated.

## Decision
| Concern | Choice |
|---------|--------|
| Model | SCUNet real PSNR, pinned by revision and SHA-256 like the zone models; its graph and its weights are two files kept side by side |
| Tiles | 512², overlapping by 32 pixels, blended across the overlap; the photo's border mirrored to fill the last tiles |
| What the model sees | the working image, each channel divided by the photo's ceiling (its largest value, at least 1) and raised to 1/2.2: reversible, so the result goes back to the working space |
| Detail | after the model, an unsharp mask on the denoised result: sharpening no longer amplifies noise |
| In the pipeline | the enhanced working image is a second source; the display stage mixes source and enhanced by the intensity before any adjustment |
| Kept result | beside the photo, `<file name>.ziv.enhanced`: the difference between enhanced and original in the model's encoding, 8 bits a channel, as a JPEG after a short header (format version, photo size, ceiling). The original keeps its own precision |
| Thread | one enhancement at a time on its own thread; it re-decodes the photo itself |

## Alternatives rejected
- **SCUNet GAN** — invents texture; a RAW developer should not.
- **NAFNet deblurring**, alone or after SCUNet — see the spike; and a second
  model doubles the minutes.
- **Tiles of 1024²** — five times slower per pixel.
- **The enhanced image itself in the file** — 16 bits a channel is some
  150 MB a photo; 8 bits would band under strong edits.
- **Mixing on the CPU and uploading the result** — the intensity slider would
  re-upload 33 megapixels at each move.
- **A cache in the app's data folder** — the author chose beside the photo.

## Consequences
- An A7 IV photo (32.7 MP) takes about 5 minutes; progress and Cancel are
  part of the feature.
- The GPU holds a second full-size texture for an enhanced photo.
- A photo whose file is deleted is simply not enhanced any more: nothing else
  is lost.
- The model store learns that a model can be two files.
