# specs/plan — Implementation tracking

`specs/*.md` say **what we want**. This folder says **where we are** and **what
comes next**.

| File | Role |
|------|------|
| `README.md` | Conventions: statuses, per-task workflow, *Definition of Done*. Stable. |
| `STATE.md` | Living dashboard: milestones, task cards, next actions, blockers. Source of truth for progress. |

History and the "why" live in `git log`, not here.

## Statuses
| Symbol | Meaning |
|--------|---------|
| `☐` | To do |
| `◐` | In progress |
| `☑` | Done **and verified** |
| `⊘` | Blocked (reason in STATE §Blockers) |
| `⏭` | Deferred, with justification |

## Identifiers
Milestones `M<n>`, tasks `M<n>.<k>`. Never reused or renumbered once started.

## Task card
```
- ☐ **M1.2 — Title.** What exists when it is done. *Spec*: `specs/x.md` §n.
  *Depends*: M1.1. *Tests*: U, Eb, G, Eu.
```

## Per-task workflow
1. **Read** the task card and the spec it references.
2. **Implement the minimum** that meets the acceptance criteria, domain apart
   from rendering (`specs/architecture.md`).
3. **Test** at the levels the card lists (`specs/testing.md`).
4. **Verify** — `/verify`; for a visible UI change, also `/headless-verify`.
5. **Update `STATE.md`**: status, counter, next actions. Status only.

## Definition of Done
**Task `☑`**: acceptance criteria met · tests at the listed levels written and
green · `cargo clippy --all-targets -- -D warnings` clean · no dead code, no
leftover TODO · domain isolated from rendering · `STATE.md` up to date.

**Milestone `☑`**: all tasks `☑` (or `⏭` justified) · its demo scenario is shown
by `cargo test` or `/headless-verify`.

## Rules
- A milestone has no task cards until `/spec` wrote its spec.
- An open decision (`specs/overview.md` §4) blocking a task is settled by `/adr`
  first — never guessed.
- No evidence ⇒ `◐`, not `☑`.
