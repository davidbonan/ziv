# ziv

Lightweight Lightroom-like **RAW photo editor**, native **macOS (Apple Silicon first)**, in **Rust**.
UI `eframe`/`egui`, image engine on `wgpu`, RAW decoding via `rawler`.
**lib + bin**: `src/lib.rs` (testable modules) + `src/main.rs` (thin `eframe` wrapper).

## Documentation
- `specs/overview.md` — goal, feature set, locked and open decisions, specs index.
- `specs/architecture.md` — modules, folder rule, engine/UI boundary, threads.
- `specs/export.md` — export to JPEG / PNG (M3).
- `specs/local-adjustments.md` — masks and their adjustments (M4).
- `specs/zone-masks.md` — masks detected by models: subject, sky, persons (M5).
- `specs/enhance.md` — noise removed and detail strengthened by a model (M6).
- `specs/catalog.md` — series catalog and its sidebar (M7).
- `specs/shell.md` — top bar, layout, develop panel controls (M8).
- `specs/histogram.md` — histogram and shooting data in the develop panel (M9).
- `specs/culling.md` — Cull mode: grid, preview, ratings, series filter, removal and Trash (M10).
- `specs/mockups/shell.html` — visual reference of the window, direction A.
- `specs/testing.md` — feedback loop: unit / business e2e / golden image / UI e2e.
- `specs/import.md` — F1: opening photos, filmstrip, viewport zoom and pan.
- `specs/develop.md` — F2: develop panel, base rendering, sidecar, undo, before/after.
- `specs/curve-and-color.md` — F2: tone curve, color mixer, color grading (M11).
- `specs/crop-and-straighten.md` — F11: crop frame, aspect ratios, angle, quarter turns, mirror (M12).
- `specs/update.md` — F12: app bundle, releases, in-app update, release notes (M13).
- `specs/ai-presets.md` — F13: one-click retouches from detected zones, intensity (M14).
- `specs/adr/` — one file per technical decision (`/adr`).
- `specs/<feature>.md` — product intent per feature (`/spec`).

## Implementation tracking — `specs/plan/`
**At the start of a dev session, read `specs/plan/STATE.md`.** Conventions and
*Definition of Done*: `specs/plan/README.md`. After each task, update `STATE.md`
(status only); the "why" lives in commit messages.

## Skills
- `/spec <feature>` — frame a feature into `specs/<feature>.md` + task cards in `STATE.md`. No code.
- `/adr <decision>` — record a technical decision in `specs/adr/`.
- `/implement-state [task]` — implement exactly one task from `STATE.md` through the DoD.
- `/release [patch|minor|major|x.y.z]` — publish a version: bump, release notes, tag, CI, published asset checked.
- `/verify` — quality gate: fmt, clippy `-D warnings`, tests.
- `/headless-verify [instructions]` — open the real app headless, drive it, PNG + a11y evidence.
- `image-pipeline` — reference: pipeline invariants, how to add an adjustment. Read before touching `engine`.

## Commands
```sh
cargo run                                  # launch the app
cargo test                                 # unit + business e2e + UI e2e
cargo test --lib                           # unit only
cargo test --test it <module>              # one integration module
cargo fmt
cargo clippy --all-targets -- -D warnings
```

## Rules
- Domain isolated from rendering: image math, edit model, RAW decoding never import `egui`/`eframe`.
- Rendering = `pub fn(&mut egui::Ui, …)` functions, drivable by `egui_kittest`.
- The engine renders without a window; the viewport only displays its output.
- A locked decision (`specs/overview.md`) changes only through a new ADR.
- No speculative abstraction; single crate until a real need forces a workspace.
