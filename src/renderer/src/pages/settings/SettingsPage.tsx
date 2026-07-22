import { Button } from 'antd'
import { useState } from 'react'
import type { LocalePreference } from '@shared/i18n'
import mascotSrc from '../../assets/brand/neko-mascot.png'
import { useI18n } from '../../i18n/use-i18n'
import { isBrowserPreview } from '../../lib/neko'
import { BreakTab } from './tabs/BreakTab'
import { HoursTab } from './tabs/HoursTab'
import { LookTab } from './tabs/LookTab'
import { SystemTab } from './tabs/SystemTab'
import { TAB_META, type SettingsTab } from './tab-meta'
import { TodayPanel } from './TodayPanel'
import { useRuntimeStatus } from './use-runtime-status'
import { useSettingsDraft } from './use-settings-draft'
import './settings.css'

export function SettingsPage({
  onLocalePreferenceChange
}: {
  onLocalePreferenceChange: (preference: LocalePreference) => void
}): React.JSX.Element {
  const { t } = useI18n()
  const { draft, dirty, loading, bridgeError, platform, appVersion, patch, save, reset } =
    useSettingsDraft(onLocalePreferenceChange)
  const runtime = useRuntimeStatus()
  const [tab, setTab] = useState<SettingsTab>('break')
  const [visitedTabs, setVisitedTabs] = useState<Set<string>>(() => new Set(['break']))

  const activeMeta = TAB_META.find((item) => item.key === tab) ?? TAB_META[0]

  const switchTab = (next: SettingsTab): void => {
    setTab(next)
    setVisitedTabs((prev) => {
      if (prev.has(next)) return prev
      const copy = new Set(prev)
      copy.add(next)
      return copy
    })
  }

  if (loading) {
    return <div className="settings-shell settings-loading">{t('common.loading')}</div>
  }

  if (bridgeError) {
    return (
      <div className="settings-shell settings-loading" style={{ textAlign: 'center', padding: 48 }}>
        <div style={{ fontFamily: 'var(--neko-serif)', fontSize: 28, marginBottom: 12 }}>Neko</div>
        <div style={{ maxWidth: 420, lineHeight: 1.6, color: 'var(--neko-muted)' }}>
          {isBrowserPreview() ? t('settings.bridgeBrowser') : bridgeError}
        </div>
      </div>
    )
  }

  return (
    <div className="settings-shell">
      <aside className="settings-sidebar">
        <div className="settings-brand-block">
          <h1>{t('app.name')}</h1>
          <p>{t('app.taglineShort')}</p>
        </div>

        <nav className="settings-nav" role="tablist" aria-label="Settings">
          {TAB_META.map((item) => (
            <button
              key={item.key}
              type="button"
              role="tab"
              aria-selected={tab === item.key}
              className={`settings-nav-item ${tab === item.key ? 'is-active' : ''}`}
              onClick={() => switchTab(item.key)}
            >
              <span className="settings-nav-icon">{item.icon}</span>
              {t(item.labelKey as 'settings.tab.breaks')}
            </button>
          ))}
        </nav>

        <div className="settings-sidebar-foot">
          <img className="settings-mascot" src={mascotSrc} alt="" />
          <div className="settings-status">
            <div className="settings-status-dot">
              {runtime.breaksEnabled && !runtime.idle
                ? t('settings.status.running')
                : t('common.off')}
            </div>
          </div>
        </div>
      </aside>

      <div className={`settings-main ${dirty ? 'is-editing' : ''}`}>
        <div className="settings-main-head">
          <h2>{t(activeMeta.labelKey as 'settings.tab.breaks')}</h2>
          <p>{t(activeMeta.descKey as 'settings.tab.breaksDesc')}</p>
        </div>

        <div className="settings-main-scroll">
          {visitedTabs.has('break') && (
            <div
              className="settings-tab-panel"
              role="tabpanel"
              hidden={tab !== 'break'}
              aria-hidden={tab !== 'break'}
            >
              <BreakTab draft={draft} patch={patch} />
            </div>
          )}

          {visitedTabs.has('hours') && (
            <div
              className="settings-tab-panel"
              role="tabpanel"
              hidden={tab !== 'hours'}
              aria-hidden={tab !== 'hours'}
            >
              <HoursTab draft={draft} patch={patch} />
            </div>
          )}

          {visitedTabs.has('look') && (
            <div
              className="settings-tab-panel"
              role="tabpanel"
              hidden={tab !== 'look'}
              aria-hidden={tab !== 'look'}
            >
              <LookTab draft={draft} patch={patch} />
            </div>
          )}

          {visitedTabs.has('system') && (
            <div
              className="settings-tab-panel"
              role="tabpanel"
              hidden={tab !== 'system'}
              aria-hidden={tab !== 'system'}
            >
              <SystemTab draft={draft} patch={patch} platform={platform} />
            </div>
          )}
        </div>

        {dirty && (
          <footer className="settings-footer">
            <span className="settings-version">
              {appVersion ? t('settings.version', { version: appVersion }) : null}
            </span>
            <div className="settings-footer-actions">
              <Button onClick={reset}>{t('common.discard')}</Button>
              <Button type="primary" onClick={() => void save()}>
                {t('common.save')}
              </Button>
            </div>
          </footer>
        )}
      </div>

      <TodayPanel status={runtime} />
    </div>
  )
}
