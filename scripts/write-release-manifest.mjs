import { createHash } from 'node:crypto'
import { readdir, readFile, stat, writeFile } from 'node:fs/promises'
import { join, relative, resolve } from 'node:path'

const [bundleRootArg = 'src-tauri/target/release/bundle', outputArg] = process.argv.slice(2)
const bundleRoot = resolve(bundleRootArg)
const output = resolve(outputArg ?? join(bundleRoot, 'release-manifest.json'))

async function filesUnder(directory) {
  const entries = await readdir(directory, { withFileTypes: true })
  const files = []
  for (const entry of entries) {
    const path = join(directory, entry.name)
    if (entry.isDirectory()) files.push(...(await filesUnder(path)))
    else if (entry.isFile() && path !== output) files.push(path)
  }
  return files
}

const files = (await filesUnder(bundleRoot)).sort()
const artifacts = []
for (const path of files) {
  if (!/\.(?:dmg|exe|AppImage|deb|rpm|msi|zip|tar\.gz|json)$/i.test(path)) continue
  const bytes = await readFile(path)
  const info = await stat(path)
  artifacts.push({
    path: relative(bundleRoot, path),
    bytes: info.size,
    sha256: createHash('sha256').update(bytes).digest('hex')
  })
}

await writeFile(
  output,
  `${JSON.stringify({ generatedAt: new Date().toISOString(), artifacts }, null, 2)}\n`,
  'utf8'
)
