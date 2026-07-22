import { Form, Input, Select, Switch, Typography } from 'antd'
import { NotificationType, type Settings } from '@shared/settings'
import { useI18n } from '../../../i18n/use-i18n'
import { UnitField } from '../components/UnitField'
import type { SettingsPatch } from '../use-settings-draft'

const { Text } = Typography

export function BreakTab({
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
            <h3>{t('settings.enableBreaks')}</h3>
            <p>{t('settings.enableBreaksHint')}</p>
          </div>
          <Switch checked={draft.breaksEnabled} onChange={(v) => patch('breaksEnabled', v)} />
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
          <Switch checked={draft.skipBreakEnabled} onChange={(v) => patch('skipBreakEnabled', v)} />
        </div>
        <div className="settings-row">
          <Text>{t('settings.allowEndEarly')}</Text>
          <Switch checked={draft.endBreakEnabled} onChange={(v) => patch('endBreakEnabled', v)} />
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
          <Switch checked={draft.idleResetEnabled} onChange={(v) => patch('idleResetEnabled', v)} />
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
  )
}
