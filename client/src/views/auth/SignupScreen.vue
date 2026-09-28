<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { transportApi } from '@/transport/auth.api'
import AuthLayout from '@/components/layout/AuthLayout.vue'
import AuthInput from '@/components/ui/AuthInput.vue'
import AuthButton from '@/components/ui/AuthButton.vue'

const router = useRouter()

const fullName = ref('')
const email = ref('')
const password = ref('')
const loading = ref(false)
const errorMessage = ref<string | null>(null)
const showPassword = ref(false)

const handleSignup = async () => {
  errorMessage.value = null
  loading.value = true

  try {
    const response = await transportApi.signup({
      full_name: fullName.value,
      email: email.value,
      password: password.value
    })
    
    // 1. Check if the wrapper returned an error state
    if (!response.success) {
      throw new Error(response.error?.message || 'Failed to create account.')
    }
    
    // 2. Safe to extract user_id now
    const userId = response.error ? null : (response as any).user_id // Adjust based on your actual data structure wrapper
    
    router.push({ 
      path: '/verify', 
      query: { 
        email: email.value, 
        userId: userId 
      } 
    })
  } catch (err: any) {
    errorMessage.value = err.message || 'Failed to create account. Please try again.'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <AuthLayout 
    title="Create an Account" 
    subtitle="Get started managing your shop with ClientFeed."
  >
    <form @submit.prevent="handleSignup" class="mt-6 space-y-4">
      <div 
        v-if="errorMessage" 
        class="rounded-lg border border-error-border bg-error-bg px-3 py-3 text-[13px] text-error-text"
      >
        {{ errorMessage }}
      </div>

      <AuthInput 
        id="fullname"
        type="text"
        label="Full Name" 
        v-model="fullName" 
        placeholder="John Doe" 
      />

      <AuthInput 
        id="email"
        type="email"
        label="Email Address" 
        v-model="email" 
        placeholder="name@example.com" 
      />

      <AuthInput
        id="password"
        :type="showPassword ? 'text' : 'password'"
        label="Password" 
        v-model="password" 
        placeholder="********">
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

      <AuthButton :isLoading="loading">
        {{ loading ? 'Creating account...' : 'Create Account' }}
      </AuthButton>
    </form>

    <template #footer>
      <div class="mb-4 text-tx-secondary">
        Already have an account? 
        <button 
          @click="router.push('/login')" 
          class="font-semibold text-brand hover:underline"
        >
          Sign in
        </button>
      </div>
    </template>
  </AuthLayout>
</template>