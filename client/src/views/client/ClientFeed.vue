<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const authStore = useAuthStore()

const actionStatus = ref<string | null>(null)
const isActionLoading = ref(false)

// 1. Test Silent Refresh Flow
const handleTestRefresh = async () => {
  isActionLoading.value = true
  actionStatus.value = null

  const success = await authStore.refreshSession()
  isActionLoading.value = false

  if (success) {
    actionStatus.value = 'Session rotated successfully! New Access Token acquired.'
  } else {
    actionStatus.value = 'Refresh failed. Cookie was missing or invalid in Redis.'
  }
}

// 2. Test Revocation & Logout Flow
const handleLogout = async () => {
  isActionLoading.value = true
  await authStore.logout()
  isActionLoading.value = false

  // Redirect to login after session destruction
  router.push('/login')
}
</script>

<template>
  <div class="min-h-screen bg-app-bg text-tx-primary p-6 flex flex-col items-center justify-center">
    <div class="max-w-md w-full bg-surface border border-border-subtle rounded-2xl p-6 shadow-xl space-y-6">
      
      <!-- Header -->
      <div class="text-center">
        <h1 class="text-2xl font-bold">Dashboard</h1>
        <p class="text-xs text-tx-secondary mt-1">
          Authenticated Route Perimeter (`/clientfeed`)
        </p>
      </div>

      <!-- Live Session State -->
      <div class="bg-app-bg p-4 rounded-xl border border-border-subtle space-y-2 text-xs font-mono">
        <div class="flex justify-between">
          <span class="text-tx-muted">User ID:</span>
          <span class="text-brand font-semibold truncate max-w-[200px]">
            {{ authStore.user?.user_id || 'N/A' }}
          </span>
        </div>

        <div class="flex justify-between">
          <span class="text-tx-muted">Profile:</span>
          <span>{{ authStore.isBarberActive ? 'BARBER' : 'CLIENT' }}</span>
        </div>

        <div class="flex justify-between items-center">
          <span class="text-tx-muted">Access Token:</span>
          <span class="text-tx-secondary truncate max-w-[180px]">
            {{ authStore.accessToken ? `${authStore.accessToken.substring(0, 18)}...` : 'None' }}
          </span>
        </div>
      </div>

      

      <!-- Status Notification -->
      <div 
        v-if="actionStatus" 
        class="rounded-lg border border-brand/20 bg-brand/10 p-3 text-xs text-brand text-center"
      >
        {{ actionStatus }}
      </div>

      <!-- Test Actions -->
      <div class="space-y-3">
        <button 
          @click="handleTestRefresh" 
          :disabled="isActionLoading"
          class="w-full h-11 bg-surface-hover border border-border-subtle text-tx-primary rounded-xl font-medium text-sm transition-colors hover:bg-border-subtle hover:text-white disabled:opacity-50"
        >
          {{ isActionLoading ? 'Rotating...' : '🔄 Test Token Refresh' }}
        </button>

        <button 
          @click="handleLogout" 
          :disabled="isActionLoading"
          class="w-full h-11 bg-error-bg/20 border border-error-border text-error-text rounded-xl font-medium text-sm transition-colors hover:bg-error-bg hover:text-white disabled:opacity-50"
        >
          🚪 Logout
        </button>
      </div>

    </div>
  </div>
</template>