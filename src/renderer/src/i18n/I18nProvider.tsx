import { useEffect, type ReactNode } from 'react'
import { I18nextProvider, initReactI18next } from 'react-i18next'
import { ensureI18n, i18n, type AppLocale } from '@shared/i18n'

let reactI18nWired = false

function wireReactI18n(): void {
  if (reactI18nWired) return
  // Must run before first init in the renderer process.
  if (!i18n.isInitialized) {
    void i18n.use(initReactI18next)
  }
  reactI18nWired = true
}

export function I18nProvider({
  locale,
  children
}: {
  locale: AppLocale
  children: ReactNode
}): React.JSX.Element {
  wireReactI18n()
  ensureI18n(locale)

  useEffect(() => {
    ensureI18n(locale)
    if (i18n.language !== locale) {
      void i18n.changeLanguage(locale)
    }
  }, [locale])

  return <I18nextProvider i18n={i18n}>{children}</I18nextProvider>
}
