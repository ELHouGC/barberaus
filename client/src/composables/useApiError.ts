// src/composables/useApiError.ts
import { useI18n } from 'vue-i18n'
import type { ProblemDetails } from '@/transport/envelopes'

/** Maps API problems to what the UI shows. Holds no state. */
export function useApiError() {
  const { t } = useI18n()

  /**
   * Translate a backend `i18n_key` (or a client-side problem key).
   * Unknown key => generic message. vue-i18n resolves fallbackLocale first and
   * already warns about the missing key in dev.
   */
  function translate(key: string): string {
    return t(key, t('errors.client.unexpected'))
  }

  /** Backend field errors -> { fieldName: i18nKey }, ready for vee-validate `setErrors`. */
  function fieldKeys(problem: ProblemDetails): Record<string, string> {
    const keys: Record<string, string> = {}
    for (const { pointer, key } of problem.errors ?? []) {
      keys[pointer.replace(/^\//, '')] = key // "/email" -> "email"
    }
    return keys
  }

  return { translate, fieldKeys }
}