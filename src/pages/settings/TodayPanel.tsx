import type { RuntimeStatus } from '@shared/stats'
import { formatClockDuration, formatCountdown } from '@shared/time'
import { useI18n } from '../../i18n/use-i18n'
import './today-panel.css'

function Stars({ filled }: { filled: number }): React.JSX.Element {
  return (
    <span className="today-stars" aria-hidden>
      {Array.from({ length: 5 }, (_, i) => (
        <span key={i} className={i < filled ? 'is-on' : ''}>
          ★
        </span>
      ))}
    </span>
  )
}

function ProgressRing({ percent }: { percent: number }): React.JSX.Element {
  const size = 132
  const stroke = 11
  const r = (size - stroke) / 2
  const c = 2 * Math.PI * r
  const offset = c * (1 - Math.max(0, Math.min(100, percent)) / 100)

  return (
    <svg className="today-ring" width={size} height={size} viewBox={`0 0 ${size} ${size}`}>
      <circle
        className="today-ring-track"
        cx={size / 2}
        cy={size / 2}
        r={r}
        strokeWidth={stroke}
        fill="none"
      />
      <circle
        className="today-ring-value"
        cx={size / 2}
        cy={size / 2}
        r={r}
        strokeWidth={stroke}
        fill="none"
        strokeDasharray={c}
        strokeDashoffset={offset}
        strokeLinecap="round"
      />
    </svg>
  )
}

export function TodayPanel({ status }: { status: RuntimeStatus }): React.JSX.Element {
  const { t } = useI18n()

  const countdownLabel = status.havingBreak
    ? t('stats.onBreak')
    : status.outsideWorkingHours
      ? t('stats.outsideHoursShort')
      : status.secondsToNextBreak == null
        ? t('stats.countdownIdle')
        : formatCountdown(status.secondsToNextBreak)

  const countdownMeta = status.havingBreak
    ? t('stats.countdownOnBreakHint')
    : status.outsideWorkingHours
      ? t('stats.countdownOutsideHoursHint')
      : status.secondsToNextBreak == null
        ? t('stats.countdownIdleHint')
        : t('stats.countdownWorkHint')

  return (
    <div className="today-panel">
      <section className="today-card">
        <header className="today-card-head">
          <h3>{t('stats.todayTitle')}</h3>
        </header>

        <ul className="today-rows">
          <li>
            <span>{t('stats.worked')}</span>
            <strong>{formatClockDuration(status.today.workSeconds)}</strong>
          </li>
          <li>
            <span>{t('stats.rested')}</span>
            <strong>{formatClockDuration(status.today.restSeconds)}</strong>
          </li>
          <li>
            <span>{t('stats.completed')}</span>
            <strong>
              {status.today.completedBreaks} {t('common.unit.times')}
            </strong>
          </li>
          <li>
            <span>{t('stats.focus')}</span>
            <Stars filled={status.focusStars} />
          </li>
        </ul>

        <div className="today-progress">
          <ProgressRing percent={status.progressPercent} />
          <div className="today-progress-copy">
            <span>{t('stats.progress')}</span>
            <strong>{status.progressPercent}%</strong>
            <em>{t('stats.goal', { count: status.dailyGoal })}</em>
          </div>
        </div>

        <div className="today-tip">{t('stats.tip')}</div>
      </section>

      <section className="today-card today-countdown">
        <header className="today-card-head">
          <h3>{t('stats.countdownTitle')}</h3>
          <p>{t('stats.countdownSubtitle')}</p>
        </header>
        <div className="today-countdown-value">{countdownLabel}</div>
        <p className="today-countdown-meta">{countdownMeta}</p>
      </section>
    </div>
  )
}
