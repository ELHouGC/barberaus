import type { 
  AuthResponse, 
  LoginRequest, 
  MessageResponse, 
  ResendVerificationRequest, 
  SignupRequest, 
  SignupResponse, 
  VerifyEmailRequest 
} from "./auth.contracts";
import { api } from "./client.api";
import { useAuthStore } from "@/stores/auth"; // Adjust import path to match your store
import type { ApiResponse } from "./envelopes";

export const transportApi = {
  async signup(data: SignupRequest): Promise<ApiResponse<SignupResponse>> {
    try {
      return await api.post<ApiResponse<SignupResponse>>('/auth/signup', data);
    } catch (err) {
      return handleApiError(err);
    }
  },

  async resendVerification(data: ResendVerificationRequest): Promise<ApiResponse<MessageResponse>> {
    try {
      return await api.post<ApiResponse<MessageResponse>>('/auth/resend-verification', data);
    } catch (err) {
      return handleApiError(err);
    }
  },

  async verify(data: VerifyEmailRequest): Promise<ApiResponse<AuthResponse>> {
    try {
      const response = await api.post<ApiResponse<AuthResponse>>('/auth/verify-email', data);
      if (response.success && response.data) {
        const authStore = useAuthStore();
        authStore.setSession(response.data);
      }
      return response;
    } catch (err) {
      return handleApiError(err);
    }
  },

  async login(credentials: LoginRequest): Promise<ApiResponse<AuthResponse>> {
    try {
      const response = await api.post<ApiResponse<AuthResponse>>('/auth/login', credentials);

      return response;
    } catch (err) {
      return handleApiError(err);
    }
  },

  async refresh(): Promise<ApiResponse<AuthResponse>> {
    try {
      const response = await api.post<ApiResponse<AuthResponse>>('/auth/session/refresh');
      if (response.success && response.data) {
        const authStore = useAuthStore();
        authStore.setSession(response.data);
      }
      return response;
    } catch (err) {
      return handleApiError(err);
    }
  },

  async logout(): Promise<ApiResponse<MessageResponse>> {
    const authStore = useAuthStore();
    try {
      const response = await api.post<ApiResponse<MessageResponse>>('/auth/session/logout');
      authStore.clearSession();
      return response;
    } catch (err) {
      authStore.clearSession();
      return handleApiError(err);
    }
  },
};

function handleApiError<T>(err: unknown): ApiResponse<T> {
  return {
    success: false,
    error: {
      code: 'NETWORK_ERROR',
      message: err instanceof Error ? err.message : 'Unable to connect to backend server.',
    },
  };
}