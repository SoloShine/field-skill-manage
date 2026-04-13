import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { zhCN, dateZhCN, enUS, dateEnUS } from 'naive-ui'
import type { NLocale, NDateLocale } from 'naive-ui'
import type { SupportedLocale } from './index'

const STORAGE_KEY = 'spm_locale'

export function useLocale() {
  const { locale, t } = useI18n()

  const currentLocale = computed(() => locale.value as SupportedLocale)

  function setLocale(newLocale: SupportedLocale) {
    locale.value = newLocale
    localStorage.setItem(STORAGE_KEY, newLocale)
    document.documentElement.lang = newLocale
  }

  const naiveLocale = computed<NLocale>(() => {
    return locale.value === 'en-US' ? enUS : zhCN
  })

  const naiveDateLocale = computed<NDateLocale>(() => {
    return locale.value === 'en-US' ? dateEnUS : dateZhCN
  })

  return { currentLocale, setLocale, naiveLocale, naiveDateLocale, t }
}
