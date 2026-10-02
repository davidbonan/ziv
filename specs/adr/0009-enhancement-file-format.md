# ADR 0009 — Enhancement file format

- **Status**: accepted
- **Date**: 2026-10-02

## Context
ADR 0008 keeps an enhancement beside the photo as the difference between
enhanced and original in the model's encoding, 8 bits a channel, compressed as
a JPEG. What the model removes is noise, and JPEG is built to drop exactly
that. Measured on three 992² crops of an A7 IV RAW at low ISO, enhanced by the
real model (difference: 0.0055–0.0066 RMS in the model's encoding), sizes
scaled to 32.7 MP:

| Format | Size | Share of the difference lost |
|--------|------|------------------------------|
| JPEG q92 | 3.0–3.2 MB | 67–85 % |
| JPEG q100 | 13–15 MB | 51–63 % |
| JPEG q92, levels four times finer | 10 MB | 44–57 % |
| JPEG q92, square-root levels | 17–19 MB | 52–63 % |
| PNG | 33–34 MB | 0 % |

On a synthetic noise of 6 levels, JPEG q92 came back 4.2 levels away on
average: most of the noise would stay in the photo.

## Decision
| Concern | Choice |
|---------|--------|
| Picture | PNG, lossless, three channels of 8 bits: one level is 0.5 / 127 of the model's encoding, half a display level at most of rounding |
| Header | before the picture: signature, format version, photo size, ceiling |
| Size | about 33 MB for a clean 32.7 MP photo; a noisy one is **not measured**, its entropy says 50–60 MB |

## Alternatives rejected
- **JPEG at any quality** — see the table: it undoes the enhancement.
- **Finer levels** (16 bits, or a smaller reach) — two more bits a channel are
  25 MB more for an error already under half a display level.
- **Recomputing instead of keeping** — five minutes at each opening.

## Consequences
- An enhancement file weighs about as much as the RAW it sits beside. It stays
  a cache: deleting it loses only the time to compute it again.
- What is shown right after the computation and after reopening are the same
  pixels: both come from the same levels.
