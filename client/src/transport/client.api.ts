// src/transport/client.api.ts
import { fetch as tauriFetch } from '@tauri-apps/plugin-http';
import { isTauri } from '@tauri-apps/api/core';
import { useAuthStore } from '@/stores/auth';
import type {
  ApiErrorResponse,
  ApiResponse,
  ApiSuccessResponse,
  ProblemDetails,
} from './envelopes';

const getBaseUrl = (): string => {
  const configuredBaseUrl = import.meta.env.VITE_API_BASE_URL;
  if (configuredBaseUrl) {
    return configuredBaseUrl;
  }

  if (import.meta.env.DEV) {
    const isAndroid = isTauri() && /android/i.test(navigator.userAgent);
    return isAndroid ? 'http://192.168.100.94:8080/api/v1' : 'http://localhost:8080/api/v1';
  }
  return 'https://api.yourdomain.com/api/v1';
};

export const BASE_URL = getBaseUrl();

export interface RequestOptions extends Omit<RequestInit, 'body'> {
  body?: unknown;
}

// ---------------------------------------------------------------------------
// Error normalisation
// ---------------------------------------------------------------------------

// Client-side failures use the same shape as server problems so that callers
// have a single error type. Keys below must exist in the locale files.
function clientProblem(
  status: number,
  code: string,
  i18nKey: string,
  title: string,
  detail?: string,
): ProblemDetails {
  return {
    type: 'about:blank',
    title,
    status,
    code,
    i18n_key: i18nKey,
    ...(detail ? { detail } : {}),
  };
}

function failure(error: ProblemDetails): ApiErrorResponse {
  return { success: false, error };
}

function isProblemDetails(value: unknown): value is ProblemDetails {
  if (typeof value !== 'object' || value === null) return false;
  const v = value as Record<string, unknown>;
  return (
    typeof v.code === 'string' &&
    typeof v.i18n_key === 'string' &&
    typeof v.title === 'string' &&
    typeof v.status === 'number'
  );
}

async function readProblem(response: Response): Promise<ProblemDetails> {
  try {
    const body: unknown = await response.json();
    if (isProblemDetails(body)) return body;
  } catch {
    // Non-JSON body (proxy error page, framework default rejection...)
  }
  return clientProblem(
    response.status,
    'UNEXPECTED_RESPONSE',
    'errors.unexpected',
    response.statusText || 'Unexpected response',
  );
}

async function readSuccess<T>(response: Response): Promise<ApiResponse<T>> {
  try {
    const body: unknown = await response.json();
    if (typeof body === 'object' && body !== null && (body as { success?: unknown }).success === true) {
      return body as ApiSuccessResponse<T>;
    }
  } catch {
    // fall through
  }
  // 2xx without the success envelope: backend contract violation, fail loudly.
  return failure(
    clientProblem(response.status, 'INVALID_RESPONSE', 'errors.invalid_response', 'Invalid response'),
  );
}

// ---------------------------------------------------------------------------
// Single request (no retry logic). Never throws for HTTP/network failures.
// ---------------------------------------------------------------------------

async function send<T>(
  endpoint: string,
  { body, headers, ...rest }: RequestOptions,
): Promise<ApiResponse<T>> {
  const authStore = useAuthStore();
  const fetchFn = isTauri() ? tauriFetch : window.fetch;

  const requestHeaders: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(headers as Record<string, string>),
  };

  // Read the token on every call so a retry after refresh picks up the new one.
  if (authStore.accessToken) {
    requestHeaders['Authorization'] = `Bearer ${authStore.accessToken}`;
  }

  const init: RequestInit = {
    ...rest,
    credentials: 'include', // HttpOnly refresh cookie (Rust cookie jar under Tauri)
    headers: requestHeaders,
  };

  if (body !== undefined) {
    init.body = typeof body === 'string' ? body : JSON.stringify(body);
  }

  let response: Response;
  try {
    response = await fetchFn(`${BASE_URL}${endpoint}`, init);
  } catch (err) {
    return failure(
      clientProblem(
        0,
        'NETWORK_ERROR',
        'errors.network',
        'Network error',
        err instanceof Error ? err.message : undefined,
      ),
    );
  }

  if (!response.ok) return failure(await readProblem(response));

  if (response.status === 204) {
    // No body: callers of 204 endpoints must not read `data`.
    return { success: true, data: undefined as T };
  }

  return readSuccess<T>(response);
}

// ---------------------------------------------------------------------------
// Public client
// ---------------------------------------------------------------------------

export async function apiClient<T>(
  endpoint: string,
  options: RequestOptions = {},
): Promise<ApiResponse<T>> {
  const result = await send<T>(endpoint, options);

  // Auth endpoints answer 401 for business reasons (bad credentials, bad code):
  // never try to refresh for them.
  if (result.success || result.error.status !== 401 || endpoint.startsWith('/auth/')) {
    return result;
  }

  // Single-flight refresh lives in the auth store (it owns the session).
  const refreshed = await useAuthStore().refreshSession();
  if (!refreshed) return result; // surface the original 401 problem

  return send<T>(endpoint, options); // exactly one retry, no recursion
}

export const api = {
  get: <T>(endpoint: string, options?: RequestOptions) =>
    apiClient<T>(endpoint, { ...options, method: 'GET' }),

  post: <T>(endpoint: string, body?: unknown, options?: RequestOptions) =>
    apiClient<T>(endpoint, { ...options, method: 'POST', body }),

  put: <T>(endpoint: string, body?: unknown, options?: RequestOptions) =>
    apiClient<T>(endpoint, { ...options, method: 'PUT', body }),

  patch: <T>(endpoint: string, body?: unknown, options?: RequestOptions) =>
    apiClient<T>(endpoint, { ...options, method: 'PATCH', body }),

  delete: <T>(endpoint: string, options?: RequestOptions) =>
    apiClient<T>(endpoint, { ...options, method: 'DELETE' }),
};