import { createI18n } from 'vue-i18n'
import zhCN from './locales/zh-CN.json'
import enUS from './locales/en-US.json'

export type MessageSchema = typeof zhCN
export type SupportedLocale = 'zh-CN' | 'en-US'

export const SUPPORTED_LOCALES: { value: SupportedLocale; label: string }[] = [
  { value: 'zh-CN', label: '简体中文' },
  { value: 'en-US', label: 'English' },
]

const savedLocale = (localStorage.getItem('spm_locale') || 'zh-CN') as SupportedLocale

const i18n = createI18n<[MessageSchema], SupportedLocale>({
  legacy: false,
  locale: savedLocale,
  fallbackLocale: 'zh-CN',
  messages: {
    'zh-CN': zhCN,
    'en-US': enUS,
  },
})

export default i18n
