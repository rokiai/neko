import { App } from 'antd'
import { useEffect, useMemo, useState } from 'react'
import type { LocalePreference } from '@shared/i18n'
import { DEFAULT_SETTINGS, type Settings } from '@shared/settings'
import { useI18n } from '../../i18n/use-i18n'
import { getNekoApi, hasNekoBridge } from '../../lib/neko'

export type SettingsPatch = <K extends keyof Settings>(key: K, value: Settings[K]) => void

export function useSettingsDraft(
  onLocalePreferenceChange: (preference: LocalePreference) => void
): {
  draft: Settings
  dirty: boolean
  loading: boolean
  bridgeError: string | null
  platform: NodeJS.Platform
  appVersion: string
  patch: SettingsPatch
  save: () => Promise<void>
  reset: () => void
} {
  const { message } = App.useApp()
  const { t } = useI18n()
  const [draft, setDraft] = useState<Settings>(DEFAULT_SETTINGS)
  const [saved, setSaved] = useState<Settings>(DEFAULT_SETTINGS)
  const [platform] = useState<NodeJS.Platform>(() =>
    hasNekoBridge() ? getNekoApi().platform : 'darwin'
  )
  const [bridgeError, setBridgeError] = useState<string | null>(() =>
    hasNekoBridge() ? null : t('settings.bridgeBrowser')
  )
  const [loading, setLoading] = useState(() => hasNekoBridge())
  const [appVersion, setAppVersion] = useState('')
  const dirty = useMemo(() => JSON.stringify(draft) !== JSON.stringify(saved), [draft, saved])

  useEffect(() => {
    if (!hasNekoBridge()) return

    let cancelled = false
    void getNekoApi()
      .getAppVersion()
      .then((version) => {
        if (!cancelled) setAppVersion(version)
      })
      .catch((error: unknown) => {
        console.warn('[neko] getAppVersion failed', error)
      })

    return () => {
      cancelled = true
    }
  }, [])

  useEffect(() => {
    if (!hasNekoBridge()) return

    let cancelled = false
    void getNekoApi()
      .getSettings()
      .then((settings) => {
        if (cancelled) return
        const merged = { ...DEFAULT_SETTINGS, ...settings }
        setDraft(merged)
        setSaved(merged)
        onLocalePreferenceChange(merged.locale ?? 'system')
        setLoading(false)
      })
      .catch((err: unknown) => {
        if (cancelled) return
        setBridgeError(err instanceof Error ? err.message : t('settings.loadFailed'))
        setLoading(false)
      })

    return () => {
      cancelled = true
    }
    // Mount-only: avoid wiping draft when locale/t identity changes.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const patch: SettingsPatch = (key, value) => {
    setDraft((prev) => {
      const next = { ...prev, [key]: value }
      if (key === 'locale') onLocalePreferenceChange(value as LocalePreference)
      return next
    })
  }

  const save = async (): Promise<void> => {
    await getNekoApi().setSettings(draft)
    setSaved(draft)
    message.success(t('settings.saved'))
  }

  const reset = (): void => {
    setDraft(saved)
    onLocalePreferenceChange(saved.locale ?? 'system')
  }

  return {
    draft,
    dirty,
    loading,
    bridgeError,
    platform,
    appVersion,
    patch,
    save,
    reset
  }
}
