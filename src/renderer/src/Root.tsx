import { ConfigProvider, App as AntApp } from 'antd'
import { useCallback, useEffect, useState, type ReactNode } from 'react'
import { resolveAppLocale, type AppLocale, type LocalePreference } from '@shared/i18n'
import { nekoTheme } from './theme/neko-theme'
import { I18nProvider } from './i18n/I18nProvider'
import { useI18n } from './i18n/use-i18n'
import { antdLocale } from './i18n/antd-locale'
import { BreakPage } from './pages/break/BreakPage'
import { SettingsPage } from './pages/settings/SettingsPage'
import { SoundsPage } from './pages/sounds/SoundsPage'
import { getNekoApi, hasNekoBridge } from './lib/neko'

function resolvePage(): 'settings' | 'break' | 'sounds' {
  const page = new URLSearchParams(window.location.search).get('page')
  if (page === 'break' || page === 'sounds') return page
  return 'settings'
}

function LocaleShell({ children }: { children: ReactNode }): React.JSX.Element {
  const { locale } = useI18n()
  return (
    <ConfigProvider
      theme={nekoTheme}
      locale={antdLocale(locale)}
      getPopupContainer={() => document.body}
    >
      <AntApp>{children}</AntApp>
    </ConfigProvider>
  )
}

export function Root(): React.JSX.Element {
  const page = resolvePage()
  const [locale, setLocale] = useState<AppLocale>(() => resolveAppLocale('system'))

  useEffect(() => {
    if (!hasNekoBridge()) return
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

  if (page === 'sounds') return <SoundsPage />

  return (
    <I18nProvider locale={locale}>
      <LocaleShell>
        {page === 'break' ? (
          <BreakPage />
        ) : (
          <SettingsPage onLocalePreferenceChange={handleLocalePreference} />
        )}
      </LocaleShell>
    </I18nProvider>
  )
}
