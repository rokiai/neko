import { Button, Modal } from 'antd'
import { useState } from 'react'
import { useI18n } from '../../../i18n/use-i18n'

export function AutoLaunchOnboarding({
  open,
  onEnable,
  onSkip
}: {
  open: boolean
  onEnable: () => Promise<void>
  onSkip: () => Promise<void>
}): React.JSX.Element {
  const { t } = useI18n()
  const [busy, setBusy] = useState(false)

  const run = async (action: () => Promise<void>): Promise<void> => {
    if (busy) return
    setBusy(true)
    try {
      await action()
    } finally {
      setBusy(false)
    }
  }

  return (
    <Modal
      open={open}
      centered
      closable={!busy}
      maskClosable={!busy}
      keyboard={!busy}
      footer={null}
      width={420}
      className="auto-launch-onboarding"
      onCancel={() => void run(onSkip)}
      destroyOnHidden
    >
      <div className="auto-launch-onboarding-body">
        <p className="auto-launch-onboarding-eyebrow">{t('app.name')}</p>
        <h2>{t('onboarding.autoLaunch.title')}</h2>
        <p className="auto-launch-onboarding-copy">{t('onboarding.autoLaunch.body')}</p>
        <div className="auto-launch-onboarding-actions">
          <Button type="primary" loading={busy} onClick={() => void run(onEnable)}>
            {t('onboarding.autoLaunch.enable')}
          </Button>
          <Button disabled={busy} onClick={() => void run(onSkip)}>
            {t('onboarding.autoLaunch.skip')}
          </Button>
        </div>
      </div>
    </Modal>
  )
}
