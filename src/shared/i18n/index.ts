import i18n from 'i18next'
import { en, type MessageKey, type Messages } from './en'
import { ja } from './ja'
import { zh } from './zh'
import {
  detectSystemLocale,
  resolveAppLocale,
  type AppLocale,
  type LocalePreference
} from './types'

const resources = {
  en: { translation: en as Messages },
  zh: { translation: zh },
  ja: { translation: ja }
}

let currentLocale: AppLocale = 'en'

function syncResourceBundles(): void {
  // deep=true + overwrite=true keeps HMR / newly added keys from sticking as raw keys
  i18n.addResourceBundle('en', 'translation', en, true, true)
  i18n.addResourceBundle('zh', 'translation', zh, true, true)
  i18n.addResourceBundle('ja', 'translation', ja, true, true)
}

export function ensureI18n(locale: AppLocale = currentLocale): typeof i18n {
  currentLocale = locale

  if (!i18n.isInitialized) {
    // Bundled dictionaries → sync init so the first paint never flashes raw keys.
    void i18n.init({
      resources,
      lng: locale,
      fallbackLng: 'en',
      // Flat keys like "settings.tab.breaks" (also avoids tray.disable vs tray.disable.30m clash)
      keySeparator: false,
      nsSeparator: false,
      initAsync: false,
      interpolation: {
        escapeValue: false
      },
      returnNull: false,
      returnEmptyString: false
    })
  } else {
    syncResourceBundles()
    if (i18n.language !== locale) {
      void i18n.changeLanguage(locale)
    }
  }

  return i18n
}

export function getLocale(): AppLocale {
  return currentLocale
}

export function setLocale(preference: LocalePreference, systemLocale?: string): AppLocale {
  const locale = resolveAppLocale(preference, systemLocale ?? detectSystemLocale())
  setResolvedLocale(locale)
  return locale
}

export function setResolvedLocale(locale: AppLocale): void {
  currentLocale = locale
  ensureI18n(locale)
  if (i18n.language !== locale) {
    void i18n.changeLanguage(locale)
  }
}

export function t(key: MessageKey, vars?: Record<string, string | number>): string {
  ensureI18n(currentLocale)
  return i18n.t(key, { lng: currentLocale, ...(vars ?? {}) })
}

export { i18n }
export * from './types'
export type { MessageKey, Messages }
