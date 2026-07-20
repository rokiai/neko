import {
  App,
  Button,
  ColorPicker,
  Divider,
  Form,
  Input,
  InputNumber,
  Select,
  Slider,
  Space,
  Switch,
  Typography
} from 'antd'
import { useEffect, useMemo, useRef, useState, type ReactNode } from 'react'
import { resolveBreakMessage, resolveBreakTitle } from '@shared/break-copy'
import { LOCALE_OPTIONS, type LocalePreference } from '@shared/i18n'
import {
  DEFAULT_SETTINGS,
  BreakPopupStyle,
  BreakVideoSource,
  NotificationType,
  SOUND_TYPES,
  SoundType,
  TrayTextMode,
  WORKING_HOURS_DAY_KEYS,
  type Settings,
  type WorkingHours,
  type WorkingHoursDayKey
} from '@shared/settings'
import mascotSrc from '../../assets/brand/neko-mascot.png'
import { useI18n } from '../../i18n/use-i18n'
import { getNekoApi, hasNekoBridge, isBrowserPreview } from '../../lib/neko'
import { useRuntimeStatus } from './use-runtime-status'
import { TodayPanel } from './TodayPanel'
import './settings.css'

const { Paragraph, Text } = Typography

const APP_VERSION = '0.1.0'

type SettingsTab = 'break' | 'hours' | 'look' | 'system'

const DAY_LABEL_KEY: Record<WorkingHoursDayKey, string> = {
  workingHoursMonday: 'day.monday',
  workingHoursTuesday: 'day.tuesday',
  workingHoursWednesday: 'day.wednesday',
  workingHoursThursday: 'day.thursday',
  workingHoursFriday: 'day.friday',
  workingHoursSaturday: 'day.saturday',
  workingHoursSunday: 'day.sunday'
}

const SOUND_LABEL_KEY: Record<SoundType, string> = {
  [SoundType.None]: 'sound.none',
  [SoundType.Gong]: 'sound.gong',
  [SoundType.Blip]: 'sound.blip',
  [SoundType.Bloop]: 'sound.bloop',
  [SoundType.Ping]: 'sound.ping',
  [SoundType.Scifi]: 'sound.scifi'
}

function LookVideoPreview({
  source,
  path
}: {
  source: BreakVideoSource
  path: string
}): React.JSX.Element {
  const { t } = useI18n()
  const videoRef = useRef<HTMLVideoElement | null>(null)

  // Sync fallbacks avoid setState-in-effect; Electron path loads asynchronously.
  const syncSrc: string | null | undefined = !hasNekoBridge()
    ? source === BreakVideoSource.Builtin
      ? './videos/calm.mp4'
      : null
    : source === BreakVideoSource.Custom && !path
      ? null
      : undefined

  const [asyncSrc, setAsyncSrc] = useState<string | null>(null)

  useEffect(() => {
    if (syncSrc !== undefined) return

    let cancelled = false
    void getNekoApi()
      .getBreakVideoSrc({
        breakVideoSource: source,
        breakVideoPath: path
      })
      .then((next) => {
        if (!cancelled) setAsyncSrc(next)
      })
      .catch(() => {
        if (!cancelled) setAsyncSrc(null)
      })

    return () => {
      cancelled = true
    }
  }, [source, path, syncSrc])

  const src = syncSrc !== undefined ? syncSrc : asyncSrc

  useEffect(() => {
    const el = videoRef.current
    if (!el || !src) return
    el.muted = true
    void el.play().catch(() => undefined)
  }, [src])

  if (!src) {
    return (
      <div className="look-preview-video is-empty" aria-hidden>
        <span>{t('settings.videoPathEmpty')}</span>
      </div>
    )
  }

  return (
    <div className="look-preview-video" aria-hidden>
      <video
        key={src}
        ref={videoRef}
        className="look-preview-video-el"
        src={src}
        muted
        loop
        playsInline
        autoPlay
        preload="metadata"
      />
    </div>
  )
}

