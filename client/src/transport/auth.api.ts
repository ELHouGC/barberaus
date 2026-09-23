import axios from "axios";
import type { AuthResponse, LoginRequest, MessageResponse, ResendVerificationRequest, SignupRequest, SignupResponse, VerifyEmailRequest } from "./auth.contracts";
import { apiClient, setAccessToken } from "./client.api";
import type { ApiErrorResponse, ApiResponse } from "./envelopes";

export const transportApi = {
  async signup(data: SignupRequest): Promise<ApiResponse<SignupResponse>> {
    try {
      const response = await apiClient.post<ApiResponse<SignupResponse>>('/auth/signup', data);
      return response.data;
    } catch (err) {
      return handleApiError(err);
    }
  },

  async resendVerification(data: ResendVerificationRequest): Promise<ApiResponse<MessageResponse>> {
    try {
      const response = await apiClient.post<ApiResponse<MessageResponse>>('/auth/resend-verification', data);
      return response.data;
    } catch (err) {
      return handleApiError(err);
    }
  },

  async verify(data: VerifyEmailRequest): Promise<ApiResponse<AuthResponse>> {
    try {
      const response = await apiClient.post<ApiResponse<AuthResponse>>('/auth/verify-email', data);
      if (response.data.success) {
        setAccessToken(response.data.data.access_token);
      }
      return response.data;
    } catch (err) {
      return handleApiError(err);
    }
  },

  async login(credentials: LoginRequest): Promise<ApiResponse<AuthResponse>> {
    try {
      const response = await apiClient.post<ApiResponse<AuthResponse>>('/auth/login', credentials);
      if (response.data.success) {
        setAccessToken(response.data.data.access_token);
      }
      return response.data;
    } catch (err) {
      return handleApiError(err);
    }
  },

  async refresh(): Promise<ApiResponse<AuthResponse>> {
    try {
      const response = await apiClient.post<ApiResponse<AuthResponse>>('/auth/session/refresh');
      if (response.data.success) {
        setAccessToken(response.data.data.access_token);
      }
      return response.data;
    } catch (err) {
      return handleApiError(err);
    }
  },

  async logout(): Promise<ApiResponse<MessageResponse>> {
    try {
      // Receives refresh cookie via Path=/api/v1/auth/session and clears it with Set-Cookie Max-Age=0
      const response = await apiClient.post<ApiResponse<MessageResponse>>('/auth/session/logout');
      setAccessToken(null);
      return response.data;
    } catch (err) {
      setAccessToken(null);
      return handleApiError(err);
    }
  },
};

function handleApiError<T>(err: unknown): ApiResponse<T> {
  if (axios.isAxiosError(err) && err.response?.data) {
    const apiError = err.response.data as ApiErrorResponse;
    if (apiError.error) {
      return apiError;
    }
  }

  return {
    success: false,
    error: {
      code: 'NETWORK_ERROR',
      message: err instanceof Error ? err.message : 'Unable to connect to backend server.',
    },
  };
}