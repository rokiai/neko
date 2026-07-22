import { useEffect, useRef, useState } from 'react'
import { BreakVideoSource } from '@shared/settings'
import { useI18n } from '../../../i18n/use-i18n'
import { getNekoApi, hasNekoBridge } from '../../../lib/neko'

export function LookVideoPreview({
  source,
  path
}: {
  source: BreakVideoSource
  path: string
}): React.JSX.Element {
  const { t } = useI18n()
  const videoRef = useRef<HTMLVideoElement | null>(null)

  // Sync fallbacks avoid setState-in-effect; Electron path loads asynchronously.
  const syncSrc: string | null | undefined = !hasNekoBridge()
    ? source === BreakVideoSource.Builtin
      ? './videos/calm.mp4'
      : null
    : source === BreakVideoSource.Custom && !path
      ? null
      : undefined

  const [asyncSrc, setAsyncSrc] = useState<string | null>(null)

  useEffect(() => {
    if (syncSrc !== undefined) return

    let cancelled = false
    void getNekoApi()
      .getBreakVideoSrc({
        breakVideoSource: source,
        breakVideoPath: path
      })
      .then((next) => {
        if (!cancelled) setAsyncSrc(next)
      })
      .catch(() => {
        if (!cancelled) setAsyncSrc(null)
      })

    return () => {
      cancelled = true
    }
  }, [source, path, syncSrc])

  const src = syncSrc !== undefined ? syncSrc : asyncSrc

  useEffect(() => {
    const el = videoRef.current
    if (!el || !src) return
    el.muted = true
    void el.play().catch(() => undefined)
  }, [src])

  if (!src) {
    return (
      <div className="look-preview-video is-empty" aria-hidden>
        <span>{t('settings.videoPathEmpty')}</span>
      </div>
    )
  }

  return (
    <div className="look-preview-video" aria-hidden>
      <video
        key={src}
        ref={videoRef}
        className="look-preview-video-el"
        src={src}
        muted
        loop
        playsInline
        autoPlay
        preload="metadata"
      />
    </div>
  )
}
