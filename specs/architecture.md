# ziv — Architecture

## 1. Boundary
- **Domain** (image math, edit model, masks, RAW decoding, export): no `egui`,
  no `eframe`. `pub` from the lib.
- **Engine**: turns *source image + edit parameters* into pixels on `wgpu`.
  Takes a `wgpu::Device`, never a window — the same code path serves the
  viewport, the export and the tests.
- **UI**: `pub fn(&mut egui::Ui, …)` render functions. They read state and emit
  intents; they hold no business rule.
- **`app.rs`**: wires the modules to `eframe`. Kept thin.

## 2. Folder rule
Module → part of the feature → technical layer.

```
src/<module>/<part>/{domain,application,infrastructure,ui}/
```

- A module is a domain concept (`library`, …). Never `utils/`, `helpers/`, `common/`.
- Layers are leaves. `infrastructure/` and `ui/` may import `domain/`, never the
  reverse; `application/` depends only on ports declared in `domain/`.
- A module with a single part keeps its layers directly under the module
  (`src/library/ui/`) until a second part exists.

## 3. Current tree
```
src/main.rs                      eframe wrapper
src/lib.rs                       module declarations
src/app.rs                       ZivApp, run()
src/library/ui/empty_state.rs    empty library screen
tests/it/main.rs                 single integration binary
tests/it/ui_empty_state.rs       UI e2e
```

Modules are added by the milestone that needs them, not ahead of it.

## 4. Threads
Not designed yet. Constraint already known: RAW decoding and AI inference never
run on the UI thread. To be specified with M1.
