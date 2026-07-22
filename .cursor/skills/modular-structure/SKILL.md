---
name: modular-structure
description: >-
  Enforces modular, maintainable file structure for Neko. Use when adding or
  refactoring renderer pages, settings UI, main-process modules, CSS, or when
  the user mentions 拆分, 模块化, 臃肿, maintainability, or file size.
---

# Neko Modular Structure

Prefer many small focused modules over one God file. SettingsPage (~1100 lines) was the cautionary case — do not repeat it.

## Hard limits

| Kind                       | Soft limit | Hard limit (must split) |
| -------------------------- | ---------- | ----------------------- |
| React page / component     | ~200 lines | **400 lines**           |
| Main-process module        | ~200 lines | **350 lines**           |
| CSS co-located with a page | ~200 lines | **400 lines**           |

Line count alone is not enough: **one file = one primary job**. If a file owns multiple tabs, multiple window types, or UI + IPC + domain state together, split even under the soft limit.

## Renderer pages

**Do**

- Page file = thin shell (layout, tab switch, compose children).
- One folder per feature area: `pages/<feature>/`.
- Extract: `tabs/`, `components/`, `hooks/` (`use-*.ts`), co-located `*.css`.
- Page-local reusable widgets → `pages/<feature>/components/`.
- Cross-page widgets → `renderer/src/components/` (create when needed).

**Don't**

- Inline multi-section forms / tab bodies in the page file.
- Define several helper components at the bottom of a 500+ line page.
- Dump all styles into one mega `settings.css` / `page.css`.

### Settings target shape (canonical)

```
pages/settings/
  SettingsPage.tsx          # shell + tab routing only
  use-settings-draft.ts     # load / patch / dirty / save
  tabs/
    BreakTab.tsx
    HoursTab.tsx
    LookTab.tsx
    SystemTab.tsx
  components/
    UnitField.tsx
    RangeEditor.tsx
    …
  settings-shell.css
  …
```

`TodayPanel.tsx` + `use-runtime-status.ts` is the good existing pattern — follow it.

## Main process

**Do**

- Keep `index.ts` / `ipc.ts` as thin wiring.
- Split by domain into folders when a file grows: `lib/break/`, `lib/windows/`, `lib/tray/`.
- Separate: scheduling tick, break lifecycle, idle detection, window factories, tray icon/menu.

**Don't**

- Grow `scheduler.ts` / `windows.ts` / `tray.ts` into multi-concern blobs.
- Mix Dock/icon helpers with break-window creation in the same file forever.

## CSS

- Prefer styles next to the component or tab that owns them.
- Split by region (`shell`, `hours`, `look`, `form`) instead of one 700+ line sheet.
- Global tokens stay in theme / `main.css`; feature chrome does not.

## When adding features

1. Prefer a new file over appending to an already-large one.
2. If the change would push a file past the hard limit, **split first**, then feature.
3. New settings sections → new tab/component file, not more JSX in `SettingsPage.tsx`.
4. New break behaviors in main → new module under `lib/break/`, not another 80 lines in `scheduler.ts`.

## Refactor checklist

```
Modular Progress:
- [ ] Identify God file(s) and primary jobs inside
- [ ] Extract hooks / pure helpers first (lowest risk)
- [ ] Extract presentational components / tabs
- [ ] Split CSS with the same boundaries
- [ ] Keep public exports stable (barrel or re-export from thin facade if needed)
- [ ] Typecheck / smoke the affected surface
```

## Known debt (split when touching)

| Path                              | Issue                                        |
| --------------------------------- | -------------------------------------------- |
| `SettingsPage.tsx` + CSS          | **Done** — shell / tabs / hooks / region CSS |
| `main/lib/scheduler.ts`           | Tick + idle + lifecycle + stats + status     |
| `main/lib/windows.ts`             | All window types + Dock in one file          |
| `main/lib/tray.ts`                | Icon + menu + disable timer                  |
| `BreakProgress.tsx` + `break.css` | Timer + video + views intertwined            |

Do **not** invent huge abstractions; split along existing UI/domain seams.
