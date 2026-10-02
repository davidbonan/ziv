# Progress state — ziv

> Source of truth for progress. Conventions: [`README.md`](README.md).
> `☐` to do · `◐` in progress · `☑` done+verified · `⊘` blocked · `⏭` deferred.

## Next actions
1. `/spec` M1 (import + viewport).
2. `/adr` working color space and display transform — blocks M1.

## Blockers
None.

---

## ☑ M0 — Foundations · 3/3
- ☑ **M0.1 — Repository.** Git, private GitHub repo, toolchain pinned.
- ☑ **M0.2 — App skeleton + test loop.** lib + bin, `ZivApp` on eframe/wgpu,
  one UI e2e, whole-app headless capture proven. *Tests*: Eu, HV.
- ☑ **M0.3 — Working method.** Specs, plan, ADR 0001, Claude Code skills.

## ☐ M1 — Import & viewport
Open a RAW or a standard image and see it, zoom and pan. Features F1.
*Spec*: to write. *Blocked by*: color space ADR.

## ☐ M2 — Global development
Non-destructive adjustments with live preview, edits persisted. Feature F2.
*Spec*: to write. *Blocked by*: edit persistence ADR.

## ☐ M3 — Export
Render the developed photo to a file at full resolution. Feature F6.
*Spec*: to write.

## ☐ M4 — Local adjustments
Brush, gradients and shapes as masks carrying their own adjustments. Feature F3.
*Spec*: to write.

## ☐ M5 — Zone masks
Masks generated on precise zones (hair, eyes, skin…). Feature F4.
*Spec*: to write. *Blocked by*: AI runtime and segmentation model ADRs.

## ☐ M6 — AI sharpening
Feature F5. *Spec*: to write. *Blocked by*: sharpening model ADR.
