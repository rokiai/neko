import { App } from 'antd'
import { useCallback, useEffect, useMemo, useState } from 'react'
import type { LocalePreference } from '@shared/i18n'
import { DEFAULT_SETTINGS, type Settings } from '@shared/settings'
import { useI18n } from '../../i18n/use-i18n'
import { getNekoApi, invokeErrorText, isTauriRuntime } from '../../lib/neko'

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
  /** Persists draft + partial. Resolves false when saving failed. */
  commit: (partial: Partial<Settings>) => Promise<boolean>
  reset: () => void
} {
  const { message } = App.useApp()
  const { t } = useI18n()
  const [draft, setDraft] = useState<Settings>(DEFAULT_SETTINGS)
  const [saved, setSaved] = useState<Settings>(DEFAULT_SETTINGS)
  const [platform] = useState<NodeJS.Platform>(() => getNekoApi().platform)
  const [bridgeError, setBridgeError] = useState<string | null>(() =>
    isTauriRuntime() ? null : t('settings.bridgeBrowser')
  )
  const [loading, setLoading] = useState(true)
  const [appVersion, setAppVersion] = useState('')
  // Settings nest per-weekday range arrays, so a shallow compare would miss
  // edits. Serialising `saved` separately means a draft edit only re-stringifies
  // the draft — relevant while dragging a slider, which patches continuously.
  const savedJson = useMemo(() => JSON.stringify(saved), [saved])
  const dirty = useMemo(() => JSON.stringify(draft) !== savedJson, [draft, savedJson])

  useEffect(() => {
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

  // Stable across renders: these are handed to every tab, so a fresh identity
  // each render would defeat any memoisation downstream.
  const patch: SettingsPatch = useCallback(
    (key, value) => {
      setDraft((prev) => {
        const next = { ...prev, [key]: value }
        if (key === 'locale') onLocalePreferenceChange(value as LocalePreference)
        return next
      })
    },
    [onLocalePreferenceChange]
  )

  const save = async (): Promise<void> => {
    try {
      await getNekoApi().setSettings(draft)
    } catch (error: unknown) {
      message.error(invokeErrorText(error, t('settings.saveFailed')))
      return
    }
    setSaved(draft)
    message.success(t('settings.saved'))
  }

  const commit = async (partial: Partial<Settings>): Promise<boolean> => {
    const next = { ...draft, ...partial }
    try {
      await getNekoApi().setSettings(next)
    } catch (error: unknown) {
      message.error(invokeErrorText(error, t('settings.saveFailed')))
      return false
    }
    setDraft(next)
    setSaved(next)
    if (partial.locale != null) onLocalePreferenceChange(partial.locale)
    message.success(t('settings.saved'))
    return true
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
    commit,
    reset
  }
}
