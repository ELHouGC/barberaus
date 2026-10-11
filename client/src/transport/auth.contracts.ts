
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

export type SignupGender = 'Male' | 'Female' | 'PreferNotToSay'

export interface SignupRequest {
  full_name: string
  email: string
  password: string
  date_of_birth: string // ISO date string "YYYY-MM-DD"
  gender: SignupGender
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