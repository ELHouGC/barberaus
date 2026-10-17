// src/stores/auth.ts
import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { transportApi } from '@/transport/auth.api'
import type {
  AuthResponse,
  LoginRequest,
  UserPayload,
  VerifyEmailRequest,
} from '@/transport/auth.contracts'
import type { ApiErrorResponse, ProblemDetails } from '@/transport/envelopes'

/** Outcome of an action that returns no payload: success, or the problem. */
export type AuthResult = { success: true } | ApiErrorResponse

export type LoginResult =
  | { success: true }
  | { success: false; errorType: 'unverified'; userId?: string; email: string }
  | { success: false; errorType: 'invalid' }
  | { success: false; errorType: 'other'; error: ProblemDetails }

export const useAuthStore = defineStore('auth', () => {
  // 1. State
  const accessToken = ref<string | null>(null)
  const user = ref<UserPayload | null>(null)
  const isLoading = ref<boolean>(false)

  // 2. Getters
  const isAuthenticated = computed(() => !!accessToken.value)
  const isBarberActive = computed(() => user.value?.is_barber_profile_active ?? false)

  // 3. Session state (the store is the only place that mutates it)
  function setSession(authData: AuthResponse) {
    accessToken.value = authData.access_token
    user.value = authData.user
  }

  function clearSession() {
    accessToken.value = null
    user.value = null
  }

  // 4. Actions
  // transportApi never throws: every action branches on `response.success`.

  async function verifyEmail(payload: VerifyEmailRequest): Promise<AuthResult> {
    isLoading.value = true
    try {
      const response = await transportApi.verify(payload)
      if (!response.success) {
        clearSession()
        return response
      }
      setSession(response.data)
      return { success: true }
    } finally {
      isLoading.value = false
    }
  }

  async function login(credentials: LoginRequest): Promise<LoginResult> {
    isLoading.value = true
    try {
      const response = await transportApi.login(credentials)
      if (response.success) {
        setSession(response.data)
        return { success: true }
      }

      clearSession()
      switch (response.error.code) {
        case 'ACCOUNT_NOT_VERIFIED':
          return {
            success: false,
            errorType: 'unverified',
            userId: response.error.user_id,
            email: credentials.email,
          }
        case 'INVALID_CREDENTIALS':
          return { success: false, errorType: 'invalid' }
        default:
          return { success: false, errorType: 'other', error: response.error }
      }
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Silent refresh. Used by the 401 interceptor (client.api.ts) and at app
   * start. Concurrent callers share one request: with rotating refresh
   * tokens, a parallel second call would present an already-used token.
   */
  let refreshInFlight: Promise<boolean> | null = null

  function refreshSession(): Promise<boolean> {
    refreshInFlight ??= performRefresh().finally(() => {
      refreshInFlight = null
    })
    return refreshInFlight
  }

  async function performRefresh(): Promise<boolean> {
    const response = await transportApi.refresh()
    if (response.success) {
      setSession(response.data)
      return true
    }
    // Rejected by the server (4xx) => session is over.
    // Network error (status 0) or 5xx => keep it, the user may just be offline.
    if (response.error.status >= 400 && response.error.status < 500) {
      clearSession()
    }
    return false
  }

  /** The local session ends whatever the server answered. */
  async function logout(): Promise<void> {
    try {
      await transportApi.logout()
    } finally {
      clearSession()
    }
  }

  return {
    // State & Getters
    accessToken,
    user,
    isLoading,
    isAuthenticated,
    isBarberActive,
    // Actions
    setSession,
    clearSession,
    verifyEmail,
    login,
    refreshSession,
    logout,
  }
})