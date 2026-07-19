import type { MessageKey } from './i18n'

/** Older shipped defaults — treat as “use locale default” when migrating/resolving. */
export const LEGACY_BREAK_TITLE = 'Time for a break.'
export const LEGACY_BREAK_MESSAGE = 'Rest your eyes.\nStretch your legs.\nBreathe. Relax.'

export function isDefaultBreakTitle(value: string): boolean {
  const trimmed = value.trim()
  return !trimmed || trimmed === LEGACY_BREAK_TITLE
}

export function isDefaultBreakMessage(value: string): boolean {
  const trimmed = value.trim()
  return !trimmed || trimmed === LEGACY_BREAK_MESSAGE
}

export function resolveBreakTitle(value: string, t: (key: MessageKey) => string): string {
  return isDefaultBreakTitle(value) ? t('break.defaultTitle') : value
}

export function resolveBreakMessage(value: string, t: (key: MessageKey) => string): string {
  return isDefaultBreakMessage(value) ? t('break.defaultMessage') : value
}
