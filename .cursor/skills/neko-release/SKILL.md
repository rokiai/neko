---
name: neko-release
description: >-
  Ships a Neko release: CHANGELOG, pnpm version tag, push to GitHub (CI) and
  Gitee mirror. Use when the user asks to 发版, release, cut a version, bump
  version, publish, or push a v* tag for Neko.
---

# Neko Release

Tag push of `v*` to **GitHub** (`origin`) runs `.github/workflows/release.yml`: multi-platform build + **public** GitHub Release. Mirror the same commits/tags to **Gitee** (`gitee`). Do not force-push tags or amend published history unless the user explicitly asks.

## Remotes

| Remote   | URL                                     | Role                |
| -------- | --------------------------------------- | ------------------- |
| `origin` | `git@github.com-rokiai:rokiai/neko.git` | CI + GitHub Release |
| `gitee`  | `git@gitee.com:rokiai/neko.git`         | Mirror              |

`main` should track `origin/main`. Restore after accidental `-u gitee`:

```bash
git branch --set-upstream-to=origin/main main
```

## Preconditions

- Confirm target version (e.g. `0.1.4`). If unclear, ask.
- Working tree should only contain intentional release changes.
- Prefer `main` in sync with `origin/main` before tagging.

## Checklist

```
Release Progress:
- [ ] 1. Commit feature / fix changes (if any) — use neko-commit
- [ ] 2. Update CHANGELOG.md for the new version
- [ ] 3. Commit CHANGELOG (must be on the tag)
- [ ] 4. Bump version with pnpm version
- [ ] 5. Push branch + tag to origin AND gitee
- [ ] 6. Verify Release workflow on GitHub
- [ ] 7. Confirm public Release + artifacts after CI
- [ ] 8. Ensure main tracks origin/main
```

## Step 1 — Commit product changes

If there are uncommitted changes, commit them first (`neko-commit` / HEREDOC). Do **not** bump `package.json` version in that commit.

## Step 2 — CHANGELOG (before the tag)

Update `CHANGELOG.md` (Keep a Changelog: `Added` / `Changed` / `Fixed` / `Docs` / …).

```markdown
## [X.Y.Z] — YYYY-MM-DD

### Changed

- …

### Fixed

- …

## [previous]
```

Footer link:

```markdown
[X.Y.Z]: https://github.com/rokiai/neko/releases/tag/vX.Y.Z
```

**Critical:** CHANGELOG must be committed **before** `pnpm version` / tag. Otherwise the tagged tree misses it.

## Step 3 — Bump version

```bash
pnpm version X.Y.Z
```

This runs `version` → `pnpm sync:readme`, bumps `package.json`, creates commit + tag `vX.Y.Z`.

Verify:

```bash
node -p "require('./package.json').version"
git rev-parse vX.Y.Z
pnpm sync:readme:check
```

If `pnpm version` fails, stop and fix; do not hand-edit version + tag inconsistently.

## Step 4 — Push (GitHub + Gitee)

```bash
# GitHub — required to trigger CI / Release
git push origin HEAD
git push origin vX.Y.Z

# Gitee — mirror
git push gitee HEAD
git push gitee vX.Y.Z

git branch --set-upstream-to=origin/main main
```

Never `--force` a release tag unless the user explicitly requests it.

## Step 5 — Verify CI (GitHub only)

```bash
gh run list --workflow=release.yml --limit 3
```

On success: public Release at `https://github.com/rokiai/neko/releases/tag/vX.Y.Z` with artifacts.

Gitee has no equivalent Actions release job unless configured separately — mirroring code/tags is enough.

## Step 6 — Handoff

Tell the user:

1. Version / tag / commit SHAs
2. Pushed to `origin` + `gitee`
3. GitHub Actions run URL (if available)
4. Release URL (after publish job)
5. Optionally align GitHub Release notes with CHANGELOG `X.Y.Z`

## Do not

- Skip CHANGELOG or update it only after the tag
- Push `main` to GitHub without the `v*` tag (no release job)
- Push release only to Gitee and forget `origin` (CI never runs)
- Flip `draft: true` unless the user wants manual publish
- Re-tag / force-push to “fix” a missed CHANGELOG without approval

## Quick reference

| Item         | Detail                                    |
| ------------ | ----------------------------------------- |
| Trigger      | `push` of tag `v*` to **GitHub** `origin` |
| Mirror       | Same commit + tag to `gitee`              |
| Version tool | `pnpm version X.Y.Z`                      |
| README sync  | `version` script → `sync:readme`          |
| Release type | Public (`draft: false`)                   |
| Platforms    | macOS, Windows, Linux                     |
