---
name: implement-state
description: >-
  Implements the next task from specs/plan/STATE.md (or continues the in-progress
  one) end to end through the Definition of Done, then updates the progress
  state. Use when the user says "next task", "continue", "avance", or names a
  task ID. ONE task per invocation. Optional argument: a task ID (M1.2).
argument-hint: "[task ID — optional]"
---

# implement-state

Advances ziv by **exactly one task** of [`specs/plan/STATE.md`](../../../specs/plan/STATE.md).
Workflow and *Definition of Done* are defined in
[`specs/plan/README.md`](../../../specs/plan/README.md); on divergence, that file wins.

## Procedure

### 1. Read the state
`specs/plan/STATE.md`, then `specs/plan/README.md`.

### 2. Choose the task
1. Argument ⇒ that task ID.
2. Else the in-progress task `◐`.
3. Else the first `☐` of "Next actions", else the first `☐` in order.

Validate before coding — otherwise stop and ask:
- the milestone has a spec and task cards (else: `/spec`);
- every dependency is `☑`;
- no open decision blocks it (else: `/adr`).

### 3. Frame and mark `◐`
Read the card and the spec section it references. Restate in one sentence: goal,
files touched, acceptance criteria, test levels. Set the task to `◐` in `STATE.md`.

### 4. Implement the minimum
- Boundary and folder rule: `specs/architecture.md`.
- Touching the engine, a shader, a mask or color math: load the `image-pipeline` skill first.
- Locked decisions (`specs/overview.md` §3) preserved.

### 5. Write the tests
At the levels the card lists (`specs/testing.md`): `U`, `Eb`, `G`, `Eu`. They
exercise the card's acceptance criteria.

### 6. Verify
Run `/verify`. For a visible UI or rendering change, also `/headless-verify`.
Red ⇒ no `☑`.

### 7. Update the tracking
`STATE.md`: task `☑`, counter, "Next actions". All tasks of the milestone
`☑`/`⏭` ⇒ run its demo scenario, then mark the milestone `☑`.

### 8. Report
Task and final status · files touched · evidence (test count, clippy, HV folder) · next action.

## When to stop and ask
- dependency not `☑`, or decision still open;
- acceptance criteria ambiguous;
- the task needs to deviate from a locked decision or from the spec.

Leave the task `◐`, or `⊘` with the reason in `STATE.md` §Blockers.

## Guardrails
- One task per invocation.
- Never `☑` without green evidence.
- Do not commit unless asked.
