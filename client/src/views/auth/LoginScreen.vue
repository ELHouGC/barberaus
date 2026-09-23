<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import AuthLayout from '@/components/layout/AuthLayout.vue'
import AuthInput from '@/components/ui/AuthInput.vue'
import AuthButton from '@/components/ui/AuthButton.vue'

const router = useRouter()
const authStore = useAuthStore()

// Form State
const email = ref('')
const password = ref('')
const showPassword = ref(false)

// Submission & Alert State
const loading = ref(false)
const alertType = ref<'none' | 'unverified' | 'invalid' | 'network_error'>('none')

const handleLogin = async () => {
  loading.value = true;
  alertType.value = 'none';

  const result = await authStore.login({
    email: email.value,
    password: password.value,
  });

  loading.value = false;

  if (result.success) {
    router.push('/clientfeed');
  } else if (result.errorType === 'unverified') {
    alertType.value = 'unverified';
    
    // Auto-redirect to /verify with payload after showing notification
    setTimeout(() => {
      router.push({
        path: '/verify',
        query: {
          userId: result.userId ,
          email: result.email || email.value,
        },
      });
    }, 2000);
  } else if (result.errorType === 'invalid') {
    alertType.value = 'invalid';
  } else {
    alertType.value = 'network_error';
  }
};
</script>

<template>
  <AuthLayout 
    title="Welcome Back" 
    subtitle="Sign in to access your shop dashboard and appointments."
  >
    <form @submit.prevent="handleLogin" class="mt-6 space-y-4">
      
      <!-- Alert States -->
      <div 
        v-if="alertType === 'unverified'" 
        class="rounded-lg border border-error-border bg-error-bg px-3 py-3 text-[13px] text-error-text"
      >
        Your email address is not verified yet. Redirecting to verification page...
      </div>

      <div 
        v-else-if="alertType === 'invalid'" 
        class="rounded-lg border border-error-border bg-error-bg px-3 py-3 text-[13px] text-error-text"
      >
        Invalid email or password. Please try again.
      </div>

      <div 
        v-else-if="alertType === 'network_error'" 
        class="rounded-lg border border-error-border bg-error-bg px-3 py-3 text-[13px] text-error-text"
      >
        Unable to connect to the server. Please check your connection.
      </div>

      <!-- Inputs -->
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
      <AuthButton :isLoading="loading">
        {{ loading ? 'Signing in...' : 'Sign In' }}
      </AuthButton>
    </form>

    <!-- Footer & Links -->
    <template #footer>
      <div class="mb-4 text-tx-secondary">
        Don't have an account? 
        <button 
          @click="router.push('/signup')" 
          class="font-semibold text-brand hover:underline"
        >
          Sign up
        </button>
      </div>
    </template>
  </AuthLayout>
</template>