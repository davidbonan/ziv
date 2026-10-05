# ADR 0007 — Zone mask models

- **Status**: accepted; the consequence on distribution is superseded by 0016
- **Date**: 2026-10-02

## Context
M5 masks the subject, the sky, and the parts of chosen persons (skin, hair,
eyes, lips, clothes). The author stated that ziv is **for personal use only**
and chose **non-commercial models** for person parts after seeing the spike.

Spike (ADR 0006 for the timings), judged by eye on four photos:
- **BiRefNet lite** (`onnx-community/BiRefNet_lite-ONNX`, MIT): the subject of
  the A7 IV sample and of a studio portrait cleanly cut.
- **skyseg** (`JianyuanWang/skyseg`, U²-Net, MIT): a landscape sky exact. Raw
  output peaks at 1.0 on that landscape, 0.07 on a sky darkened by an edit,
  0.02 on a photo without sky, and 1.0 on a grey studio backdrop (false sky).
- **RF-DETR small** (`onnx-community/rfdetr_small-ONNX`, Apache-2.0): the four
  persons of a group photo found, one box each.
- **MediaPipe selfie multiclass + face landmarks** (Apache-2.0): hair and face
  found, clothes and body skin missed on full-length persons, landmarks valid
  on two faces of four.
- **SegFormer B2 clothes** (`Xenova/segformer_b2_clothes`, NVIDIA SegFormer
  licence: non-commercial) on each person box: hair, face, arms, legs and
  clothes found on the four persons.
- **SegFormer face parsing** (`Xenova/face-parsing`, trained on CelebAMask-HQ:
  non-commercial) on each face: eyes, brows, lips found on a portrait; lips
  only, or nothing, on faces of 130–250 px seen in profile or behind sunglasses.

## Decision
| Zone | Model | File | Licence |
|------|-------|------|---------|
| Subject, background | BiRefNet lite, input 1024² | `model.onnx`, 224 MB | MIT |
| Sky | skyseg, input 320² | `skyseg.onnx`, 176 MB | MIT |
| Persons | RF-DETR small, input 512², class person, score > 0.5 | `model.onnx`, 115 MB | Apache-2.0 |
| Skin, hair, clothes | SegFormer B2 clothes on each person box, input 512² | `model.onnx`, 110 MB | non-commercial |
| Eyes, lips | SegFormer face parsing on each face, input 512² | `model.onnx`, 340 MB | non-commercial |

- Sky coverage is the model's output read on a fixed scale (none below 0.2,
  full above 0.6: an overcast sky scores lower than a blue one), never
  stretched to its own maximum: a photo without sky gives an empty mask.
- A matte coarser than the coverage image (sky: 320²; person parts: 128²) has
  its edges moved onto the photo's outlines by a guided filter on the photo's
  colours. With the photo's lightness alone as guide, sunlit rock under a sky
  of the same lightness left a halo; with its colours the ridge is clean.
- A person's parts are computed inside that person's box and limited to the
  subject matte where it helps the edge; the face box is the extent of the
  class Face.
- A zone mask is stored as its coverage image (long edge ≤ 2048), rendered as
  a layer like a brush mask (ADR 0005), and written in the sidecar as a PNG in
  base64 (sidecar version 3): reopening a photo never needs a model.

## Alternatives rejected
- **MediaPipe models only** — permissive, but body parts are missed outside
  selfie framing (spike).
- **Sapiens** (CC BY-NC 4.0) — not evaluated; larger.
- **Quantized or fp16 files** — BiRefNet fp16 is slower on CPU (6.0–7.4 s);
  the others were not evaluated.
- **Coverage in a file beside the sidecar** — keeps the JSON small, but two
  files must stay in step and a deleted mask leaves an orphan that undo may
  still need.
- **Recomputing zone masks at each opening** — needs the models and seconds
  per mask, and a model update would silently change an edit.

## Consequences
- **ziv cannot be distributed or sold with the two SegFormer models.** Before
  any distribution they are replaced, through a new ADR.
- Five downloads, 965 MB in all, each fetched only when its zone is first used.
- Known limits, shown to the user as what they are: a uniform backdrop can be
  taken for sky; eyes and lips are not found on small, turned or covered faces.
