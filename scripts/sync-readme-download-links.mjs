#!/usr/bin/env node
/**
 * Sync Download sections in README.md / README.zh-CN.md from package.json version
 * (or --version / NEKO_VERSION / git tag).
 *
 * Usage:
 *   node scripts/sync-readme-download-links.mjs
 *   node scripts/sync-readme-download-links.mjs --version 0.2.0
 *   node scripts/sync-readme-download-links.mjs --check   # exit 1 if out of date
 */
import { readFileSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = join(dirname(fileURLToPath(import.meta.url)), '..')
const pkg = JSON.parse(readFileSync(join(root, 'package.json'), 'utf8'))

const args = process.argv.slice(2)
const checkOnly = args.includes('--check')
const versionFlag = args.indexOf('--version')

function resolveVersion() {
  if (versionFlag >= 0 && args[versionFlag + 1]) {
    return String(args[versionFlag + 1])
  }
  if (process.env.NEKO_VERSION) {
    return String(process.env.NEKO_VERSION)
  }
  // On tag workflows GITHUB_REF_NAME is e.g. v0.1.0; ignore branch names like "main".
  const refName = process.env.GITHUB_REF_NAME
  if (refName && /^v?\d+\.\d+\.\d+/.test(refName)) {
    return refName
  }
  return pkg.version
}

const versionRaw = resolveVersion()
const version = String(versionRaw).replace(/^v/, '')
const tag = `v${version}`
const base = `https://github.com/rokiai/neko/releases/download/${tag}`
const latest = 'https://github.com/rokiai/neko/releases/latest'

const assets = {
  macArm64: `${base}/Neko-${version}-arm64.dmg`,
  macX64: `${base}/Neko-${version}-x64.dmg`,
  win: `${base}/Neko-${version}-setup.exe`,
  linuxAppImage: `${base}/Neko-${version}.AppImage`,
  linuxDeb: `${base}/neko_${version}_amd64.deb`
}

const START = '<!-- DOWNLOAD_LINKS:START -->'
const END = '<!-- DOWNLOAD_LINKS:END -->'

function blockEn() {
  return `${START}
## Download

**Current release: [${tag}](${latest})**

| Platform | Installer |
|----------|-----------|
| **macOS (Apple Silicon)** | [Neko-${version}-arm64.dmg](${assets.macArm64}) |
| **macOS (Intel)** | [Neko-${version}-x64.dmg](${assets.macX64}) |
| **Windows** | [Neko-${version}-setup.exe](${assets.win}) |
| **Linux (AppImage)** | [Neko-${version}.AppImage](${assets.linuxAppImage}) |
| **Linux (deb)** | [neko_${version}_amd64.deb](${assets.linuxDeb}) |

All releases: [${latest}](${latest})

> Links are generated from \`package.json\` version via \`pnpm sync:readme\`. Re-run after bumping the version (also runs on \`pnpm version\`).
${END}`
}

function blockZh() {
  return `${START}
## 下载安装包

**当前版本：[${tag}](${latest})**

| 平台 | 安装包 |
|------|--------|
| **macOS（Apple Silicon）** | [Neko-${version}-arm64.dmg](${assets.macArm64}) |
| **macOS（Intel）** | [Neko-${version}-x64.dmg](${assets.macX64}) |
| **Windows** | [Neko-${version}-setup.exe](${assets.win}) |
| **Linux（AppImage）** | [Neko-${version}.AppImage](${assets.linuxAppImage}) |
| **Linux（deb）** | [neko_${version}_amd64.deb](${assets.linuxDeb}) |

全部版本：[${latest}](${latest})

> 下载链接由 \`package.json\` 版本生成，改版本后执行 \`pnpm sync:readme\`（\`pnpm version\` 时也会自动跑）。
${END}`
}

function upsert(path, block) {
  const text = readFileSync(path, 'utf8')
  let next
  if (text.includes(START) && text.includes(END)) {
    next = text.replace(new RegExp(`${START}[\\s\\S]*?${END}`), block)
  } else {
    // Insert after the first language / nav paragraph block (after first --- or after intro links)
    const marker = '\nRepo: '
    const markerZh = '\n仓库：'
    if (text.includes(marker)) {
      const i = text.indexOf(marker)
      const lineEnd = text.indexOf('\n', i + 1)
      next = `${text.slice(0, lineEnd + 1)}\n${block}\n${text.slice(lineEnd + 1)}`
    } else if (text.includes(markerZh)) {
      const i = text.indexOf(markerZh)
      const lineEnd = text.indexOf('\n', i + 1)
      next = `${text.slice(0, lineEnd + 1)}\n${block}\n${text.slice(lineEnd + 1)}`
    } else {
      throw new Error(`Cannot find insertion point in ${path}`)
    }
  }
  return { path, text, next, changed: text !== next }
}

const targets = [
  { path: join(root, 'README.md'), block: blockEn() },
  { path: join(root, 'README.zh-CN.md'), block: blockZh() }
]

const requiredUrls = Object.values(assets)

if (checkOnly) {
  let failed = false
  for (const { path } of targets) {
    const text = readFileSync(path, 'utf8')
    const missing = requiredUrls.filter((url) => !text.includes(url))
    if (!text.includes(tag)) missing.push(`version marker ${tag}`)
    if (missing.length) {
      failed = true
      console.error(`${path} missing for ${tag}:`)
      for (const m of missing) console.error(`  - ${m}`)
    }
  }
  if (failed) {
    console.error(`README download links are out of date for ${tag}. Run: pnpm sync:readme`)
    process.exit(1)
  }
  console.log(`README download links OK for ${tag}`)
  process.exit(0)
}

for (const { path, block } of targets) {
  const result = upsert(path, block)
  writeFileSync(path, result.next)
  console.log(`${result.changed ? 'updated' : 'unchanged'} ${path}`)
}
console.log(`synced download links → ${tag}`)
