import { Select, Space } from 'antd'

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

export function MinutesOfDaySelect({
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
