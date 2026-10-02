# ziv — Enhance

## 1. Goal
Remove the noise of a photo and strengthen its detail, once, with a model; then
dose the result with a slider.

## 2. Scope
**In**
- "Enhance" on the whole photo: computed on demand, with progress and Cancel.
- An Intensity slider, instant, from the original (0) to the full result (100).
- The result kept in a file beside the photo, reused at the next opening and
  by exports.

**Out**
- Enhancing through a mask; separate sliders for noise and sharpness.
- Enlarging the photo (super-resolution).
- Enhancing several photos in one go.
- Choosing the model.

## 3. Vocabulary
- **Enhancement** — the denoised and sharpened version of a photo, computed
  from the photo without its edit.
- **Intensity** — how much of the enhancement is mixed into the photo, 0 … 100.
- **Enhancement file** — the file beside the photo that keeps its enhancement.

## 4. Behavior

### Computing
1. The develop panel has a "Detail" section. On a photo without enhancement it
   holds one button, "Enhance".
2. Enhance starts the computation. A message shows the photo's name and the
   progress in percent, with a Cancel button. The window stays usable: other
   photos can be viewed and edited meanwhile.
3. Only one enhancement runs at a time; Enhance is disabled on every photo
   meanwhile.
4. The first use downloads the model, with the same message as for zone masks.
5. When the computation ends the enhancement file is written, and the photo's
   Intensity is set to 100. If that photo is the one being viewed it shows the
   result at once.
6. Cancel stops the computation: nothing is written and the photo is unchanged.
   A failure — download, model, file that cannot be written — is reported and
   leaves the photo unchanged.

### Intensity
7. On an enhanced photo the Detail section shows the Intensity slider, 0 … 100.
   At 0 the photo is exactly the original; at 100 it is the enhancement.
8. The enhancement is applied before every adjustment and every mask: they
   work on the cleaned photo.
9. Intensity is part of the edit: saved in the sidecar, undone and redone,
   reset to 0 by Reset. Before shows the photo without it.
10. Paste copies the Intensity; it has an effect only on a photo that has an
    enhancement.

### The enhancement file
11. The file is named after the photo's whole file name, with `.ziv.enhanced`.
12. A photo is enhanced exactly when its file exists and matches its size.
    Deleting the file loses nothing but the time to compute it again: the
    photo shows the Enhance button again, its Intensity kept for next time.
13. An export applies the enhancement of each photo that has one, at its
    Intensity; a photo without the file is exported without it.

## 5. Acceptance criteria
- [x] E1 — The enhancement of an image is the model's answer tile by tile:
      seams do not show, the border is whole, values above white survive
      (rule 2).
- [x] E2 — The enhancement file gives back the enhancement it was written
      from; a file of another photo size is refused
      (rules 11, 12).
- [x] E3 — Intensity 0 renders the original, 100 the enhancement, 50 their
      middle; adjustments and masks apply on top; export equals the display
      (rules 7, 8, 13).
- [x] E4 — Intensity is saved, undone, reset and pasted with the edit; an
      earlier sidecar still opens (rules 9, 10).
- [x] E5 — The Detail section shows Enhance or the slider according to the
      photo; Enhance is disabled while an enhancement runs (rules 1, 3, 7).
- [x] E6 — A run reports its progress, can be cancelled without leaving a
      file, and reports a failure (rules 2, 5, 6).
- [x] E7 — In the real app: enhance a photo, see the progress, dose the
      Intensity, reopen and find it enhanced, export it; the window keeps
      producing frames during the computation (rules 2, 5, 7, 12, 13).

## 6. Test plan
| Criterion | Level | Fixture |
|-----------|-------|---------|
| E1 | U | synthetic image, model stub |
| E2 | U + Eb | temp folder |
| E3 | U + G | synthetic image and a synthetic enhancement through the headless engine |
| E4 | U | none |
| E5 | Eu | none |
| E6 | U | model stub |
| E7 | HV | local model, a small local photo |

## 7. Open questions
None.
