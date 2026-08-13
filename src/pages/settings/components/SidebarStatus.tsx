import { useI18n } from '../../../i18n/use-i18n'
import { useRuntimeStatus } from '../use-runtime-status'

/**
 * The only part of the sidebar that depends on the once-a-second runtime
 * status. Subscribing here keeps `SettingsPage` — and with it every visited
 * tab — out of the per-second render path.
 */
export function SidebarStatus(): React.JSX.Element {
  const { t } = useI18n()
  const runtime = useRuntimeStatus()

  return (
    <div className="settings-status">
      <div className="settings-status-dot">
        {runtime.breaksEnabled && !runtime.idle ? t('settings.status.running') : t('common.off')}
      </div>
    </div>
  )
}
