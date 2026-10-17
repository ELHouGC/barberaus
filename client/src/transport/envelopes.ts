// src/transport/envelopes.ts
//
// Wire contract with the Axum backend:
//   2xx            -> { success: true, data }
//   4xx / 5xx      -> RFC 9457 problem details (application/problem+json)
//
// The frontend keeps a single discriminated union (`ApiResponse<T>`) so views
// never deal with exceptions or raw HTTP: they check `res.success`.

/** One field-level validation error. */
export interface ProblemFieldError {
  /** JSON pointer to the offending field, e.g. "/email". */
  pointer: string;
  /** Stable error key for this field. */
  key: string;
}

/** RFC 9457 problem details as emitted by the backend. */
export interface ProblemDetails {
  type: string;
  title: string;
  status: number;
  detail?: string;
  instance?: string;
  /** Stable machine-readable code (e.g. INVALID_CREDENTIALS). Branch on this. */
  code: string;
  /** Translation key. Display this, not `title`/`detail`. */
  i18n_key: string;
  /** Present on validation failures. */
  errors?: ProblemFieldError[];
  /** Extension member: only present on specific errors (e.g. unverified email). */
  user_id?: string;
}

export interface ApiSuccessResponse<T> {
  success: true;
  data: T;
  error?: never;
}

/**
 * Failure branch. `error` is the backend problem document when the server
 * answered, or a client-side one (type "about:blank", status 0 for network
 * failures) when it did not. Same shape either way.
 */
export interface ApiErrorResponse {
  success: false;
  data?: never;
  error: ProblemDetails;
}

export type ApiResponse<T> = ApiSuccessResponse<T> | ApiErrorResponse;