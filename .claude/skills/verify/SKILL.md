---
name: verify
description: >-
  ziv quality gate: cargo fmt, clippy -D warnings on all targets, full test
  suite; reports PASS/FAIL with the real output. Use before checking a task ☑,
  before a commit, or when asked whether the build is green. Does not fix
  anything by itself beyond formatting.
---

# verify

## Procedure

Run in order, stop at the first failure:

```sh
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

Then check the working tree carries nothing ephemeral:

```sh
git status --short
```

No `tests/headless_verify_scratch_*.rs`, nothing from `verify-artifacts/`, no
golden image regenerated without having been looked at.

## When the diff is visible
A change to a UI component, the viewport or the engine output also needs
`/headless-verify`. Say so in the report if it was not run.

## Report
- **PASS** — test counts per binary (lib / `it`), clippy clean.
- **FAIL** — the failing command and its verbatim output. No diagnosis presented as fact
  before the cause is read in the code.

## Guardrails
- A failing golden test is a regression until proven otherwise: never regenerate
  the reference to turn the gate green.
- Never silence a lint with `#[allow]` to pass; fix it or ask.
