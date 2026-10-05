# ADR 0017 — Licence of ziv's source

- **Status**: accepted
- **Date**: 2026-10-05

## Context
The repository is public since ADR 0016 and had no licence: nobody but the
author could use, change or redistribute the source. `rawler` (LGPL-2.1) asks
that a user may rebuild ziv with another `rawler`, which the source must allow.

Licences in the dependency tree (`cargo tree`, 2026-10-05): MIT, Apache-2.0,
BSD-like, plus `rawler` LGPL-2.1, `option-ext` MPL-2.0, `webpki-roots`
CDLA-Permissive-2.0.

## Decision
ziv's own source is under the **MIT** licence: `LICENSE` at the repository
root, `license = "MIT"` in `Cargo.toml`. The author chose it.

## Alternatives rejected
- **MIT OR Apache-2.0**, as helm — the author chose MIT alone.
- **GPL or LGPL** — not evaluated.

## Consequences
- Anyone may rebuild ziv with another `rawler`: the LGPL condition of ADR 0016
  holds.
- The licence covers ziv's source only: each dependency keeps its own, and the
  models stay under their publishers' licences (ADR 0007, 0016).
- Whether a binary linking `rawler` statically needs more than the public
  source to meet the LGPL was not checked by a lawyer.
