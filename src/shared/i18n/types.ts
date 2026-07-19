export type AppLocale = 'en' | 'zh' | 'ja'
export type LocalePreference = 'system' | AppLocale

export const LOCALE_OPTIONS: Array<{ value: LocalePreference; labelKey: string }> = [
  { value: 'system', labelKey: 'locale.system' },
  { value: 'en', labelKey: 'locale.en' },
  { value: 'zh', labelKey: 'locale.zh' },
  { value: 'ja', labelKey: 'locale.ja' }
]

export function resolveAppLocale(
  preference: LocalePreference,
  systemLocale = detectSystemLocale()
): AppLocale {
  if (preference !== 'system') return preference
  const normalized = systemLocale.toLowerCase().replace('_', '-')
  if (normalized.startsWith('zh')) return 'zh'
  if (normalized.startsWith('ja')) return 'ja'
  return 'en'
}

export function detectSystemLocale(): string {
  if (typeof navigator !== 'undefined' && navigator.language) {
    return navigator.language
  }
  try {
    return Intl.DateTimeFormat().resolvedOptions().locale
  } catch {
    return 'en'
  }
}
