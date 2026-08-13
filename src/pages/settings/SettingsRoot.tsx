import { ConfigProvider, App as AntApp } from 'antd'
import type { LocalePreference } from '@shared/i18n'
import { antdLocale } from '../../i18n/antd-locale'
import { useI18n } from '../../i18n/use-i18n'
import { nekoTheme } from '../../theme/neko-theme'
import { SettingsPage } from './SettingsPage'

// Module-level so the provider's context value does not change identity on
// every render.
const popupContainer = (): HTMLElement => document.body

/**
 * antd entry point for the Settings window.
 *
 * Break windows render no antd at all, so the provider stack lives here rather
 * than in `Root` — that keeps antd, its locales and the theme out of the entry
 * chunk that every Break window has to load.
 */
export function SettingsRoot({
  onLocalePreferenceChange
}: {
  onLocalePreferenceChange: (preference: LocalePreference) => void
}): React.JSX.Element {
  const { locale } = useI18n()

  return (
    <ConfigProvider
      theme={nekoTheme}
      locale={antdLocale(locale)}
      getPopupContainer={popupContainer}
    >
      <AntApp>
        <SettingsPage onLocalePreferenceChange={onLocalePreferenceChange} />
      </AntApp>
    </ConfigProvider>
  )
}
