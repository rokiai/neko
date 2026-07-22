import { Button, Switch, Typography } from 'antd'
import type { WorkingHours } from '@shared/settings'
import { useI18n } from '../../../i18n/use-i18n'
import { MinutesOfDaySelect } from './MinutesOfDaySelect'

const { Text } = Typography

export function RangeEditor({
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
