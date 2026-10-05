# ziv — Distribution and update

## 1. Goal
Use ziv without `cargo run`: install it as an application, and let it bring
itself up to date — it says when a newer version exists, installs it on request,
relaunches, and shows what changed.

Reference: helm (`../helm-studio/specs/update.md`). Parity is wanted, except
where this spec says otherwise (§2 Out, and ADR 0016 for the network calls).

## 2. Scope
**In**
- `ziv.app` built from the repository, and a release published by CI for each
  version tag (ADR 0016).
- A script that installs the latest release into `/Applications`.
- A check for a newer version at launch and on request.
- Installing the newer version from the app, then relaunching.
- Release notes carried by the app: shown once after an update, readable at
  any time.
- A `/release` skill that publishes a version.

**Out**
- A Preferences page: ziv has none; the Updates dialog stands in for helm's
  Preferences › Updates.
- Beta channel, delta updates, install without asking, a check repeated while
  the app runs, going back to an older version.
- Developer ID signing and notarization, Sparkle.
- Images in the release notes; notes in another language than English.
- Windows and Linux packages.

## 3. Vocabulary
- **Version** — three numbers, `major.minor.patch`. The version of a build is
  the one in `Cargo.toml`.
- **Release** — a version published on GitHub: a `v<version>` tag and its
  **release asset**, the zipped application.
- **App bundle** — the `ziv.app` folder macOS runs. A build launched outside
  one (`cargo run`) is **unbundled**.
- **Update check** — asking GitHub for the latest release and comparing it to
  the running version.
- **Available update** — a release newer than the running version.
- **Update install** — download, unpack, validate, swap, relaunch.
- **Swap** — the running app bundle replaced by the downloaded one.
- **Release notes** — what changed, one section per version, newest first.
- **Seen version** — the last version whose notes the app showed or took as
  its baseline.
- **Updates dialog** — the window showing the version, the check, the install
  and the release notes.

## 4. Behavior

### Package and release
1. A script builds `ziv.app` from the release build: the executable, the
   icon, the identifier `io.github.davidbonan.ziv`, the version of `Cargo.toml`, an
   ad-hoc signature that validates.
2. Pushing a `v<version>` tag makes CI run the tests, build the app bundle, zip
   it as `ziv-macos.zip` and publish the release with that asset. A tag that
   differs from the version of `Cargo.toml` fails the run and publishes
   nothing.
3. The install script downloads the asset of the latest release, replaces
   `/Applications/ziv.app` and opens it.

### Update check
4. A bundled app checks once at launch, in the background. A failed check at
   launch says nothing.
5. The user can check from the Updates dialog at any time; a failed check
   there says why.
6. The latest release is newer than the running version ⇒ an update is
   available. Equal or older ⇒ ziv is up to date.
7. A release whose tag is not a version, or without the release asset, is a
   failed check.
8. An unbundled app never checks by itself and cannot install: the Updates
   dialog says "Running outside an app bundle: updates are disabled".

### Update install
9. Installing downloads the release asset, unpacks it, and validates the
   signature of the app bundle it holds. A failure at any of these steps
   leaves the running app untouched, removes what was downloaded and says why.
10. A validated app bundle takes the place of the running one. If it cannot,
    the running one is put back and the failure says why; an app that cannot
    be replaced where it is asks to be moved to `/Applications`.
11. After the swap, what the app has to save is saved (edits, settings), the
    new version is launched and the running one quits.
12. One operation at a time: while a check or an install runs, asking for
    another does nothing.
13. An install cannot be asked for while an export or an enhancement runs.

### What the user sees
14. An available update shows a strip at the bottom of the window:
    "Version 0.2.0 is available", **Install and relaunch**, **Later**. Later
    hides it until the next launch.
15. During an install the strip says "Downloading 0.2.0…" then "Installing
    0.2.0…", without buttons. A failed install shows its reason as a notice.
16. The foot of the series sidebar shows the running version; clicking it
    opens the Updates dialog.
17. The Updates dialog shows: the running version; **Check for updates** and
    its result — checking, "ziv is up to date", "Version 0.2.0 is available"
    with **Install and relaunch**, or the reason of a failure; the release
    notes; **Close**.
