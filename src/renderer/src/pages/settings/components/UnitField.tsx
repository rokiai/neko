import { InputNumber } from 'antd'

export function UnitField({
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
