import { Howl } from 'howler'
import { useEffect } from 'react'
import { SoundType } from '@shared/settings'

function soundUrl(type: SoundType, kind: 'start' | 'end'): string {
  return `./sounds/${type.toLowerCase()}_${kind}.wav`
}

function play(type: SoundType, kind: 'start' | 'end', volume: number): void {
  if (type === SoundType.None) return
  const howl = new Howl({
    src: [soundUrl(type, kind)],
    volume: Math.min(1, Math.max(0, volume))
  })
  howl.play()
}

export function SoundsPage(): React.JSX.Element {
  useEffect(() => {
    const offStart = window.neko.onSoundStart((type, volume) => play(type, 'start', volume))
    const offEnd = window.neko.onSoundEnd((type, volume) => play(type, 'end', volume))
    return () => {
      offStart()
      offEnd()
    }
  }, [])

  return <div />
}
