import { app } from 'electron'
import { existsSync } from 'fs'
import { join } from 'path'

/** Resolve a file under project `resources/` for both electron-vite and packaged builds. */
export function resolveResource(...parts: string[]): string {
  const candidates = [
    join(__dirname, '../../resources', ...parts),
    join(__dirname, '../../../resources', ...parts),
    join(app.getAppPath(), 'resources', ...parts),
    join(process.resourcesPath, 'resources', ...parts),
    join(process.resourcesPath, 'app.asar.unpacked', 'resources', ...parts)
  ]

  for (const candidate of candidates) {
    if (existsSync(candidate)) return candidate
  }

  return candidates[0]
}
