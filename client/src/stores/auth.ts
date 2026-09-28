import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { transportApi } from '@/transport/auth.api'
import type { 
  AuthResponse, 
  LoginRequest, 
  VerifyEmailRequest, 
  UserPayload 
} from '@/transport/auth.contracts'
import type { s } from 'vue-router/dist/router-CWoNjPRp.mjs'

export const useAuthStore = defineStore('auth', () => {
  // 1. State
  const accessToken = ref<string | null>(null)
  const user = ref<UserPayload | null>(null)
  const isLoading = ref<boolean>(false)

  // 2. Getters
  const isAuthenticated = computed(() => !!accessToken.value)
  const isBarberActive = computed(() => user.value?.is_barber_profile_active ?? false)

  // 3. Helper to set/hydrate session in memory
  function setSession(authData: AuthResponse) {
    accessToken.value = authData.access_token
    user.value = authData.user
  }
  function updateToken(access_token: string ) {
    accessToken.value = access_token;
  }

  function clearSession() {
    accessToken.value = null
    user.value = null
  }

  // 4. Actions

  /**
   * Verifies the email code.
   * On success, immediately stores the returned access token and user state.
   */
  async function verifyEmail(payload: VerifyEmailRequest): Promise<void> {
    isLoading.value = true
    try {
      const response = await transportApi.verify(payload)

      // Extract AuthResponse whether transportApi returns AxiosResponse or ApiResponse directly
      const authData = response.data

      if (authData?.access_token) {
        setSession(authData)
      } else {
        throw new Error('Invalid verification response structure from server.')
      }
    } catch (error: any) {
      clearSession()
      const message =
        error.response?.data?.error?.message ||
        error.message ||
        'Verification failed. Please check your code.'
      throw new Error(message)
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Logs in with email and password.
   */
  async function login(credentials: LoginRequest) {
    isLoading.value = true
    try {
      const response = await transportApi.login(credentials)
      const authData = response.data

      if (authData?.access_token) {
        setSession(authData)
        return { success: true }
      }

      return { success: false, errorType: 'invalid' as const }
    } catch (error: any) {
      clearSession()
      const status = error.response?.status
      const errorMsg = error.response?.data?.error?.message || ''

      if (status === 403 || errorMsg.includes('unverified')) {
        return {
          success: false,
          errorType: 'unverified' as const,
          userId: error.response?.data?.data?.user_id,
          email: credentials.email,
        }
      }

      if (status === 401 || status === 400) {
        return { success: false, errorType: 'invalid' as const }
      }

      return { success: false, errorType: 'network_error' as const }
    } finally {
      isLoading.value = false
    }
  }

  /**
   * Silent Token Refresh:
   * Called by HTTP interceptors when 401 is encountered, or on app initialization.
   */
  async function refreshSession(): Promise<boolean> {
    try {
      const response = await transportApi.refresh()
      if (response.data?.access_token) {
        setSession(response.data)
        return true
      }
      clearSession()
      return false
    } catch {
      clearSession()
      return false
    }
  }

  /**
   * Logout:
   * Calls backend logout (terminates Redis keys & deletes cookie) and clears local state.
   */
  async function logout(): Promise<void> {
    try {
      await transportApi.logout()
    } catch (err) {
      console.warn('Server logout failed, clearing local state anyway.', err)
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
    updateToken,
    clearSession,
    verifyEmail,
    login,
    refreshSession,
    logout,
  }
})