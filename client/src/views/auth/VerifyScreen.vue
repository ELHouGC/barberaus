<script setup lang="ts">
import { ref, reactive, nextTick } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import AuthLayout from '@/components/layout/AuthLayout.vue'
import AuthButton from '@/components/ui/AuthButton.vue'

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()

const email = (route.query.email as string) || 'your email'
const userId = (route.query.userId as string) || (route.query.user_id as string) || ''

// 6 separate box states for code input
const digits = reactive(['', '', '', '', '', ''])
const inputRefs = ref<HTMLInputElement[]>([])

const loading = ref(false)
const errorMessage = ref<string | null>(null)

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
    errorMessage.value = 'Please enter all 6 digits.'
    return
  }

  if (!userId) {
    errorMessage.value = 'Missing user identification. Please sign up again.'
    return
  }

  errorMessage.value = null
  loading.value = true

  try {
    // Calling store method handles saving access token and session metadata
    await authStore.verifyEmail({
      user_id: userId,
      code: code
    })
    
    // Direct navigate to application post-verification
    router.push('/clientfeed')
  } catch (err: any) {
    errorMessage.value = err.message || 'Invalid verification code. Please try again.'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <AuthLayout 
    title="Verify Your Email" 
    :subtitle="`Enter the 6-digit code sent to ${email}.`"
  >
    <form @submit.prevent="handleVerify" class="mt-6 space-y-6">
      <div 
        v-if="errorMessage" 
        class="rounded-lg border border-error-border bg-error-bg px-3 py-3 text-[13px] text-error-text"
      >
        {{ errorMessage }}
      </div>

      <!-- 6-Digit Code Inputs -->
      <div>
        <label class="block text-xs font-medium text-tx-secondary mb-3">Verification Code</label>
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

      <AuthButton :isLoading="loading">
        {{ loading ? 'Verifying...' : 'Confirm Code' }}
      </AuthButton>
    </form>

    <template #footer>
      <div class="mt-4 text-tx-secondary text-sm">
        Back to 
        <button @click="router.push('/login')" class="font-semibold text-brand hover:underline">
          Sign In
        </button>
      </div>
    </template>
  </AuthLayout>
</template>