const TAB_META: Array<{
  key: SettingsTab
  labelKey: string
  descKey: string
  icon: ReactNode
}> = [
  {
    key: 'break',
    labelKey: 'settings.tab.breaks',
    descKey: 'settings.tab.breaksDesc',
    icon: (
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" aria-hidden>
        <path
          d="M12 3c-2.2 2.4-3.5 4.6-3.5 7a3.5 3.5 0 1 0 7 0c0-2.4-1.3-4.6-3.5-7Z"
          stroke="currentColor"
          strokeWidth="1.8"
          strokeLinejoin="round"
        />
        <path
          d="M8 17c1.2 1.5 2.5 2.5 4 2.5s2.8-1 4-2.5"
          stroke="currentColor"
          strokeWidth="1.8"
          strokeLinecap="round"
        />
      </svg>
    )
  },
  {
    key: 'hours',
    labelKey: 'settings.tab.hours',
    descKey: 'settings.tab.hoursDesc',
    icon: (
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" aria-hidden>
        <circle cx="12" cy="12" r="8" stroke="currentColor" strokeWidth="1.8" />
        <path
          d="M12 8v4.5l3 1.5"
          stroke="currentColor"
          strokeWidth="1.8"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </svg>
    )
  },
  {
    key: 'look',
    labelKey: 'settings.tab.look',
    descKey: 'settings.tab.lookDesc',
    icon: (
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" aria-hidden>
        <rect
          x="3.5"
          y="5"
          width="17"
          height="12"
          rx="2.5"
          stroke="currentColor"
          strokeWidth="1.8"
        />
        <path d="M8 20h8" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" />
        <path d="M12 17v3" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" />
      </svg>
    )
  },
  {
    key: 'system',
    labelKey: 'settings.tab.system',
    descKey: 'settings.tab.systemDesc',
    icon: (
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" aria-hidden>
        <circle cx="12" cy="12" r="3" stroke="currentColor" strokeWidth="1.8" />
        <path
          d="M12 4.5v2M12 17.5v2M4.5 12h2M17.5 12h2M6.4 6.4l1.4 1.4M16.2 16.2l1.4 1.4M17.6 6.4l-1.4 1.4M7.8 16.2l-1.4 1.4"
          stroke="currentColor"
          strokeWidth="1.8"
          strokeLinecap="round"
        />
      </svg>
    )
  }
]

const HOUR_OPTIONS = Array.from({ length: 24 }, (_, hour) => ({
  value: hour,
  label: String(hour).padStart(2, '0')
}))

const MINUTE_OPTIONS = Array.from({ length: 60 }, (_, minute) => ({
  value: minute,
  label: String(minute).padStart(2, '0')
}))

function clampDayMinutes(total: number): number {
  return Math.min(23 * 60 + 59, Math.max(0, Math.floor(total)))
}

function MinutesOfDaySelect({
  value,
  onChange,
  disabled = false
}: {
  value: number
  onChange: (next: number) => void
  disabled?: boolean
}): React.JSX.Element {
  const safe = clampDayMinutes(value)
  const hours = Math.floor(safe / 60)
  const minutes = safe % 60

  return (
    <Space.Compact className="hours-time-select">
      <Select
        value={hours}
        options={HOUR_OPTIONS}
        disabled={disabled}
        popupMatchSelectWidth={72}
        listHeight={256}
        onChange={(hour) => onChange(clampDayMinutes(hour * 60 + minutes))}
        aria-label="Hour"
      />
      <Select
        value={minutes}
        options={MINUTE_OPTIONS}
        disabled={disabled}
        popupMatchSelectWidth={72}
        listHeight={256}
        onChange={(minute) => onChange(clampDayMinutes(hours * 60 + minute))}
        aria-label="Minute"
      />
    </Space.Compact>
  )
}

