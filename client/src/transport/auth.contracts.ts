// src/transport/auth.contracts.ts

/**
 * Mirrors the Rust `UserPayload`. No index signature on purpose: reading a
 * field the backend doesn't send must be a compile error, not `unknown`.
 * Extend both sides together when the payload is remodeled.
 */
export interface UserPayload {
  user_id: string;
  is_barber_profile_active: boolean;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface AuthResponse {
  access_token: string;
  token_type: string;
  user: UserPayload;
}

// Single source for the type, the yup schema and the <select> options.
export const SIGNUP_GENDERS = ['Male', 'Female', 'PreferNotToSay'] as const;
export type SignupGender = (typeof SIGNUP_GENDERS)[number];

export interface SignupRequest {
  full_name: string;
  email: string;
  password: string;
  date_of_birth: string; // ISO date "YYYY-MM-DD"
  gender: SignupGender;
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
}

export interface MessageResponse {
  message: string;
}