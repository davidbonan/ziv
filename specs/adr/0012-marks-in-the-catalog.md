# ADR 0012 — Marks and series filters in the catalog

- **Status**: accepted
- **Date**: 2026-10-02

## Context
Culling (`specs/culling.md`) gives a photo a rating from 0 to 5 and a rejected
mark, and gives a series a filter. All three must survive a quit (rules 28,
35). The author chose marks proper to ziv: no XMP is read or written.

Two places already keep things between launches: the edit sidecar beside each
photo (ADR 0003) and the catalog in the user data folder (ADR 0011).

A sidecar means an edit: a photo with a sidecar is an edited photo, counted in
the sidebar and marked in the filmstrip, and a photo back to no edit loses its
sidecar (ADR 0003, `catalog.md` rule 12). A rated photo is not an edited photo
(`culling.md` rule 29).

The catalog is already written at every change of selection (`catalog.md`
rule 13), to a temporary file renamed over it.

## Decision
| Concern | Choice |
|---------|--------|
| Where marks are kept | in the catalog, one entry per photo path that carries a mark |
| Whose they are | the photo's: two series holding the same path read the same entry |
| A photo at rating 0 and not rejected | has no entry |
| Where a series filter is kept | in its series, in the catalog |
| Format | the catalog's `version` goes to 2; a version 1 catalog is read as one without marks nor filters |
| Writing | with the catalog, when the mark is made |
| Marks of a photo no series holds any more | kept: importing the photo again finds them |

The author left the choice to the recommendation made with the spec.

## Alternatives rejected
- **In the edit sidecar** — travels with the folder, but a rating would create
  a sidecar: "edited" would have to stop meaning "has a sidecar", and the
  sidecar could no longer be removed at all-default.
- **A second sidecar per photo** — travels with the folder and leaves the edit
  alone, at the price of one more file beside every rated photo and of one file
  read per photo to show a grid.
- **XMP sidecar (`xmp:Rating`)** — readable by Bridge and Lightroom; the author
  chose marks proper to ziv. JPEG, PNG and TIFF carry XMP inside the file,
  which ziv never writes.
- **A file of its own in the data folder** — same ties to this machine as the
  catalog, with a second file to version, write and report.

Catalog size and write latency with thousands of marks were not evaluated.

## Consequences
- Marks are tied to this machine and to absolute paths, as the catalog is: a
  folder moved outside ziv loses its marks unless its series is located again,
  which has to move the marks with the paths.
- Losing the catalog loses the marks; the edits stay (ADR 0003).
- A catalog written by this ziv is refused by an older one (version 2), by the
  rule of ADR 0011.
