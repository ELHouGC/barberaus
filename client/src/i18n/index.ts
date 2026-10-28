// src/i18n/index.ts
import { createI18n } from 'vue-i18n'
import en from './locales/en'
import fr from './locales/fr'

// Single source of truth for supported locales (type + detection + future switcher).
export const SUPPORTED_LOCALES = ['en', 'fr'] as const
export type Locale = (typeof SUPPORTED_LOCALES)[number]
export const DEFAULT_LOCALE: Locale = 'en'

export type MessageSchema = typeof en

const STORAGE_KEY = 'locale'

const isLocale = (v: string | null): v is Locale =>
  SUPPORTED_LOCALES.some((l) => l === v)

const detectLocale = (): Locale => {
  const saved = localStorage.getItem(STORAGE_KEY)
  if (isLocale(saved)) return saved // explicit choice beats detection
  return SUPPORTED_LOCALES.find((l) => navigator.language.startsWith(l)) ?? DEFAULT_LOCALE
}

const i18n = createI18n<[MessageSchema], Locale, false>({
  legacy: false, // Composition API: useI18n()
  locale: detectLocale(),
  fallbackLocale: DEFAULT_LOCALE,
  messages: { en, fr },
})

export default i18n