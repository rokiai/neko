import enUS from 'antd/locale/en_US'
import zhCN from 'antd/locale/zh_CN'
import jaJP from 'antd/locale/ja_JP'
import type { Locale } from 'antd/es/locale'
import type { AppLocale } from '@shared/i18n'

export function antdLocale(locale: AppLocale): Locale {
  switch (locale) {
    case 'zh':
      return zhCN
    case 'ja':
      return jaJP
    default:
      return enUS
  }
}