18. While a check or an install runs, the buttons that would start one are
    disabled.

### Release notes
19. The release notes are part of the app: they need no network.
20. They hold at most the 10 latest versions, newest first, each under its
    version.
21. A bundled app launched with a version newer than the seen version shows
    the release notes once, in a "What's new" window, and takes the running
    version as seen.
22. With no seen version (first install) the running version is taken as seen
    and nothing is shown.
23. An unbundled app shows no "What's new" window and changes no seen version.
24. An update installed by the install script is noticed the same way at the
    next launch.

### Publishing
25. `/release` bumps the version of `Cargo.toml`, writes the section of that
    version in the release notes from the commits since the last tag, runs the
    quality gate, commits, tags, pushes, watches CI and checks the published
    asset: no quarantine, signature valid, version matching.

## 5. Acceptance criteria
- **P1** The bundle script makes a `ziv.app` whose signature validates and
  whose version is the one of `Cargo.toml`. (rule 1)
- **P2** The release workflow refuses a tag that differs from the version and
  publishes `ziv-macos.zip` otherwise. (rule 2)
- **P3** The install script fetches the asset of the latest release into
  `/Applications`. (rule 3)
- **P4** Versions parse with or without `v`, order by major, minor, patch;
  anything else is refused. (rules 6, 7)
- **P5** A release answer gives its version and the address of its asset; a
  malformed tag, a missing asset and an answer that is not a release are three
  distinct failures. (rule 7)
- **P6** A check against a served release reports an available update when it
  is newer, up to date when equal or older, and a failure when the server
  cannot be reached. (rules 5, 6)
- **P7** The app bundle of an executable is found from its path; an executable
  outside one has none. (rule 8)
- **P8** An install from a served zip replaces a signed app bundle by the new
  one; the old one is no longer in place. (rules 9, 10)
- **P9** A zip whose app bundle has a broken signature is refused: the app
  bundle in place is unchanged and nothing is left of the download.
  (rule 9)
- **P10** A swap whose second step fails puts the old app bundle back.
  (rule 10)
- **P11** The update goes Idle → Checking → UpToDate or Available →
  Downloading → Installing; a failure at launch falls back to Idle, any other
  shows its reason; a request during an operation is ignored. (rules 4, 5, 12)
- **P12** The strip offers Install and relaunch and Later on an available
  update, and shows progress without buttons during an install.
  (rules 14, 15)
- **P13** The Updates dialog shows each state of rule 17, disables its buttons
  while an operation runs, and says updates are disabled when unbundled.
  (rules 8, 17, 18)
- **P14** The sidebar foot shows the version and asks for the Updates dialog.
  (rule 16)
- **P15** The bundled release notes hold 1 to 10 version sections, each a
  version, newest first. (rule 20)
- **P16** What's new is shown for a newer running version, skipped for a seen
  or older one, taken as baseline with no seen version, and never when
  unbundled. (rules 21–23)
- **P17** The release notes appear in the Updates dialog and in the What's new
  window, which closes on Close. (rules 17, 21)
- **P18** In the real app, unbundled: the sidebar foot shows the version, the
  dialog opens with the notes and says updates are disabled; no strip, no
  What's new window. (rules 8, 16, 23)
- **P19** A release published by `/release` installs through the install
  script, and an older installed version updates itself to it and shows What's
  new. (rules 2, 3, 11, 21, 25)

## 6. Test plan
| Criterion | Level | Fixture |
|-----------|-------|---------|
| P1 | run of the script | `codesign --verify --strict`, `PlistBuddy` on the result |
| P2, P3 | read in review; proven by P19 | — |
| P4, P5, P7 | U | release answers written in the test |
| P6 | Eb | a local HTTP server answering one request |
| P8, P9, P10 | Eb | app bundles made in a temporary folder, signed ad-hoc, zipped with `ditto`, served locally |
| P11 | U | results handed to the update run |
| P12, P13, P14, P17 | Eu | — |
| P15, P16 | U | — |
| P18 | HV | — |
| P19 | by hand, once the repository is public | two published releases |

## 7. Open questions
None: the repository is public and the source is under the MIT licence (ADR 0017).
