// src/transport/auth.api.ts
import type {
  AuthResponse,
  LoginRequest,
  MessageResponse,
  ResendVerificationRequest,
  SignupRequest,
  SignupResponse,
  VerifyEmailRequest,
} from './auth.contracts';
import { api } from './http';

// Pure HTTP: no store access. `api.*` never throws and returns ApiResponse<T>.
// Session state is owned by the auth store.
export const transportApi = {
  signup: (data: SignupRequest) => api.post<SignupResponse>('/auth/signup', data),

  resendVerification: (data: ResendVerificationRequest) =>
    api.post<MessageResponse>('/auth/resend-verification', data),

  verify: (data: VerifyEmailRequest) => api.post<AuthResponse>('/auth/verify-email', data),

  login: (credentials: LoginRequest) => api.post<AuthResponse>('/auth/login', credentials),

  refresh: () => api.post<AuthResponse>('/auth/session/refresh'),

  logout: () => api.post<MessageResponse>('/auth/session/logout'),
};