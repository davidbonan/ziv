# ADR 0016 — Distribution and in-app update

- **Status**: accepted
- **Date**: 2026-10-05

## Context
ziv only runs through `cargo run`. The author wants it distributed and updated
from the app, as helm is (`../helm-studio/specs/update.md`), and chose a
**public** GitHub repository. Open before this: "Distribution: `.app` bundle,
signing, LGPL compliance for `rawler`" (`overview.md` §4). ADR 0007 forbids
distributing ziv "with the two SegFormer models".

Facts checked on this machine, 2026-10-05:
- `davidbonan/ziv` is private (`gh repo view`): the releases API answers
  without a token only once the repository is public.
- The release binary links system frameworks only (`otool -L
  target/release/ziv`): ONNX Runtime is linked statically, the bundle holds one
  executable.
- No model weight is in the repository or the binary: each file is fetched
  from its publisher at first use (`zone_models.rs`, `enhancement_model.rs`).
- A file ziv downloaded with `ureq` carries `com.apple.provenance` and no
  `com.apple.quarantine` (`xattr -l` on a model file, macOS 26.6.2).
- Licences in the tree other than MIT / Apache / BSD-like: `rawler` 0.8.0
  LGPL-2.1, `option-ext` MPL-2.0, `webpki-roots` CDLA-Permissive-2.0
  (`cargo tree`).
- `egui_commonmark` 0.25.0: MIT OR Apache-2.0, `egui ^0.36`, rust-version 1.95
  (toolchain here: 1.98.1).
- ziv has no licence file.

## Decision
| Concern | Choice |
|---------|--------|
| Channel | GitHub Releases of the public repository `davidbonan/ziv`; a release is a `v<semver>` tag, built and published by CI |
| Version | `Cargo.toml` is the single source; the tag must match it |
| Package | `ziv.app`, identifier `io.github.davidbonan.ziv`, zipped with `ditto` as `ziv-macos.zip` |
| Signing | ad-hoc (`codesign -s -`); the identity stays a parameter of the bundle script |
| First install | a `curl` script at the repository root, into `/Applications` |
| Update check | `releases/latest` of the GitHub API, without a token, through `ureq` |
| Update install | download through `ureq`; `ditto` to unpack, `codesign --verify --strict` to validate, rename to swap, `open -n` to relaunch |
| Integrity | TLS and the bundle signature; no checksum of our own |
| Release notes | one `release-notes.md` embedded in the binary, shown with `egui_commonmark` |
| What the app remembers | the last version whose notes were shown, in `eframe` persistence (ADR 0004) |
| `rawler` (LGPL-2.1) | the source of each release is public at its tag, so anyone can rebuild ziv with another `rawler`; the README says so |
| SegFormer models (ADR 0007) | kept; never in the bundle, fetched by the user from their publisher under their non-commercial licence; ziv is given away, never sold; the README says so |

## Alternatives rejected
- **Private repository, check through the `gh` CLI** — keeps the letter of ADR
  0007, but only updates machines where `gh` is signed in; the author chose
  public.
- **A public repository for the releases only** — the binary is public either
  way, with a second repository to keep.
- **`curl` as a subprocess for check and download**, as helm does — helm has no
  HTTP crate; ziv already ships `ureq` for the models.
- **Developer ID and notarization** — needs a paid Apple account; a download by
  `curl` or `ureq` is not quarantined (see the facts), so Gatekeeper does not
  ask. Not evaluated beyond that.
- **Sparkle** — a framework to embed and an appcast to publish; not evaluated.
- **Release notes read from the GitHub release** — needs the network and a
  cache at each launch.
- **Disabling person parts in the distributed build, or replacing the two
  models first** — the author chose to keep them.

## Consequences
- The repository must be public before the first release; until then the check
  fails as any network failure does.
- ADR 0007's consequence "before any distribution they are replaced" no longer
  holds: the two models stay, under the terms above. Whether those terms meet
  their licences is the author's reading, not a lawyer's.
- A zip downloaded with a browser is quarantined: macOS asks for a manual
  approval. Only the `curl` script and the in-app update avoid it.
- An ad-hoc signature proves the bundle is whole, not who built it.
- ziv's own source has no licence yet: to settle before the first release.
