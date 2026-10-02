---
name: adr
description: >-
  Records a ziv technical decision as specs/adr/NNNN-title.md and updates the
  locked / open decision tables in specs/overview.md. Use when choosing a crate,
  a file format, a color space, an ML model or runtime, when an open decision
  blocks a task, or when a locked decision must change. Argument: the decision
  to make.
argument-hint: "[decision to make]"
---

# adr

One decision, one file, evidence attached.

## Procedure

### 1. Frame
State the decision as a question and what it blocks (`specs/plan/STATE.md`).
Read `specs/overview.md` §3–4 and the existing `specs/adr/`.

### 2. Gather evidence
For each candidate, measured facts only:
- crate: version and last release date (`cargo search`, `cargo info`), license, maintenance;
- model: license, size, input/output shape, where the weights come from;
- anything performance- or quality-related: a spike run on this machine, numbers kept.

A spike is thrown away after the numbers are recorded. No evidence for a claim ⇒ write "not evaluated".

### 3. Decide with the user
Present the candidates and a recommendation. The user picks.

### 4. Write `specs/adr/NNNN-kebab-title.md`
Next free number, never reused.

```markdown
# ADR NNNN — Title

- **Status**: accepted | superseded by NNNN
- **Date**: YYYY-MM-DD

## Context
## Decision
## Alternatives rejected
## Consequences
```

### 5. Propagate
- `specs/overview.md`: move the row from §4 (open) to §3 (locked), with the ADR number.
- Replacing a locked decision: mark the old ADR `superseded by NNNN`, do not edit its body.
- `specs/plan/STATE.md`: lift the "Blocked by" it resolves.

## Guardrails
- No code change beyond a discarded spike.
- An ADR records why; how-to belongs in the spec or the code.
