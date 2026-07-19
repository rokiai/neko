import { protocol, net } from 'electron'
import { pathToFileURL } from 'url'

const SCHEME = 'neko-video'

/** Must run before app is ready. */
export function registerVideoScheme(): void {
  protocol.registerSchemesAsPrivileged([
    {
      scheme: SCHEME,
      privileges: {
        standard: true,
        secure: true,
        supportFetchAPI: true,
        stream: true,
        bypassCSP: true,
        corsEnabled: true
      }
    }
  ])
}

/** Wire the custom protocol after app ready. */
export function registerVideoProtocolHandler(): void {
  protocol.handle(SCHEME, (request) => {
    const fileUrl = request.url.replace(/^neko-video:/i, 'file:')
    return net.fetch(fileUrl)
  })
}

export function toNekoVideoUrl(absolutePath: string): string {
  return pathToFileURL(absolutePath).href.replace(/^file:/i, `${SCHEME}:`)
}
