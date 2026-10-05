---
name: headless-verify
description: >-
  Opens the real ziv app (ZivApp) headless via egui_kittest, drives it, and
  reports PASS/FAIL with evidence (PNG capture + accessibility-tree dump in
  verify-artifacts/<run-id>/, gitignored). Without an argument, verifies the
  uncommitted changes; with an argument, verifies the given instructions.
  Ephemeral: writes no persistent test. Use to confirm a UI, keyboard, viewport
  or rendering change works in the real app without opening a window.
argument-hint: "[natural-language test instructions]"
---

# headless-verify

- **No argument** ⇒ target is the uncommitted diff.
- **Argument** ⇒ target is the instruction.

The harness is a throwaway file, run then deleted. Harness and evidence folder
share one run-id, so parallel sessions never collide.

## Prerequisites (in place)
- Cargo feature `headless-verify = ["egui_kittest/eframe", "egui_kittest/wgpu"]`.
- wgpu renders headless on this machine (Metal).

## Procedure

### 0. Mint the run-id
```sh
HV_ID="$(date +%Y%m%d_%H%M%S)_$$"; echo "$HV_ID"
```
Reuse the printed value literally in every step (shell state does not persist).
A `tests/headless_verify_scratch_*.rs` without **this** id belongs to another
session: never read, overwrite or delete it.

### 1. Determine the scenario
- Argument ⇒ translate it into interactions + observable assertions.
- No argument ⇒ `git status --short`, `git diff`, `git diff --cached`; map touched
  files to interactions. Empty diff ⇒ report it and run a smoke test.

### 2. Check it compiles
```sh
cargo check --features headless-verify --tests
```
Compile error ⇒ that is the result: FAIL, clean up, stop.

### 3. Write the harness
`tests/headless_verify_scratch_<HV_ID>.rs`. Verified skeleton (egui_kittest 0.36):

```rust
#![cfg(feature = "headless-verify")]

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use ziv::app::ZivApp;

#[test]
fn headless_verify() {
    let session_dir = std::env::var("HV_SESSION_DIR").expect("HV_SESSION_DIR set by the skill");
    std::fs::create_dir_all(&session_dir).unwrap();
    // Never the real catalog: ZivApp::new reads and writes the user's own.
    let data_folder = std::path::PathBuf::from(format!("{session_dir}/data"));

    let mut harness = Harness::builder()
        .with_size(egui::vec2(1280.0, 800.0))
        .with_os(egui::os::OperatingSystem::Mac)
        .wgpu()
        .build_eframe(|cc| ZivApp::keeping_data_in(cc, data_folder.clone()));
    harness.run();

    let tree = format!("{:#?}", harness.root());
    std::fs::write(format!("{session_dir}/headless_verify.a11y.txt"), &tree).unwrap();

    // --- scenario-specific interactions and assertions ---
    assert!(harness.query_by_label("No photos yet").is_some(), "empty state missing");

    let img = harness.render().expect("render wgpu");
    img.save(format!("{session_dir}/headless_verify.png")).expect("save png");
}
```

- One capture per observed state (`before.png`, `after.png`): `render()` again after an interaction.
- An isolated component: `Harness::new_ui(|ui| …)` on the `pub` render function.

### 4. Run
```sh
HV_SESSION_DIR="verify-artifacts/<HV_ID>" \
  cargo test --features headless-verify --test headless_verify_scratch_<HV_ID> -- --nocapture
```

### 5. Look at the evidence
`verify-artifacts/<HV_ID>/`: open every `*.png` with the Read tool and actually
look at it; read the `*.a11y.txt`. A passing assertion with a wrong picture is a FAIL.

### 6. Clean up (always, pass or fail)
```sh
rm -f tests/headless_verify_scratch_<HV_ID>.rs
```
Never a glob. Never delete `verify-artifacts/`.

### 7. Report
PASS / FAIL · what was verified · PNG paths · relevant a11y excerpt · panic message if any.

## Seeding state
`ZivApp::keeping_data_in(cc, folder)` keeps the catalog in `folder`: empty
folder, empty library; a folder a previous harness wrote to, the series it left
(a relaunch). `ZivApp::new(cc)` uses this Mac's data folder: never in a harness.
Native dialogs (file picker) hang
a headless run: never drive them. State is injected through a constructor seam:

- `.opening(&paths)` chained on it — the app as if the user had opened those files
  or folders (`tests/fixtures/…`, `tests/fixtures/local/DSC07070.ARW` when
  present). A drop is injected with `harness.input_mut().dropped_files.push(…)`
  (an `Arc` of a small `egui::DroppedFile` impl), then `harness.step()`. Loading is asynchronous:
  the first frame shows `Loading <name>…`; `harness.step()` in a loop, with a
  deadline, until the label `Photo` appears. `harness.run()` panics while the
  spinner is on screen (it repaints forever).

  An import of a new series lands in Cull mode (`specs/culling.md` rule 4):
  `harness.key_press(egui::Key::D)` then `harness.step()` for Develop mode.

- `.trashing_with(trash)` chained on it — files "moved to the Trash" go to
  `trash` (any `PhotoTrash`, e.g. one renaming them into a folder of the run)
  instead of this Mac's Trash: never let a harness reach the real one.

- `.exporting_to(folder)` chained on it — as if the export destination had
  already been chosen (the folder picker cannot be driven). Export settings are
  not persisted in a headless run: there is no eframe storage.

An edit made in a harness is stored as `<photo>.ziv.json` beside the path the
photo was opened from, and the next run opens it already edited. Never edit a
fixture in place: copy it into `verify-artifacts/<HV_ID>/` and open the copy
(a symlink there is enough for the large local RAW — the sidecar lands beside
the symlink, not beside its target).

`.wgpu()` on the builder is required: `ZivApp` takes its device from
`cc.wgpu_render_state`, which the default lazy renderer does not provide.

## What this proves about pixels
The capture shows the photo viewport as the engine rendered it, so a visual
regression is visible to the eye. It is not a pixel assertion: exact image
output is asserted by golden tests (`specs/testing.md` §5).

## egui_kittest cheatsheet (0.36)
| Need | Call |
|---|---|
| Whole app | `Harness::builder().with_size(..).with_os(Mac).wgpu().build_eframe(\|cc\| ZivApp::keeping_data_in(cc, folder))` |
| Component | `Harness::new_ui(\|ui\| component(ui, …))` |
| Component + state | `Harness::new_ui_state(\|ui, state\| …, state)` |
| Advance a frame | `harness.run()` |
| Presence, no panic | `harness.query_by_label("X").is_some()` |
| Get, panic if absent | `harness.get_by_label("X")` |
| Click | `harness.get_by_label("X").click(); harness.run();` |
| Type | `harness.get_by_label("X").type_text("…")` |
| Keyboard | `harness.key_press(egui::Key::J)` / `key_press_modifiers(mods, key)` |
| Pointer on canvas | `harness.hover_at(pos)` / `harness.drag_at(pos)` |
| App state | `harness.state()` |
| a11y dump | `format!("{:#?}", harness.root())` |
| PNG | `harness.render().unwrap().save(path)` |

## Guardrails
- Never commit a scratch harness or `verify-artifacts/`.
- This skill observes the app; it does not modify application code.
- `headless-verify` stays out of the default `cargo test` loop.
