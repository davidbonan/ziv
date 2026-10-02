---
name: spec
description: >-
  Frames a ziv feature or milestone before any code: writes specs/<feature>.md
  (goal, scope, behavior, acceptance criteria, test plan) and the matching task
  cards in specs/plan/STATE.md. Use when starting a milestone, when the user
  describes a new feature, or when STATE.md says "Spec: to write". Writes no
  code. Argument: a milestone ID (M1) or a feature description.
argument-hint: "[milestone ID or feature description]"
---

# spec

Turns an intent into `specs/<feature>.md` + task cards. **No code, no Cargo change.**

## Procedure

### 1. Read
`specs/overview.md` (features, locked and open decisions), `specs/architecture.md`,
`specs/testing.md`, `specs/plan/STATE.md`, and any spec the feature touches.

### 2. Resolve the unknowns with the user
Ask until these are answered — never fill a business gap by guessing:
- what the user does and sees, step by step;
- what is explicitly **out** of scope;
- the reference behavior when one exists (what Lightroom does) and whether parity is wanted;
- limits that change the design: image sizes, formats, expected latency.

An open decision from `overview.md` §4 that the feature needs ⇒ stop, run `/adr` first.

### 3. Write `specs/<feature>.md`
Kebab-case name after the concept (`import.md`, `local-masks.md`), not the milestone.

```markdown
# ziv — <Feature>

## 1. Goal
## 2. Scope
In / Out.
## 3. Vocabulary
Domain terms, one line each. These become the names in code.
## 4. Behavior
Numbered rules, observable, one behavior each.
## 5. Acceptance criteria
Checklist; each item verifiable by a test or a headless-verify run.
## 6. Test plan
Per criterion: level (U / Eb / G / Eu / HV) and the fixture it needs.
## 7. Open questions
```

Product intent only: no file paths, no struct design.

### 4. Cut the milestone into task cards
In `specs/plan/STATE.md`, under the milestone, format from `specs/plan/README.md`.
- One card = one verifiable step, reviewable in one diff.
- Order by dependency; first card is the thinnest end-to-end slice.
- Each card lists its test levels and the spec section it implements.
- Set the counter (`0/N`) and update "Next actions".

### 5. Update the index
`specs/overview.md` §2: link the spec. `CLAUDE.md` §Documentation: add the line.

### 6. Report
Spec path, number of cards, open questions left, decisions that need an ADR.

## Guardrails
- A criterion nobody can test is rewritten or removed.
- Do not specify a later milestone's needs "while here".
- Section 7 not empty ⇒ say so; the cards that depend on it are `⊘`.
