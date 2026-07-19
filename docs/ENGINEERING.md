# Neko Engineering Notes

## Process boundaries

| Process    | Responsibility                                      |
| ---------- | --------------------------------------------------- |
| `main`     | Scheduling, tray, windows, system APIs, persistence |
| `preload`  | Typed, minimal IPC surface via `contextBridge`      |
| `renderer` | UI only — no Node / Electron direct access          |
| `shared`   | Types + pure functions shared by main & renderer    |

## IPC convention

1. Channel names live in `src/shared/ipc.ts` as const enums / maps.
2. Payload types are co-located with channels.
3. Preload exposes a narrow `window.neko` API — never raw `ipcRenderer`.

## Coding standards

- Prefer small modules with single responsibility.
- Domain logic (timer tick, idle rules, working hours) belongs in `shared` or `main/lib` as pure functions with tests.
- No `moment`; prefer native `Temporal` / `Date` / lightweight dayjs if needed.
- Settings schema versioned; migrations live next to the store.

## Quality gates

Local and CI both run: `format:check` → `lint` → `typecheck` → `test` → `build`.
