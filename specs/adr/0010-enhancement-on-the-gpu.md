# ADR 0010 — Enhancement model on the GPU

- **Status**: accepted
- **Date**: 2026-10-02

## Context
With ADR 0008 (SCUNet real PSNR, CPU) an A7 IV photo took about five minutes.
The author: minutes are not acceptable. ADR 0006 runs every model on the CPU
because the Core ML execution provider never finished creating a session for
the zone models.

Spike on this machine (Apple M5 Pro, `ort` 2.0.0-rc.13, release build), one
512² tile, model load excluded:

| Model | Licence | File | CPU | Core ML, GPU | Core ML, Neural Engine |
|-------|---------|------|-----|--------------|------------------------|
| SCUNet real PSNR | Apache-2.0 | 77 MB | 1.9–2.7 s | 9–56 s | not evaluated |
| DRUNet colour (`synthscript/drunet-color-onnx`, the KAIR weights unchanged) | MIT | 131 MB | 1.0 s | 0.10 s | 1.0–1.1 s |

Core ML takes a model only when every dimension is fixed: with DRUNet's batch
left free it silently ran on the CPU. DRUNet at 1024² on the GPU failed.
DRUNet is made of convolutions only; SCUNet's transformer blocks are what the
GPU runs badly. On the spike's night crop with added noise (σ = 12 of 255),
DRUNet told that level cleans flat areas at least as well as SCUNet and keeps
the backdrop's sparkles better. Its distilled 1 M-parameter student
(`seantempesta/remaster-drunet`, half-float input, trained on video
compression) and the RAW-domain denoiser of RapidRAW (works before demosaic,
which ziv's pipeline has already done) were not evaluated.

The A7 IV RAW (32.7 MP) through the whole path — decode, 150 tiles, file
written — in the real app, release build: 18–20 s, the window drawing every
frame (longest 2 ms). Reopening it enhanced: 0.5 s.

## Decision
| Concern | Choice |
|---------|--------|
| Model | DRUNet colour, pinned by revision and SHA-256; one file |
| Processor | the GPU through the Core ML execution provider, the model's dimensions fixed at one 512² tile; the CPU when Core ML refuses, without a message |
| Noise level | DRUNet is told how much noise to remove: the deviation measured on the photo's green channel in the model's encoding (median of the finest diagonal detail), at most 50 / 255 |
| Rest | tiles, encoding, unsharp mask, file and intensity as in ADR 0008 and 0009 |

The zone models stay on the CPU (ADR 0006).

## Alternatives rejected
- **SCUNet on the GPU** — slower than on the CPU.
- **DRUNet with a fixed noise level** — smears a clean photo or leaves a noisy
  one noisy; the Intensity slider would have to repair it by hand.
- **Larger tiles** — 1024² fails on the GPU and gains nothing on the CPU.

## Consequences
- The model sees the same noise level over the whole photo: shadows noisier
  than the rest keep some noise. A level per region is **not evaluated**.
- The measure assumes noise finer than the picture's detail; a demosaiced RAW
  has smoother noise, so it is under-measured rather than over.
- DRUNet's training sets include DIV2K, published for research only; ziv is
  for personal use (ADR 0007). To settle with the licences before any
  distribution.
- A model is one file again: the two-file store of ADR 0008 is removed.
