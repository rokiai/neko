---
name: neko-commit
description: >-
  Commits Neko changes with project conventions, then optionally pushes to both
  GitHub (origin) and Gitee (gitee). Use when the user asks to 提交, commit,
  提交代码, or save work in the neko repo.
---

# Neko Commit

Commit intentional changes only. Never bump release version or create tags here — that belongs to `neko-release`.

## Remotes

| Remote   | URL                                     | Role                    |
| -------- | --------------------------------------- | ----------------------- |
| `origin` | `git@github.com-rokiai:rokiai/neko.git` | Primary (CI / Releases) |
| `gitee`  | `git@gitee.com:rokiai/neko.git`         | Mirror                  |

Keep `main` tracking **`origin/main`** (not gitee). After any `git push -u gitee …`, restore:

```bash
git branch --set-upstream-to=origin/main main
```

## Checklist

```
Commit Progress:
- [ ] 1. status / diff / log (parallel)
- [ ] 2. Draft message (why, 1–2 sentences)
- [ ] 3. Stage relevant files only
- [ ] 4. Commit via HEREDOC
- [ ] 5. Verify status
- [ ] 6. Push both remotes only if user asked to push / 提交并推送 / 同步
```

## Step 1 — Inspect (parallel)

```bash
git status
git diff
git log -5 --oneline
```

Also `git diff --staged` if anything is already staged.

## Step 2 — Message

- Prefer repo style: imperative, concise, focus on **why**
- Recent examples: `Fix …`, `Split …`, `Document …`
- Do **not** commit secrets (`.env`, credentials, private keys)
- Do **not** use `--no-verify` / skip hooks unless user explicitly asks
- Do **not** amend unless user asked and amend safety rules are met

## Step 3 — Commit

```bash
git add <paths>
git commit -m "$(cat <<'EOF'
Message here.

EOF
)"
git status
```

If pre-commit hooks modify files and the commit fails or leaves dirty files, create a **new** commit (do not amend unless amend rules allow).

## Step 4 — Push (only when asked)

When the user says push / 推送 / 同步两边 / 提交并推送:

```bash
git push origin HEAD
git push gitee HEAD
git branch --set-upstream-to=origin/main main
git status -sb
```

If there are tags to mirror (uncommon for normal commits):

```bash
git push origin --tags
git push gitee --tags
```

## Do not

- Push only one remote when the user asked to sync both
- Force-push `main` or rewrite published history without explicit approval
- Mix release version bumps into a normal feature commit
- Leave `main` tracking `gitee/main` after push

## Handoff

Tell the user: commit SHA, short subject, whether pushed to `origin` and/or `gitee`.
