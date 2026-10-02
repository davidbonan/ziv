# ziv — Tests & feedback loop

How to verify a change end to end without opening a window by hand.

## 1. Levels
| Code | Level | Target | Where | State |
|------|-------|--------|-------|-------|
| `U` | Unit | pure logic, no I/O | `#[cfg(test)]` in the module | operational |
| `Eb` | Business e2e | domain against a real resource (RAW file, disk, GPU device) | `tests/it/<domain>_e2e.rs` | operational |
| `G` | Golden image | engine output compared to a reference image | `tests/it/<domain>_golden.rs` + `tests/golden/` | operational |
| `Eu` | UI e2e | egui render function driven headless | `tests/it/ui_<component>.rs` | operational |
| `HV` | Whole-app check | real `ZivApp` headless, PNG + a11y tree | `headless-verify` skill (ephemeral) | operational |

Everything except `HV` runs under `cargo test`.

## 2. Structural prerequisite
Integration tests only see the lib's public API: what must be tested is `pub`
from `src/lib.rs`. All `tests/it/*.rs` are modules of one binary
(`tests/it/main.rs`) so `cargo test` links once.

## 3. Unit
Pure function, deterministic output, test next to the code.

## 4. Business e2e
Real disposable resource, public API, assertion. Fixtures in `tests/fixtures/`
(small files only; a RAW fixture must be redistributable). Temporary outputs go
through `tempfile`.

A RAW that cannot be redistributed lives in `tests/fixtures/local/` (gitignored,
usually a symlink). Its tests print `skipped` and pass when the file is absent —
on the author's machine `DSC07070.ARW` (Sony A7 IV) is linked there.

## 5. Golden image
The engine renders offscreen on a real `wgpu` device (Metal), the result is read
back and compared to `tests/golden/<name>.png` within a tolerance — GPU float
math is not bit-exact across drivers. One golden per adjustment at a
non-default value, on a small synthetic input.

- A golden is regenerated only by an explicit command, and the new image is
  looked at before it is committed.
- A golden test asserts pixels; a UI test never does.

Harness: `tests/it/golden.rs` (`assert_matches_golden`, tolerance 2/255 per
channel) on the engine from `tests/it/gpu.rs`. On a mismatch the actual render is
written to `target/tmp/<name>.actual.png`. Regenerate with:

```sh
ZIV_UPDATE_GOLDEN=1 cargo test --test it <module>_golden
```

## 6. UI e2e — egui_kittest
```rust
let mut harness = Harness::new_ui(empty_state);
harness.run();
assert!(harness.query_by_label(EMPTY_LIBRARY_LABEL).is_some());
```

- A component with state or intents: share it through `Rc<RefCell<…>>` or
  `Harness::new_ui_state`.
- After an interaction, call `harness.run()` again.
- Assert on the accessibility tree and on emitted intents, not on pixels.
- A mute widget (slider, canvas) gets an accessible label so it can be targeted.
- A themed component needs the fonts of the theme: wrap the closure body in
  `if is_themed(ui) { … }` (`tests/it/themed.rs`).
- A double click needs frames closer than egui's 0.3 s delay:
  `Harness::builder().with_step_dt(0.1)`.

## 7. Whole app — headless-verify
`Harness::builder().wgpu().build_eframe(|cc| ZivApp::new(cc))` + `harness.render()` behind the cargo
feature `headless-verify`. Ephemeral: nothing lands in `tests/`, evidence goes to
`verify-artifacts/<run-id>/` (gitignored). Native dialogs (file picker) are never
driven: state is injected through a constructor seam.

## 8. Running
```sh
cargo test
cargo test --lib
cargo test --test it ui_empty_state
cargo clippy --all-targets -- -D warnings
```