function RangeEditor({
  value,
  onChange,
  disabled = false
}: {
  value: WorkingHours
  onChange: (next: WorkingHours) => void
  disabled?: boolean
}): React.JSX.Element {
  const { t } = useI18n()
  const ranges = value.ranges?.length ? value.ranges : [{ fromMinutes: 0, toMinutes: 23 * 60 + 59 }]

  return (
    <div className={`hours-day ${disabled ? 'is-disabled' : ''}`}>
      <div className="hours-day-head">
        <Switch
          checked={value.enabled}
          disabled={disabled}
          onChange={(enabled) => onChange({ ...value, enabled, ranges })}
          size="small"
        />
        <Text type="secondary">{value.enabled ? t('common.active') : t('common.off')}</Text>
      </div>
      {ranges.map((range, index) => (
        <div key={index} className="hours-range">
          <div className="hours-range-times">
            <MinutesOfDaySelect
              value={range.fromMinutes}
              disabled={disabled || !value.enabled}
              onChange={(fromMinutes) => {
                const nextRanges = [...ranges]
                nextRanges[index] = { ...range, fromMinutes }
                onChange({ ...value, ranges: nextRanges })
              }}
            />
            <span className="hours-to">{t('common.to')}</span>
            <MinutesOfDaySelect
              value={range.toMinutes}
              disabled={disabled || !value.enabled}
              onChange={(toMinutes) => {
                const nextRanges = [...ranges]
                nextRanges[index] = { ...range, toMinutes }
                onChange({ ...value, ranges: nextRanges })
              }}
            />
          </div>
          {ranges.length > 1 && (
            <button
              type="button"
              className="hours-range-remove"
              disabled={disabled || !value.enabled}
              aria-label={t('common.remove')}
              title={t('common.remove')}
              onClick={() =>
                onChange({
                  ...value,
                  ranges: ranges.filter((_, i) => i !== index)
                })
              }
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" aria-hidden>
                <path d="M4.5 7h15" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" />
                <path
                  d="M9.2 7V5.8c0-.7.55-1.3 1.25-1.3h3.1c.7 0 1.25.6 1.25 1.3V7"
                  stroke="currentColor"
                  strokeWidth="1.7"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
                <path
                  d="M18.2 7l-.7 11.1c-.08 1.05-.95 1.85-2 1.85H8.5c-1.05 0-1.92-.8-2-1.85L5.8 7"
                  stroke="currentColor"
                  strokeWidth="1.7"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
                <path
                  d="M10 11v5.2M14 11v5.2"
                  stroke="currentColor"
                  strokeWidth="1.7"
                  strokeLinecap="round"
                />
              </svg>
            </button>
          )}
        </div>
      ))}
      <Button
        type="default"
        size="small"
        className="hours-add-range"
        disabled={disabled || !value.enabled}
        onClick={() =>
          onChange({
            ...value,
            ranges: [...ranges, { fromMinutes: 13 * 60, toMinutes: 14 * 60 }]
          })
        }
      >
        {t('common.addRange')}
      </Button>
    </div>
  )
}

function UnitField({
  value,
  onChange,
  unit,
  min = 1,
  max = 9999
}: {
  value: number
  onChange: (next: number) => void
  unit: string
  min?: number
  max?: number
}): React.JSX.Element {
  return (
    <div className="field-with-unit">
      <InputNumber
        min={min}
        max={max}
        value={value}
        onChange={(v) => onChange(Number(v ?? min))}
        style={{ width: '100%' }}
      />
      <span className="field-unit">{unit}</span>
    </div>
  )
}

