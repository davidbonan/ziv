# ADR 0013 — Photos moved to the Trash, photos removed from a series

- **Status**: accepted
- **Date**: 2026-10-02

## Context
Culling (`specs/culling.md` rules 37–43) removes photos from a series and
moves rejected photos to the macOS Trash. Two locked decisions stand in the
way:
- ADR 0001: "the source file is never written";
- ADR 0011: the photo list of a series is frozen at import, and importing a
  folder again adds the photos its series does not have yet.

The author wants both actions, and left the open points to the recommendation
made with the spec.

Checked on crates.io, 2026-10-02: `trash` 5.2.9, MIT, rust-version 1.85.
On macOS it offers two ways: asking the Finder through `osascript` (sound,
"Put Back" in the Trash, needs the user to let the app control the Finder) or
`NSFileManager` (no sound, no "Put Back", no permission). Run on this machine
through `NSFileManager`: a temporary file left its folder for the Trash.

## Decision
| Concern | Choice |
|---------|--------|
| Content of a source file | never written, as ADR 0001 says: no adjustment, no metadata |
| Moving a source file to the Trash | allowed, on a confirmed request of the user only |
| What goes with it | the files ziv keeps beside the photo: sidecar, enhancement file |
| Permanent deletion | never: ziv does not delete a source file, the Trash is emptied by the user |
| How | the `trash` crate through `NSFileManager`, behind a port declared in the domain |
| Photo list of a series | frozen at import, except for photos the user removes or trashes |
| A photo removed from a folder series | remembered by the series: importing the folder again does not bring it back |
| A photo moved to the Trash | not remembered: a file put back in the folder is new to the series |
| Bringing removed photos back | Remove the series, then import the folder again (`catalog.md` rule 29) |

ADR 0001 and ADR 0011 are amended by this ADR on those points only; both stay
accepted.

## Alternatives rejected
- **Removal from the series only, no Trash** — keeps ADR 0001 to the letter,
  but leaves the rejected RAW files on disk: the author would go back to Bridge
  or the Finder to free the space.
- **Permanent deletion** — not recoverable; one wrong selection loses photos.
- **A removed photo comes back at the next import of its folder** — importing
  again is how new photos of a shoot join the series; it would undo the culling
  each time.
- **`NSFileManager.trashItem` through `objc2`** — no new crate, but unsafe
  bindings to write and keep; not evaluated.

## Consequences
- A trashed file has no "Put Back" in the Finder: it is dragged out of the Trash
  by hand. The price of asking no permission.
- ziv can now make a source file leave its folder: the confirmation and the
  Trash are the only safety, there is no undo in ziv.
- A trashed photo held by another series is a missing photo there.
- The catalog keeps, per folder series, the paths removed from it; they are
  forgotten with the series.
- `specs/overview.md` §3: the Edits row says what "never written" covers.
