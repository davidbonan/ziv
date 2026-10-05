# ziv

A light RAW photo editor for macOS (Apple Silicon): import, cull, develop,
retouch with masks, export. Written in Rust on `egui` and `wgpu`.

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/davidbonan/ziv/main/install.sh | sh
```

Installs `ziv.app` into `/Applications` and opens it. The app is signed ad-hoc:
a zip downloaded with a browser instead is quarantined by macOS and must be
approved in System Settings › Privacy & Security.

## Updates

ziv checks for a newer version at launch and offers to install it. The running
version, at the foot of the series sidebar, opens the Updates dialog: check on
request, release notes.

## Build

```sh
cargo run                # the app, without updates
./scripts/bundle.sh      # dist/ziv.app
cargo test
```

## Release

A `v<version>` tag matching `Cargo.toml` makes CI
([`release.yml`](.github/workflows/release.yml)) test, bundle and publish
`ziv-macos.zip`.

## Third parties

- RAW decoding by [`rawler`](https://crates.io/crates/rawler), LGPL-2.1. The
  source of every release is this repository at its tag: ziv can be rebuilt
  with another version of `rawler`.
- Detection and enhancement models are not part of ziv: each is downloaded from
  its publisher the first time it is used. The two models that find skin, hair,
  clothes, eyes and lips (`Xenova/segformer_b2_clothes`, `Xenova/face-parsing`)
  are under non-commercial licences. ziv is given away, never sold.
- Design and decisions: [`specs/`](specs/overview.md).