export function SettingsPage({
  onLocalePreferenceChange
}: {
  onLocalePreferenceChange: (preference: LocalePreference) => void
}): React.JSX.Element {
  const { message } = App.useApp()
  const { t } = useI18n()
  const [draft, setDraft] = useState<Settings>(DEFAULT_SETTINGS)
  const [saved, setSaved] = useState<Settings>(DEFAULT_SETTINGS)
  const [platform] = useState<NodeJS.Platform>(() =>
    hasNekoBridge() ? getNekoApi().platform : 'darwin'
  )
  const [bridgeError, setBridgeError] = useState<string | null>(() =>
    hasNekoBridge() ? null : t('settings.bridgeBrowser')
  )
  const [loading, setLoading] = useState(() => hasNekoBridge())
  const [tab, setTab] = useState<SettingsTab>('break')
  const [visitedTabs, setVisitedTabs] = useState<Set<string>>(() => new Set(['break']))
  const dirty = useMemo(() => JSON.stringify(draft) !== JSON.stringify(saved), [draft, saved])
  const runtime = useRuntimeStatus()

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

  useEffect(() => {
    if (!hasNekoBridge()) return

    let cancelled = false
    void getNekoApi()
      .getSettings()
      .then((settings) => {
        if (cancelled) return
        const merged = { ...DEFAULT_SETTINGS, ...settings }
        setDraft(merged)
        setSaved(merged)
        onLocalePreferenceChange(merged.locale ?? 'system')
        setLoading(false)
      })
      .catch((err: unknown) => {
        if (cancelled) return
        setBridgeError(err instanceof Error ? err.message : t('settings.loadFailed'))
        setLoading(false)
      })

    return () => {
      cancelled = true
    }
    // Mount-only: avoid wiping draft when locale/t identity changes.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const patch = <K extends keyof Settings>(key: K, value: Settings[K]): void => {
    setDraft((prev) => {
      const next = { ...prev, [key]: value }
      if (key === 'locale') onLocalePreferenceChange(value as LocalePreference)
      return next
    })
  }

  const save = async (): Promise<void> => {
    await getNekoApi().setSettings(draft)
    setSaved(draft)
    message.success(t('settings.saved'))
  }

  const reset = (): void => {
    setDraft(saved)
    onLocalePreferenceChange(saved.locale ?? 'system')
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
              <div className="settings-panel">
                <section className="settings-card">
                  <div className="settings-card-title">
                    <div>
                      <h3>{t('settings.enableBreaks')}</h3>
                      <p>{t('settings.enableBreaksHint')}</p>
                    </div>
                    <Switch
                      checked={draft.breaksEnabled}
                      onChange={(v) => patch('breaksEnabled', v)}
                    />
                  </div>
                  <Form layout="vertical" className="settings-form">
                    <Form.Item label={t('settings.frequency')}>
                      <UnitField
                        value={Math.max(1, Math.round(draft.breakFrequencySeconds / 60))}
                        unit={t('common.unit.minutes')}
                        min={1}
                        max={24 * 60}
                        onChange={(minutes) => patch('breakFrequencySeconds', minutes * 60)}
                      />
                    </Form.Item>
                    <Form.Item label={t('settings.breakLength')}>
                      <UnitField
                        value={draft.breakLengthSeconds}
                        unit={t('common.unit.seconds')}
                        min={5}
                        max={60 * 60}
                        onChange={(seconds) => patch('breakLengthSeconds', seconds)}
                      />
                    </Form.Item>
                    <Form.Item label={t('settings.notificationStyle')}>
                      <Select
                        value={draft.notificationType}
                        options={[
                          {
                            value: NotificationType.Popup,
                            label: t('settings.notification.popup')
                          },
                          {
                            value: NotificationType.Notification,
                            label: t('settings.notification.system')
                          }
                        ]}
                        onChange={(v) => patch('notificationType', v)}
                      />
                    </Form.Item>
                    <Form.Item label={t('settings.title')}>
                      <Input
                        value={draft.breakTitle}
                        placeholder={t('break.defaultTitle')}
                        onChange={(e) => patch('breakTitle', e.target.value)}
                      />
                    </Form.Item>
                    <Form.Item label={t('settings.message')} className="settings-form-full">
                      <Input.TextArea
                        rows={3}
                        maxLength={200}
                        showCount
                        value={draft.breakMessage}
                        placeholder={t('break.defaultMessage')}
                        onChange={(e) => patch('breakMessage', e.target.value)}
                      />
                    </Form.Item>
                  </Form>
                </section>

                <section className="settings-card">
                  <div className="settings-card-title">
                    <div>
                      <h3>{t('settings.snoozeSkip')}</h3>
                      <p>{t('settings.snoozeSkipHint')}</p>
                    </div>
                  </div>
                  <div className="settings-row">
                    <Text>{t('settings.allowSnooze')}</Text>
                    <Switch
                      checked={draft.postponeBreakEnabled}
                      onChange={(v) => patch('postponeBreakEnabled', v)}
                    />
                  </div>
                  <div className="settings-row">
                    <Text>{t('settings.allowSkip')}</Text>
                    <Switch
                      checked={draft.skipBreakEnabled}
                      onChange={(v) => patch('skipBreakEnabled', v)}
                    />
                  </div>
                  <div className="settings-row">
                    <Text>{t('settings.allowEndEarly')}</Text>
                    <Switch
                      checked={draft.endBreakEnabled}
                      onChange={(v) => patch('endBreakEnabled', v)}
                    />
                  </div>
                  <Form layout="vertical" className="settings-form tight">
                    <Form.Item label={t('settings.snoozeLength')}>
                      <UnitField
                        value={Math.max(1, Math.round(draft.postponeLengthSeconds / 60))}
                        unit={t('common.unit.minutes')}
                        min={1}
                        max={120}
                        onChange={(minutes) => patch('postponeLengthSeconds', minutes * 60)}
                      />
                    </Form.Item>
                    <Form.Item label={t('settings.snoozeLimit')}>
                      <UnitField
                        value={draft.postponeLimit}
                        unit={t('common.unit.times')}
                        min={0}
                        max={20}
                        onChange={(times) => patch('postponeLimit', times)}
                      />
                    </Form.Item>
                  </Form>
                </section>

                <section className="settings-card">
                  <div className="settings-card-title">
                    <div>
                      <h3>{t('settings.smartBreaks')}</h3>
                      <p>{t('settings.smartBreaksHint')}</p>
                    </div>
                    <Switch
                      checked={draft.idleResetEnabled}
                      onChange={(v) => patch('idleResetEnabled', v)}
                    />
                  </div>
                  <Form layout="vertical" className="settings-form tight">
                    <Form.Item label={t('settings.idleThreshold')}>
                      <UnitField
                        value={Math.max(1, Math.round(draft.idleResetLengthSeconds / 60))}
                        unit={t('common.unit.minutes')}
                        min={1}
                        max={240}
                        onChange={(minutes) => patch('idleResetLengthSeconds', minutes * 60)}
                      />
                    </Form.Item>
                  </Form>
                  <div className="settings-row">
                    <Text>{t('settings.idleNotify')}</Text>
                    <Switch
                      checked={draft.idleResetNotification}
                      onChange={(v) => patch('idleResetNotification', v)}
                    />
                  </div>
                </section>
              </div>
            </div>
          )}

          {visitedTabs.has('hours') && (
            <div
              className="settings-tab-panel"
              role="tabpanel"
              hidden={tab !== 'hours'}
              aria-hidden={tab !== 'hours'}
            >
              <div className="settings-panel">
                <section className="settings-card">
                  <div className="settings-card-title">
                    <div>
                      <h3>{t('settings.workingHours')}</h3>
                      <p>{t('settings.workingHoursHint')}</p>
                    </div>
                    <Switch
                      checked={draft.workingHoursEnabled}
                      onChange={(v) => patch('workingHoursEnabled', v)}
                    />
                  </div>
                  <div className="hours-grid">
                    {WORKING_HOURS_DAY_KEYS.map((key) => (
                      <div key={key} className="hours-block">
                        <Text strong>{t(DAY_LABEL_KEY[key] as 'day.monday')}</Text>
                        <RangeEditor
                          value={draft[key] ?? DEFAULT_SETTINGS[key]}
                          disabled={!draft.workingHoursEnabled}
                          onChange={(next) => patch(key, next)}
                        />
                      </div>
                    ))}
                  </div>
                </section>
              </div>
            </div>
          )}

          {visitedTabs.has('look') && (
            <div
              className="settings-tab-panel"
              role="tabpanel"
              hidden={tab !== 'look'}
              aria-hidden={tab !== 'look'}
            >
              <div className="settings-panel">
                <section className="settings-card">
                  <div className="settings-card-title">
                    <div>
                      <h3>{t('settings.breakAppearance')}</h3>
                      <p>{t('settings.breakAppearanceHint')}</p>
                    </div>
                  </div>
                  <div className="color-row">
                    <div className="color-field">
                      <span className="color-field-label">{t('settings.background')}</span>
                      <ColorPicker
                        value={draft.backgroundColor}
                        onChange={(color) => patch('backgroundColor', color.toHexString())}
                        showText
                      />
                    </div>
                    <div className="color-field">
                      <span className="color-field-label">{t('settings.text')}</span>
                      <ColorPicker
                        value={draft.textColor}
                        onChange={(color) => patch('textColor', color.toHexString())}
                        showText
                      />
                    </div>
                    <div className="color-field color-field-action">
                      <span className="color-field-label" aria-hidden>
                        &nbsp;
                      </span>
                      <Button
                        onClick={() => {
                          patch('backgroundColor', DEFAULT_SETTINGS.backgroundColor)
                          patch('textColor', DEFAULT_SETTINGS.textColor)
                        }}
                      >
                        {t('common.resetColors')}
                      </Button>
                    </div>
                  </div>
                  <Divider />
                  <Form layout="vertical" className="settings-form">
                    <Form.Item label={t('settings.popupStyle')} className="settings-form-full">
                      <Select
                        value={draft.breakPopupStyle}
                        options={[
                          {
                            value: BreakPopupStyle.Card,
                            label: t('settings.popupStyle.card')
                          },
                          {
                            value: BreakPopupStyle.Video,
                            label: t('settings.popupStyle.video')
                          }
                        ]}
                        onChange={(v) => patch('breakPopupStyle', v)}
                      />
                    </Form.Item>
                  </Form>
                  {draft.breakPopupStyle === BreakPopupStyle.Video ? (
                    <>
                      <div className="settings-row video-path-row">
                        <div className="video-path-meta">
                          <Text strong>{t('settings.importVideo')}</Text>
                          <Text type="secondary" ellipsis className="video-path-text">
                            {draft.breakVideoSource === BreakVideoSource.Custom &&
                            draft.breakVideoPath
                              ? draft.breakVideoPath.split(/[/\\]/).pop()
                              : t('settings.videoSource.builtin')}
                          </Text>
                        </div>
                        <Space wrap>
                          <Button
                            type="primary"
                            onClick={() => {
                              void getNekoApi()
                                .importBreakVideo()
                                .then((path) => {
                                  if (!path) return
                                  patch('breakVideoPath', path)
                                  patch('breakVideoSource', BreakVideoSource.Custom)
                                })
                            }}
                          >
                            {t('settings.importVideo')}
                          </Button>
                          {draft.breakVideoSource === BreakVideoSource.Custom &&
                          draft.breakVideoPath ? (
                            <Button
                              onClick={() => {
                                patch('breakVideoPath', '')
                                patch('breakVideoSource', BreakVideoSource.Builtin)
                              }}
                            >
                              {t('settings.useBuiltinVideo')}
                            </Button>
                          ) : null}
                        </Space>
                      </div>
                      <div className="settings-row">
                        <Text>{t('settings.videoMuted')}</Text>
                        <Switch
                          checked={draft.breakVideoMuted}
                          onChange={(v) => patch('breakVideoMuted', v)}
                        />
                      </div>
                    </>
                  ) : (
                    <>
                      <div className="settings-row">
                        <Text>{t('settings.backdrop')}</Text>
                        <Switch
                          checked={draft.showBackdrop}
                          onChange={(v) => patch('showBackdrop', v)}
                        />
                      </div>
                      <Form layout="vertical" className="settings-form">
                        <Form.Item
                          label={t('settings.backdropOpacity', {
                            percent: Math.round(draft.backdropOpacity * 100)
                          })}
                        >
                          <Slider
                            min={0.2}
                            max={0.92}
                            step={0.02}
                            value={draft.backdropOpacity}
                            disabled={!draft.showBackdrop}
                            onChange={(v) => patch('backdropOpacity', v)}
                          />
                        </Form.Item>
                      </Form>
                    </>
                  )}
                  <div
                    className={`look-preview ${
                      draft.breakPopupStyle === BreakPopupStyle.Video ? 'is-video' : 'is-card'
                    }`}
                  >
                    <div className="look-preview-scene" aria-hidden />
                    {draft.breakPopupStyle === BreakPopupStyle.Video ? (
                      <LookVideoPreview
                        source={draft.breakVideoSource}
                        path={draft.breakVideoPath}
                      />
                    ) : (
                      <>
                        {draft.showBackdrop && (
                          <div
                            className="look-preview-backdrop"
                            style={{
                              background: `rgba(8, 12, 10, ${draft.backdropOpacity})`
                            }}
                          />
                        )}
                        <div
                          className="look-preview-card"
                          style={{ background: draft.backgroundColor, color: draft.textColor }}
                        >
                          <span className="look-preview-kicker">{t('common.preview')}</span>
                          <strong>{resolveBreakTitle(draft.breakTitle, t)}</strong>
                          <p>{resolveBreakMessage(draft.breakMessage, t)}</p>
                        </div>
                      </>
                    )}
                  </div>
                  <div className="look-preview-actions">
                    <Button
                      type="primary"
                      onClick={() => {
                        void getNekoApi().previewBreak(draft)
                      }}
                    >
                      {t('settings.previewBreak')}
                    </Button>
                    <Text type="secondary">{t('settings.previewBreakHint')}</Text>
                  </div>
                </section>

                <section className="settings-card">
                  <div className="settings-card-title">
                    <div>
                      <h3>{t('settings.sound')}</h3>
                    </div>
                  </div>
                  <Form layout="vertical" className="settings-form">
                    <Form.Item label={t('settings.breakSound')}>
                      <Space>
                        <Select
                          style={{ minWidth: 180 }}
                          value={draft.soundType}
                          options={SOUND_TYPES.map((value) => ({
                            value,
                            label: t(SOUND_LABEL_KEY[value] as 'sound.none')
                          }))}
                          onChange={(v) => patch('soundType', v)}
                        />
                        <Button
                          disabled={draft.soundType === SoundType.None}
                          onClick={() =>
                            void getNekoApi().playStartSound(
                              draft.soundType,
                              draft.breakSoundVolume
                            )
                          }
                        >
                          {t('common.preview')}
                        </Button>
                      </Space>
                    </Form.Item>
                    <Form.Item
                      label={t('settings.volume', {
                        percent: Math.round(draft.breakSoundVolume * 100)
                      })}
                    >
                      <Slider
                        min={0}
                        max={1}
                        step={0.01}
                        value={draft.breakSoundVolume}
                        onChange={(v) => patch('breakSoundVolume', v)}
                      />
                    </Form.Item>
                  </Form>
                </section>
              </div>
            </div>
          )}

          {visitedTabs.has('system') && (
            <div
              className="settings-tab-panel"
              role="tabpanel"
              hidden={tab !== 'system'}
              aria-hidden={tab !== 'system'}
            >
              <div className="settings-panel">
                <section className="settings-card">
                  <div className="settings-card-title">
                    <div>
                      <h3>{t('locale.label')}</h3>
                      <p>{t('locale.hint')}</p>
                    </div>
                  </div>
                  <Form layout="vertical" className="settings-form">
                    <Form.Item label={t('locale.label')}>
                      <Select
                        value={draft.locale ?? 'system'}
                        options={LOCALE_OPTIONS.map((item) => ({
                          value: item.value,
                          label: t(item.labelKey as 'locale.system')
                        }))}
                        onChange={(v) => patch('locale', v)}
                      />
                    </Form.Item>
                  </Form>
                  <Divider />
                  <div className="settings-row">
                    <div>
                      <Text strong>{t('settings.autoLaunch')}</Text>
                      <Paragraph type="secondary">{t('settings.autoLaunchHint')}</Paragraph>
                    </div>
                    <Switch checked={draft.autoLaunch} onChange={(v) => patch('autoLaunch', v)} />
                  </div>
                  {platform === 'darwin' && (
                    <>
                      <Divider />
                      <div className="settings-row">
                        <div>
                          <Text strong>{t('settings.menuBarTimer')}</Text>
                          <Paragraph type="secondary">{t('settings.menuBarTimerHint')}</Paragraph>
                        </div>
                        <Switch
                          checked={draft.trayTextEnabled}
                          onChange={(v) => patch('trayTextEnabled', v)}
                        />
                      </div>
                      <Form layout="vertical" className="settings-form">
                        <Form.Item label={t('settings.menuBarMode')}>
                          <Select
                            value={draft.trayTextMode}
                            options={[
                              {
                                value: TrayTextMode.TimeToNextBreak,
                                label: t('settings.menuBar.next')
                              },
                              {
                                value: TrayTextMode.TimeSinceLastBreak,
                                label: t('settings.menuBar.since')
                              }
                            ]}
                            onChange={(v) => patch('trayTextMode', v)}
                          />
                        </Form.Item>
                      </Form>
                    </>
                  )}
                </section>
              </div>
            </div>
          )}
        </div>

        {dirty && (
          <footer className="settings-footer">
            <span className="settings-version">
              {t('settings.version', { version: APP_VERSION })}
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
