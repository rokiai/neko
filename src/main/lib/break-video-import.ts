import { app, dialog } from 'electron'
import { copyFileSync, existsSync, mkdirSync, readdirSync, unlinkSync } from 'fs'
import { basename, extname, join } from 'path'

const VIDEO_EXTS = new Set(['.mp4', '.webm', '.mov', '.m4v', '.mkv', '.avi'])

function videosDir(): string {
  const dir = join(app.getPath('userData'), 'videos')
  if (!existsSync(dir)) mkdirSync(dir, { recursive: true })
  return dir
}

function clearImportedVideos(exceptName?: string): void {
  const dir = videosDir()
  for (const name of readdirSync(dir)) {
    if (exceptName && name === exceptName) continue
    if (!name.startsWith('imported-')) continue
    try {
      unlinkSync(join(dir, name))
    } catch {
      // ignore stale cleanup failures
    }
  }
}

/** Pick a local video and copy it into the app data folder. */
export async function importBreakVideo(): Promise<string | null> {
  const result = await dialog.showOpenDialog({
    properties: ['openFile'],
    filters: [
      {
        name: 'Video',
        extensions: ['mp4', 'webm', 'mov', 'm4v', 'mkv', 'avi']
      }
    ]
  })

  if (result.canceled || !result.filePaths[0]) return null

  const source = result.filePaths[0]
  const ext = extname(source).toLowerCase()
  if (!VIDEO_EXTS.has(ext)) return null

  const destName = `imported-${Date.now()}${ext}`
  const dest = join(videosDir(), destName)
  copyFileSync(source, dest)
  clearImportedVideos(destName)
  return dest
}

export function removeImportedBreakVideo(filePath: string | undefined): void {
  if (!filePath) return
  const dir = videosDir()
  if (!filePath.startsWith(dir)) return
  if (!existsSync(filePath)) return
  try {
    unlinkSync(filePath)
  } catch {
    // ignore
  }
}

export function importedVideoLabel(filePath: string): string {
  return basename(filePath)
}
