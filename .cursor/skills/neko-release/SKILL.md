---
name: neko-release
description: >-
  Ships a Neko GitHub Release via tag-triggered CI. Use when the user asks to
  发版, release, cut a version, bump version, publish, or push a v* tag for Neko.
---

# Neko Release

Tag push (`v*`) runs `.github/workflows/release.yml`: multi-platform build, then a **public** GitHub Release (`draft: false`). Do not force-push tags or amend published history unless the user explicitly asks.

## Preconditions

- Confirm the target version (e.g. `0.1.2`). If unclear, ask.
- Working tree should only contain intentional release changes.
- Prefer `main` in sync with `origin/main` before tagging.

## Checklist

```
Release Progress:
- [ ] 1. Commit feature / fix changes (if any)
- [ ] 2. Update CHANGELOG.md for the new version
- [ ] 3. Commit CHANGELOG (must be on the tag)
- [ ] 4. Bump version with pnpm version
- [ ] 5. Push branch + tag
- [ ] 6. Verify Release workflow started
- [ ] 7. Confirm public Release + artifacts after CI
```

## Step 1 — Commit product changes

If there are uncommitted changes, commit them first (normal git commit rules / HEREDOC). Do **not** bump `package.json` version in this commit.

## Step 2 — CHANGELOG (before the tag)

Update `CHANGELOG.md` using [Keep a Changelog](https://keepachangelog.com/) sections (`Added` / `Changed` / `Fixed` / `Docs` / …).

Template:

```markdown
## [X.Y.Z] — YYYY-MM-DD

### Changed

- …

### Fixed

- …

## [previous]
```

Also add the compare link at the bottom:

```markdown
[X.Y.Z]: https://github.com/rokiai/neko/releases/tag/vX.Y.Z
```

**Critical:** CHANGELOG must be committed **before** creating the version tag. Otherwise the tagged release tree will miss it (as happened with 0.1.1).

## Step 3 — Bump version

```bash
pnpm version X.Y.Z
```

This:

1. Runs the `version` lifecycle script → `pnpm sync:readme` (updates README download links) and stages README files
2. Sets `package.json` `"version"` to `X.Y.Z`
3. Creates commit + tag `vX.Y.Z`

If `pnpm version` fails, stop and fix; do not hand-edit version + tag inconsistently.

Verify:

```bash
node -p "require('./package.json').version"   # X.Y.Z
git rev-parse vX.Y.Z                          # tag exists
pnpm sync:readme:check                        # README links match version
```

## Step 4 — Push

```bash
git push origin HEAD
git push origin vX.Y.Z
```

Push the annotated/lightweight tag created by `pnpm version`. Never `--force` a release tag unless the user explicitly requests it.

## Step 5 — Verify CI

```bash
gh run list --workflow=release.yml --limit 3
```

Open the run URL. On success, a **public** Release appears under GitHub Releases with artifacts.

## Step 6 — Handoff

Tell the user:

1. Version / tag / commit SHAs
2. Actions run URL
3. Release URL (public once the publish job finishes)
4. Optionally edit Release notes to match the CHANGELOG `X.Y.Z` section if auto-notes are thin

## Do not

- Skip CHANGELOG or update it only after the tag
- Push `main` without the `v*` tag (no release job)
- Flip `draft` to `true` unless the user explicitly wants manual publish
- Re-tag / force-push to “fix” a missed CHANGELOG without explicit approval

## Quick reference

| Item         | Detail                           |
| ------------ | -------------------------------- |
| Trigger      | `push` of tag `v*`               |
| Version tool | `pnpm version X.Y.Z`             |
| README sync  | `version` script → `sync:readme` |
| Release type | Public (`draft: false`)          |
| Platforms    | macOS, Windows, Linux            |
