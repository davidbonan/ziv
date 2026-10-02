# ADR 0001 — Technical stack

- **Status**: accepted
- **Date**: 2026-10-02

## Context
ziv is a Lightroom-like RAW editor. It needs real-time preview of a pixel
pipeline on large images, RAW decoding, later ML inference, and a UI that can be
tested end to end without a window. Rust was a given.

## Decision
| Concern | Choice | Version checked on crates.io, 2026-10-02 |
|---------|--------|------------------------------------------|
| UI | `eframe` / `egui` | 0.36.2 |
| GPU | `wgpu`, through eframe's wgpu backend | 30.0.1 |
| RAW decoding | `rawler` | 0.8.0, LGPL-2.1 |
| UI tests | `egui_kittest` | 0.36.2 |
| Platform | macOS Apple Silicon first | — |
| Layout | single crate, lib + bin | — |

Edits are non-destructive: the source file is never written.

`eframe` is built without `accesskit`: on macOS the first accessibility query
makes egui rebuild the full tree every frame for the life of the process.
`egui_kittest` re-enables it for tests only.

## Alternatives rejected
- **iced** — wgpu too, more structure, more verbose; last release 0.14.0 dates
  from 2025-12. egui + `egui_kittest` is already practised on helm, test loop included.
- **Tauri + web front** — the GPU viewport would sit under a webview or the
  pipeline would be duplicated in WebGPU on the JS side.
- **Slint** — not evaluated in depth.
- **LibRaw through FFI** — widest camera coverage, but a C++ toolchain and a
  home-made binding (`libraw-rs` on crates.io is stuck at 0.0.4).

## Consequences
- The engine and the UI share one `wgpu::Device`; the engine stays usable
  without a window, which is what makes golden-image tests and export one code path.
- The whole app renders headless (`Harness::build_eframe` + `render()`): proven
  on this machine, PNG 1280×800 produced on Metal.
- `rawler` is LGPL-2.1: a closed-source distribution must allow relinking.
  Tracked as an open decision in `specs/overview.md` §4.
- Camera coverage is `rawler`'s. Not yet tested against the author's own RAW files — first task of M1.
- Windows / Linux are not tested.
