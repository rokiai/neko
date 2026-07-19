import { useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import type { AppLocale, MessageKey } from '@shared/i18n'
import { getLocale } from '@shared/i18n'

interface UseI18nResult {
  locale: AppLocale
  t: (key: MessageKey, vars?: Record<string, string | number>) => string
}

/** Thin wrapper so existing call sites keep working on top of react-i18next. */
export function useI18n(): UseI18nResult {
  const { t: translate, i18n } = useTranslation()
  const locale = (i18n.resolvedLanguage?.split('-')[0] as AppLocale) || getLocale()

  return useMemo(
    () => ({
      locale,
      t: (key, vars) => String(translate(key, { ...(vars ?? {}) }))
    }),
    [locale, translate]
  )
}
