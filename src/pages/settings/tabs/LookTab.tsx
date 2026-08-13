import { Button, ColorPicker, Divider, Form, Select, Slider, Space, Switch, Typography } from 'antd'
import { resolveBreakMessage, resolveBreakTitle } from '@shared/break-copy'
import { DEFAULT_SETTINGS, SOUND_TYPES, SoundType, type Settings } from '@shared/settings'
import { useI18n } from '../../../i18n/use-i18n'
import { getNekoApi } from '../../../lib/neko'
import { SOUND_LABEL_KEY } from '../settings-labels'
import type { SettingsPatch } from '../use-settings-draft'

const { Text } = Typography

export function LookTab({
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
        <div className="settings-row">
          <Text>{t('settings.backdrop')}</Text>
          <Switch checked={draft.showBackdrop} onChange={(v) => patch('showBackdrop', v)} />
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
        <div className="look-preview is-card">
          <div className="look-preview-scene" aria-hidden />
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
                  void getNekoApi().playStartSound(draft.soundType, draft.breakSoundVolume)
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
  )
}
