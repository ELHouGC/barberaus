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
import type { ApiErrorResponse, ApiResponse } from '@/transport/envelopes'

/** Outcome of an action that returns no payload: success, or the problem. */
export type AuthResult = { success: true } | ApiErrorResponse

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
  // Views get the problem back as-is (code, i18n_key, errors[]) and decide
  // what to show; the store does not map errors to UI concerns.

  /** login and verifyEmail are the same flow: call, then open the session. */
  async function authenticate(request: Promise<ApiResponse<AuthResponse>>): Promise<AuthResult> {
    isLoading.value = true
    try {
      const response = await request
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

  const login = (credentials: LoginRequest) => authenticate(transportApi.login(credentials))
  const verifyEmail = (payload: VerifyEmailRequest) => authenticate(transportApi.verify(payload))

  /**
   * Silent refresh. Used by the 401 interceptor (client.api.ts) and at app
   * start. Concurrent callers share one request: with rotating refresh
   * tokens, a parallel second call would present an already-used token.
   */
  let refreshInFlight: Promise<boolean> | null = null

  function refreshSession(): Promise<boolean> {
    console.log('auth store: refreshSession()')
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
    // Only a 401 (MISSING_TOKEN / INVALID_TOKEN / TOKEN_EXPIRED) means the
    // refresh credential is dead. Other 4xx (403, 409, 422...), 5xx and network
    // errors (status 0) say nothing about the session: keep it.
    if (response.error.status === 401) {
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