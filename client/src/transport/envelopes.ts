
export interface ApiSuccessResponse<T> {
  success: true;
  data: T;
  error?: never;
}

export interface ApiErrorPayload {
  code: string;
  message: string;
  user_id?: string;
}

export interface ApiErrorResponse {
  success: false;
  data?: never;
  error: ApiErrorPayload;
}

export type ApiResponse<T> = ApiSuccessResponse<T> | ApiErrorResponse;
