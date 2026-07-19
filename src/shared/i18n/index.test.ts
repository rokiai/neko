import { describe, expect, it } from 'vitest'
import { ensureI18n, resolveAppLocale, setLocale, t } from './index'

describe('i18n (i18next)', () => {
  it('resolves system locales', () => {
    expect(resolveAppLocale('system', 'zh-CN')).toBe('zh')
    expect(resolveAppLocale('system', 'ja-JP')).toBe('ja')
    expect(resolveAppLocale('system', 'en-US')).toBe('en')
    expect(resolveAppLocale('zh', 'en-US')).toBe('zh')
  })

  it('translates with i18next interpolation', () => {
    ensureI18n('en')
    setLocale('zh')
    expect(t('tray.nextIn', { time: '5m' })).toContain('5m')
    setLocale('ja')
    expect(t('break.start')).toBe('開始')
  })

  it('resolves newly added flat keys after ensure', () => {
    ensureI18n('zh')
    expect(t('app.taglineShort')).toBe('温和地提醒你休息')
    expect(t('settings.status.running')).toBe('运行中')
    expect(t('common.unit.minutes')).toBe('分钟')
    expect(t('settings.version', { version: '0.1.0' })).toBe('版本 0.1.0')
    expect(t('settings.tab.breaksDesc')).toContain('提醒')
  })
})
