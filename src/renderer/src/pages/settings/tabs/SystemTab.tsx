import { Divider, Form, Select, Switch, Typography } from 'antd'
import { LOCALE_OPTIONS } from '@shared/i18n'
import { TrayTextMode, type Settings } from '@shared/settings'
import { useI18n } from '../../../i18n/use-i18n'
import type { SettingsPatch } from '../use-settings-draft'

const { Paragraph, Text } = Typography

export function SystemTab({
  draft,
  patch,
  platform
}: {
  draft: Settings
  patch: SettingsPatch
  platform: NodeJS.Platform
}): React.JSX.Element {
  const { t } = useI18n()

  return (
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
  )
}
