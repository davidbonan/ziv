# ADR 0006 — AI runtime and model delivery

- **Status**: accepted
- **Date**: 2026-10-02

## Context
M5 generates masks with segmentation models, M6 sharpens with one. The models
exist as ONNX files. The author chose that models are **downloaded at first
use**, not shipped in the app. What was left to settle: what runs the models,
on which processor, and how the files reach the machine.

Spike on this machine (Apple M5 Pro, release build, `ort` 2.0.0-rc.13 with its
prebuilt ONNX Runtime 1.28), six models, CPU execution provider:

| Model | Load | Inference |
|-------|------|-----------|
| BiRefNet lite, 1024² (224 MB) | 0.8–1.1 s | 2.5–5.7 s |
| BiRefNet lite fp16 (114 MB) | 1.1 s | 6.0–7.4 s |
| skyseg U²-Net, 320² (176 MB) | 0.05 s | 0.22–0.93 s |
| RF-DETR small, 512² (115 MB) | 0.12 s | 0.07–0.08 s |
| SegFormer B2 clothes, 512² (110 MB) | 0.1 s | 0.13–0.26 s |
| SegFormer face parsing, 512² (340 MB) | 0.3–0.5 s | 0.25–0.43 s |

CoreML execution provider: creating the sessions of the six models did not
finish in 10 minutes; BiRefNet lite alone did not finish in 150 s. Which
operators fall back was **not evaluated**.

Crates on crates.io, 2026-10-02: `ort` 2.0.0-rc.13 (MIT OR Apache-2.0),
`tract-onnx` 0.23.8, `candle-core` 0.11.0, `burn` 0.22.0-pre.4, `coreml-rs`
0.5.4, `ureq` 3.4.2, `sha2` 0.11.0, `dirs` 7.0.0 (all MIT OR Apache-2.0).

## Decision
| Concern | Choice |
|---------|--------|
| Runtime | `ort` 2.0.0-rc.13, pinned exactly: ONNX Runtime through its prebuilt binaries |
| Processor | CPU execution provider |
| Thread | one inference worker; never the UI thread |
| Model input | the photo developed with no edit (base rendering only), rendered by the engine at the size the model takes |
| Delivery | each model file is downloaded the first time its zone is asked for, from a pinned URL (repository revision), checked against a pinned SHA-256, and kept in the user's application data folder |
| Download | `ureq`, streamed to a temporary file, renamed once the checksum matches |

## Alternatives rejected
- **CoreML execution provider** — see the spike: session creation does not end
  in a usable time on these models.
- **`tract-onnx`** — pure Rust, no native library to ship; not evaluated on
  these models, whose operators (deformable attention, resize modes) are the
  usual gaps.
- **`candle`, `burn`** — need each architecture rewritten in Rust and the
  weights converted.
- **`coreml-rs`** — needs every model converted to Core ML first.
- **Models bundled in the app** — the author's choice is download at first use.

## Consequences
- `ort` is a release candidate: the version is pinned and upgrades are deliberate.
- The build downloads ONNX Runtime; the app ships its dynamic library
  (distribution is still an open decision).
- A subject mask takes seconds: the UI shows progress and stays usable.
- Through the store, the persons model (115 MB) downloaded and checked in
  4.5 s on this machine's connection.
- No network at first use means no zone mask: the failure is shown, nothing else breaks.
