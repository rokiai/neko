import { Button, ColorPicker, Divider, Form, Select, Slider, Space, Switch, Typography } from 'antd'
import { resolveBreakMessage, resolveBreakTitle } from '@shared/break-copy'
import {
  BreakPopupStyle,
  BreakVideoSource,
  DEFAULT_SETTINGS,
  SOUND_TYPES,
  SoundType,
  type Settings
} from '@shared/settings'
import { useI18n } from '../../../i18n/use-i18n'
import { getNekoApi } from '../../../lib/neko'
import { LookVideoPreview } from '../components/LookVideoPreview'
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
                  {draft.breakVideoSource === BreakVideoSource.Custom && draft.breakVideoPath
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
                {draft.breakVideoSource === BreakVideoSource.Custom && draft.breakVideoPath ? (
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
          </>
        )}
        <div
          className={`look-preview ${
            draft.breakPopupStyle === BreakPopupStyle.Video ? 'is-video' : 'is-card'
          }`}
        >
          <div className="look-preview-scene" aria-hidden />
          {draft.breakPopupStyle === BreakPopupStyle.Video ? (
            <LookVideoPreview source={draft.breakVideoSource} path={draft.breakVideoPath} />
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
