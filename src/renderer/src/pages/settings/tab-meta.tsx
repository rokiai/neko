import type { ReactNode } from 'react'

export type SettingsTab = 'break' | 'hours' | 'look' | 'system'

export const TAB_META: Array<{
  key: SettingsTab
  labelKey: string
  descKey: string
  icon: ReactNode
}> = [
  {
    key: 'break',
    labelKey: 'settings.tab.breaks',
    descKey: 'settings.tab.breaksDesc',
    icon: (
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" aria-hidden>
        <path
          d="M12 3c-2.2 2.4-3.5 4.6-3.5 7a3.5 3.5 0 1 0 7 0c0-2.4-1.3-4.6-3.5-7Z"
          stroke="currentColor"
          strokeWidth="1.8"
          strokeLinejoin="round"
        />
        <path
          d="M8 17c1.2 1.5 2.5 2.5 4 2.5s2.8-1 4-2.5"
          stroke="currentColor"
          strokeWidth="1.8"
          strokeLinecap="round"
        />
      </svg>
    )
  },
  {
    key: 'hours',
    labelKey: 'settings.tab.hours',
    descKey: 'settings.tab.hoursDesc',
    icon: (
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" aria-hidden>
        <circle cx="12" cy="12" r="8" stroke="currentColor" strokeWidth="1.8" />
        <path
          d="M12 8v4.5l3 1.5"
          stroke="currentColor"
          strokeWidth="1.8"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </svg>
    )
  },
  {
    key: 'look',
    labelKey: 'settings.tab.look',
    descKey: 'settings.tab.lookDesc',
    icon: (
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" aria-hidden>
        <rect
          x="3.5"
          y="5"
          width="17"
          height="12"
          rx="2.5"
          stroke="currentColor"
          strokeWidth="1.8"
        />
        <path d="M8 20h8" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" />
        <path d="M12 17v3" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" />
      </svg>
    )
  },
  {
    key: 'system',
    labelKey: 'settings.tab.system',
    descKey: 'settings.tab.systemDesc',
    icon: (
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" aria-hidden>
        <circle cx="12" cy="12" r="3" stroke="currentColor" strokeWidth="1.8" />
        <path
          d="M12 4.5v2M12 17.5v2M4.5 12h2M17.5 12h2M6.4 6.4l1.4 1.4M16.2 16.2l1.4 1.4M17.6 6.4l-1.4 1.4M7.8 16.2l-1.4 1.4"
          stroke="currentColor"
          strokeWidth="1.8"
          strokeLinecap="round"
        />
      </svg>
    )
  }
]
