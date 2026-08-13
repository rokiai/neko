import type { ThemeConfig } from 'antd'
import { theme } from 'antd'

/** Fresh mint-on-mist light theme — rounded, airy, nature-inspired. */
export const nekoTheme: ThemeConfig = {
  algorithm: theme.defaultAlgorithm,
  token: {
    colorPrimary: '#3F9F7E',
    colorInfo: '#3F9F7E',
    colorSuccess: '#4CAF7A',
    colorWarning: '#E0A25A',
    colorError: '#E57373',
    colorBgBase: '#F4F8F5',
    colorBgContainer: 'rgba(255, 255, 255, 0.86)',
    colorBgElevated: '#FFFFFF',
    colorBorder: 'rgba(74, 120, 96, 0.14)',
    colorBorderSecondary: 'rgba(74, 120, 96, 0.08)',
    colorText: '#24352C',
    colorTextSecondary: '#5F7468',
    colorTextTertiary: '#8AA092',
    borderRadius: 14,
    borderRadiusLG: 20,
    fontFamily: '"Nunito", "Noto Sans SC", "Segoe UI", sans-serif',
    fontSize: 14,
    controlHeight: 38,
    boxShadow: '0 10px 30px rgba(55, 96, 78, 0.08)',
    boxShadowSecondary: '0 6px 18px rgba(55, 96, 78, 0.06)'
  },
  components: {
    Button: {
      primaryShadow: '0 8px 18px rgba(63, 159, 126, 0.22)',
      defaultShadow: 'none',
      borderRadius: 12,
      fontWeight: 700,
      defaultBorderColor: 'rgba(63, 159, 126, 0.22)',
      defaultColor: '#2F7A60',
      defaultHoverBorderColor: 'rgba(63, 159, 126, 0.42)',
      defaultHoverColor: '#2F7A60',
      defaultHoverBg: 'rgba(63, 159, 126, 0.08)',
      defaultActiveBorderColor: 'rgba(63, 159, 126, 0.5)',
      defaultActiveColor: '#2F7A60',
      defaultActiveBg: 'rgba(63, 159, 126, 0.12)'
    },
    Switch: {
      colorPrimary: '#3F9F7E',
      colorPrimaryHover: '#359270'
    },
    Input: {
      activeBorderColor: '#3F9F7E',
      hoverBorderColor: '#6BB89A',
      activeShadow: '0 0 0 3px rgba(63, 159, 126, 0.12)'
    },
    InputNumber: {
      activeBorderColor: '#3F9F7E',
      hoverBorderColor: '#6BB89A',
      activeShadow: '0 0 0 3px rgba(63, 159, 126, 0.12)'
    },
    Select: {
      optionSelectedBg: 'rgba(63, 159, 126, 0.12)',
      optionSelectedColor: '#24352C'
    },
    Slider: {
      trackBg: '#3F9F7E',
      trackHoverBg: '#359270',
      handleColor: '#3F9F7E',
      railBg: 'rgba(63, 159, 126, 0.14)'
    },
    Message: {
      contentBg: '#FFFFFF'
    },
    Divider: {
      colorSplit: 'rgba(74, 120, 96, 0.1)'
    }
  }
}
