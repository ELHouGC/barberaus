<script setup lang="ts">
import { ref, onBeforeUnmount } from 'vue'
import { useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '@/stores/auth'
import { useApiError } from '@/composables/useApiError'

import AuthLayout from '@/components/layout/AuthLayout.vue'
import AuthInput from '@/components/auth/AuthInput.vue'
import AuthButton from '@/components/auth/AuthButton.vue'


const router = useRouter()
const authStore = useAuthStore()
const { isLoading } = storeToRefs(authStore) // replaces the local `loading` ref
const { t } = useI18n()
const { translate } = useApiError()
 
// Form State
const email = ref('')
const password = ref('')
const showPassword = ref(false)
 
// i18n key of the alert: a server key (errors.server.*), a client key
// (errors.client.*) or a frontend one (auth.login.*). Translated at render.
const alertKey = ref<string | null>(null)
 
const UNVERIFIED_REDIRECT_DELAY_MS = 2000
let redirectTimer: ReturnType<typeof setTimeout> | undefined
onBeforeUnmount(() => clearTimeout(redirectTimer))
 
const handleLogin = async () => {
  alertKey.value = null
  const submittedEmail = email.value // the field may change during the redirect delay
 
  const result = await authStore.login({
    email: submittedEmail,
    password: password.value,
  })
 
  if (result.success) {
    router.push('/clientfeed')
    return
  }
 
  const { error } = result
 
  // The only code that needs more than a message (problem.rs sends user_id
  // for this code alone): send the user to email verification.
  if (error.code === 'ACCOUNT_NOT_VERIFIED' && error.user_id) {
    const userId = error.user_id
    alertKey.value = 'auth.login.unverifiedRedirect'
    redirectTimer = setTimeout(() => {
      router.push({ path: '/verify', query: { userId, email: submittedEmail } })
    }, UNVERIFIED_REDIRECT_DELAY_MS)
    return
  }
 
  // Everything else (invalid_credentials, invalid_input, internal_error,
  // network...) is just its own i18n_key.
  alertKey.value = error.i18n_key
}
</script>

<template>

  <AuthLayout :title="t('auth.login.title')" :subtitle="t('auth.login.subtitle')">
    <form @submit.prevent="handleLogin" class="mt-6 space-y-4">
      
      <!-- Alert States -->
      <div 
        v-if="alertKey" 
        class="rounded-lg border border-error-border bg-error-bg px-3 py-3 text-[13px] text-error-text"
      >

      
        {{ translate(alertKey) }}
      </div>

      <!-- Inputs -->
      <AuthInput 
        id="email"
        type="email"
        :label="t('auth.fields.email.label')" 
        v-model="email" 
        placeholder="name@example.com" 
      />

      <AuthInput 
        id="password"
        :type="showPassword ? 'text' : 'password'"
        :label="t('auth.fields.password.label')" 
        v-model="password" 
        placeholder="••••••••"
      >
        <template #icon>
          <button
            type="button"
            @click="showPassword = !showPassword"
            class="text-tx-muted transition-colors hover:text-tx-secondary"
          >
            <!-- Eye-off icon -->
            <svg v-if="showPassword" class="size-4" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" viewBox="0 0 24 24">
              <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94" />
              <path d="M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19" />
              <path d="m1 1 22 22" />
            </svg>
            <!-- Eye icon -->
            <svg v-else class="size-4" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" viewBox="0 0 24 24">
              <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8Z" />
              <circle cx="12" cy="12" r="3" />
            </svg>
          </button>
        </template>
      </AuthInput>

      <!-- Submit -->
      <AuthButton :isLoading="isLoading">
        {{ isLoading ? t('auth.login.submitting') : t('auth.login.submit') }}
      </AuthButton>
    </form>

    <!-- Footer & Links -->
    <template #footer>
        <div class="mb-4 text-tx-secondary">
          {{ t('auth.login.noAccount') }}
          <button
            @click="router.push('/signup')"
            class="font-semibold text-brand hover:underline"
          >
            {{ t('auth.login.signUpLink') }}
          </button>
        </div>
    </template>

  </AuthLayout>
</template>


