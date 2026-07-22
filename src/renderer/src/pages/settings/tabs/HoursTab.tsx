import { Switch, Typography } from 'antd'
import { DEFAULT_SETTINGS, WORKING_HOURS_DAY_KEYS, type Settings } from '@shared/settings'
import { useI18n } from '../../../i18n/use-i18n'
import { RangeEditor } from '../components/RangeEditor'
import { DAY_LABEL_KEY } from '../settings-labels'
import type { SettingsPatch } from '../use-settings-draft'

const { Text } = Typography

export function HoursTab({
  draft,
  patch
}: {
  draft: Settings
  patch: SettingsPatch
}): React.JSX.Element {
  const { t } = useI18n()

  return (
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
  )
}
