
export interface UserPayload {
  id: string;
  email: string;
  full_name: string;
  [key: string]: unknown;
}

export interface LoginRequest {
  email: string;
  password?: string;
}

export interface AuthResponse {
  access_token: string;
  token_type: string;
  user: UserPayload;
}

export interface SignupRequest {
  email: string;
  password?: string;
  full_name: string;
}

export interface SignupResponse {
  message: string;
  user_id: string;
}

export interface VerifyEmailRequest {
  user_id: string;
  code: string;
}

export interface ResendVerificationRequest {
  user_id: string;
  email: string;
}

export interface MessageResponse {
  message: string;
}