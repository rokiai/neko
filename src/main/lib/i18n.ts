import { APP_NAME } from '@shared/constants'
import { ensureI18n, setLocale, t, type MessageKey } from '@shared/i18n'
import { app } from 'electron'
import { getSettings } from './store'

export function syncLocaleFromSettings(): void {
  ensureI18n()
  const settings = getSettings()
  setLocale(settings.locale ?? 'system', app.getLocale())
}

export function tt(key: MessageKey, vars?: Record<string, string | number>): string {
  syncLocaleFromSettings()
  return t(key, vars)
}

export function appDisplayName(): string {
  return APP_NAME
}
