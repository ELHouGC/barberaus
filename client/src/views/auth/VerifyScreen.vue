<script setup lang="ts">
import { ref, reactive, nextTick } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '@/stores/auth'
import { useApiError } from '@/composables/useApiError'
import AuthLayout from '@/components/layout/AuthLayout.vue'
import AuthButton from '@/components/auth/AuthButton.vue'

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()
const { isLoading } = storeToRefs(authStore)
const { t } = useI18n()
const { translate } = useApiError()

// Login and Signup both navigate here with ?userId=...&email=...
const email = (route.query.email as string | undefined) ?? ''
const userId = (route.query.userId as string | undefined) ?? ''

// 6 separate box states for code input
const digits = reactive(['', '', '', '', '', ''])
const inputRefs = ref<HTMLInputElement[]>([])

const errorKey = ref<string | null>(null)

const handleInput = (index: number, event: Event) => {
  const input = event.target as HTMLInputElement
  const value = input.value.replace(/[^0-9]/g, '')

  if (value) {
    digits[index] = value[value.length - 1]
    if (index < 5) {
      nextTick(() => {
        inputRefs.value[index + 1]?.focus()
      })
    }
  } else {
    digits[index] = ''
  }
}

const handleKeydown = (index: number, event: KeyboardEvent) => {
  if (event.key === 'Backspace') {
    if (!digits[index] && index > 0) {
      nextTick(() => {
        inputRefs.value[index - 1]?.focus()
      })
    } else {
      digits[index] = ''
    }
  }
}

const handlePaste = (event: ClipboardEvent) => {
  event.preventDefault()
  const pasteData = event.clipboardData?.getData('text').trim() || ''
  const numbers = pasteData.replace(/[^0-9]/g, '').split('')

  numbers.forEach((num, idx) => {
    if (idx < 6) digits[idx] = num
  })

  const focusIndex = Math.min(numbers.length, 5)
  inputRefs.value[focusIndex]?.focus()
}

const handleVerify = async () => {
  const code = digits.join('')
  if (code.length < 6) {
    errorKey.value = 'auth.verify.incomplete'
    return
  }

  if (!userId) {
    errorKey.value = 'auth.verify.missingUser'
    return
  }

  errorKey.value = null

  // The store opens the session (token + user) on success.
  const result = await authStore.verifyEmail({ user_id: userId, code })

  if (result.success) {
    router.push('/onboarding')
  } else {
    errorKey.value = result.error.i18n_key
  }
}
</script>

<template>
  <AuthLayout
    :title="t('auth.verify.title')"
    :subtitle="t('auth.verify.subtitle', { email: email || t('auth.verify.yourEmail') })"
  >
    <form @submit.prevent="handleVerify" class="mt-6 space-y-6">
      <div
        v-if="errorKey"
        class="rounded-lg border border-error-border bg-error-bg px-3 py-3 text-[13px] text-error-text"
      >
        {{ translate(errorKey) }}
      </div>

      <!-- 6-Digit Code Inputs -->
      <div>
        <label class="block text-xs font-medium text-tx-secondary mb-3">
          {{ t('auth.verify.codeLabel') }}
        </label>
        <div class="flex justify-between gap-1.5 sm:gap-2" @paste="handlePaste">
          <input
            v-for="(_, index) in digits"
            :key="index"
            :ref="el => { if (el) inputRefs[index] = el as HTMLInputElement }"
            type="text"
            maxlength="1"
            v-model="digits[index]"
            @input="handleInput(index, $event)"
            @keydown="handleKeydown(index, $event)"
            class="w-full h-12 sm:h-14 bg-app-bg border border-border-subtle rounded-xl text-center text-lg sm:text-xl font-bold text-tx-primary focus:border-brand focus:outline-none transition-colors min-w-0"
          />
        </div>
      </div>

      <AuthButton :isLoading="isLoading">
        {{ isLoading ? t('auth.verify.submitting') : t('auth.verify.submit') }}
      </AuthButton>
    </form>

    <template #footer>
      <i18n-t keypath="auth.verify.backToSignIn" tag="div" class="mt-4 text-tx-secondary text-sm">
        <template #link>
          <button
            type="button"
            @click="router.push('/login')"
            class="font-semibold text-brand hover:underline"
          >
            {{ t('auth.verify.signInLink') }}
          </button>
        </template>
      </i18n-t>
    </template>
  </AuthLayout>
</template>