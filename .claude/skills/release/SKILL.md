---
name: release
description: >-
  Publishes a new ziv version end to end: preflight, version bump in
  Cargo.toml, section of release-notes.md, quality gate, "Release v<version>"
  commit and tag, push, CI watch, check of the published ziv-macos.zip. Use
  when the user asks to release, publish or ship a version. Argument: patch |
  minor | major | x.y.z; without one, proposes the patch bump and asks.
argument-hint: "[patch|minor|major|x.y.z]"
---

# release

Pushing a `v<version>` tag makes [`release.yml`](../../../.github/workflows/release.yml)
test, bundle and publish `ziv-macos.zip` (`specs/update.md` rules 2, 25).
Repository: `davidbonan/ziv`. The tag is derived from `Cargo.toml`, never the reverse.

## Procedure

### 0. Preflight — stop on any failure
```sh
git rev-parse --abbrev-ref HEAD     # main
git status --short                  # empty
git fetch origin && git rev-list --left-right --count origin/main...main
gh repo view davidbonan/ziv --json visibility   # PUBLIC, else the app cannot check
```
- Dirty tree, or behind `origin/main` ⇒ stop and ask. Ahead ⇒ list the unpushed
  commits: they ship with the release.
- The target tag exists (`git tag -l`, `git ls-remote --tags origin`) ⇒ stop.

### 1. Pick the version
Current: `grep -m1 '^version' Cargo.toml`. Argument `x.y.z` must be greater, with
no suffix. No argument ⇒ show `git log v<last>..HEAD --oneline` and ask
(AskUserQuestion) before touching anything.

### 2. Bump
Edit `version` in `Cargo.toml`, then `cargo check` (refreshes `Cargo.lock`).

### 3. Release notes
From `git log v<last>..HEAD --no-merges --pretty=format:'%s'` (all commits when
there is no tag yet):
1. Write a `## <version>` section under `# Release notes`, newest first: one
   bullet per change a user sees, in product words. Drop refactors, tests, CI.
   Nothing user-facing ⇒ one honest line.
2. Keep the 10 latest sections; delete the older ones.
3. Show the section to the user and apply their edits.

### 4. Gate
Run `/verify`. Red ⇒ report and stop; revert nothing.

### 5. Commit, tag, push
```sh
git add Cargo.toml Cargo.lock release-notes.md
git commit -m "Release v<version>"
git tag v<version>
git push origin main v<version>
```
Report any warning the remote prints.

### 6. Watch CI
`gh run list --workflow=Release --branch v<version>`, then
`gh run watch <id> --exit-status` in the background.

### 7. Check the published release, in a temporary folder
```sh
tmp="$(mktemp -d)"
curl -fsSL -o "$tmp/ziv-macos.zip" \
  "https://github.com/davidbonan/ziv/releases/download/v<version>/ziv-macos.zip"
ditto -x -k "$tmp/ziv-macos.zip" "$tmp"
xattr -p com.apple.quarantine "$tmp/ziv.app"   # expected: No such xattr
codesign --verify --strict "$tmp/ziv.app"
/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' \
  "$tmp/ziv.app/Contents/Info.plist"            # expected: <version>
rm -rf "$tmp"
```

### 8. Report
Version, commit, tag, CI run URL, release URL, the three checks of step 7.

## Failures
- **CI red after the push**: `gh run view <id> --log-failed`, report. Fix on
  `main`, then release the next patch version.
- **Asset missing or broken**: report; re-running the workflow is the user's call.
- **Push rejected**: report verbatim.

## Guardrails
- Never `--force`, never move or delete a tag, never install into `/Applications`.
- The release commit holds `Cargo.toml`, `Cargo.lock`, `release-notes.md` only.
- No `STATE.md` or spec edit.
