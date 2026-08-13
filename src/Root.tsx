import { Suspense, lazy, useCallback, useEffect, useState } from 'react'
import { resolveAppLocale, type AppLocale, type LocalePreference } from '@shared/i18n'
import { I18nProvider } from './i18n/I18nProvider'
import { BreakPage } from './pages/break/BreakPage'
import { getNekoApi } from './lib/neko'

// Break windows are created on demand and must appear immediately, so
// `BreakPage` stays in the entry chunk. Everything antd — providers, locales,
// theme, the whole form surface — is reachable only through `SettingsRoot`, so
// a Break window never downloads or parses any of it.
const SettingsRoot = lazy(async () => ({
  default: (await import('./pages/settings/SettingsRoot')).SettingsRoot
}))

function resolvePage(): 'settings' | 'break' {
  const page = new URLSearchParams(window.location.search).get('page')
  if (page === 'break') return page
  return 'settings'
}

export function Root(): React.JSX.Element {
  const page = resolvePage()
  const [locale, setLocale] = useState<AppLocale>(() => resolveAppLocale('system'))

  useEffect(() => {
    void getNekoApi()
      .getSettings()
      .then((settings) => {
        setLocale(resolveAppLocale(settings.locale ?? 'system'))
      })
      .catch(() => undefined)
  }, [])

  const handleLocalePreference = useCallback((preference: LocalePreference): void => {
    setLocale(resolveAppLocale(preference))
  }, [])

  return (
    <I18nProvider locale={locale}>
      {page === 'break' ? (
        <BreakPage />
      ) : (
        <Suspense fallback={null}>
          <SettingsRoot onLocalePreferenceChange={handleLocalePreference} />
        </Suspense>
      )}
    </I18nProvider>
  )
}
