// src/services/client.api.ts
import { fetch as tauriFetch } from '@tauri-apps/plugin-http';
import { isTauri } from '@tauri-apps/api/core';
import { useAuthStore } from '@/stores/auth'; // Adjust import to your Pinia/Vue auth store

const getBaseUrl = (): string => {
  const configuredBaseUrl = import.meta.env.VITE_API_BASE_URL;
  if (configuredBaseUrl) {
    return configuredBaseUrl;
  }

  if (import.meta.env.DEV) {
    const isAndroid = isTauri() && /android/i.test(navigator.userAgent);
    return isAndroid ? 'http://192.168.100.94:8080/api/v1' : 'http://localhost:8080/api/v1';
  }
  return 'https://api.yourdomain.com';
};

export const BASE_URL = getBaseUrl();

export interface RequestOptions extends Omit<RequestInit, 'body'> {
  body?: any;
  _retry?: boolean; // Flag to prevent infinite refresh loops
}

// Single-flight refresh state management to prevent race conditions
let isRefreshing = false;
let failedQueue: Array<{
  resolve: (token: string) => void;
  reject: (reason?: any) => void;
}> = [];

const processQueue = (error: any, token: string | null = null) => {
  failedQueue.forEach((prom) => {
    if (error) {
      prom.reject(error);
    } else if (token) {
      prom.resolve(token);
    }
  });
  failedQueue = [];
};

export async function apiClient<T>(
  endpoint: string,
  options: RequestOptions = {}
): Promise<T> {
  const { body, headers, _retry, ...restOptions } = options;
  const authStore = useAuthStore(); // Access in-memory token state

  const fetchFn = isTauri() ? tauriFetch : window.fetch;

  // 1. REQUEST INTERCEPTOR: Attach in-memory access token
  const requestHeaders: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(headers as Record<string, string>),
  };

  if (authStore.accessToken) {
    requestHeaders['Authorization'] = `Bearer ${authStore.accessToken}`;
  }

  const config: RequestInit = {
    ...restOptions,
    credentials: 'include', // Automatically passes & saves HttpOnly cookies in Rust's cookie jar
    headers: requestHeaders,
  };

  if (body !== undefined) {
    config.body = typeof body === 'string' ? body : JSON.stringify(body);
  }

  try {
    const response = await fetchFn(`${BASE_URL}${endpoint}`, config);

    // 2. RESPONSE INTERCEPTOR: Handle 401 Unauthorized
    if (response.status === 401 && !_retry && !endpoint.includes('/auth/refresh')) {
      if (isRefreshing) {
        // Queue parallel requests until the active token refresh completes
        return new Promise<string>((resolve, reject) => {
          failedQueue.push({ resolve, reject });
        }).then(() => {
          options._retry = true;
          return apiClient<T>(endpoint, options);
        });
      }

      options._retry = true;
      isRefreshing = true;

      try {
        // Attempt token refresh call to Axum
        // (Axum will read the HttpOnly refresh_token cookie from Rust's Cookie Jar)
        const refreshResponse = await apiClient<{
          access_token: string;
          jti: string;
          user_id: string;
        }>('/api/v1/auth/refresh', {
          method: 'POST',
          _retry: true,
        });
       
        // Update in-memory auth store
        authStore.updateToken(refreshResponse.access_token);

        processQueue(null, refreshResponse.access_token);

        // Retry original failed request with the new token
        return apiClient<T>(endpoint, options);
      } catch (refreshError) {
        processQueue(refreshError, null);
        authStore.clearSession(); // Force logout on failed refresh
        throw refreshError;
      } finally {
        isRefreshing = false;
      }
    }

    if (!response.ok) {
      let errorMessage = `HTTP ${response.status}: ${response.statusText}`;
      try {
        const errorData = await response.json();
        if (errorData.message) errorMessage = errorData.message;
      } catch {
        // Non-JSON error payload
      }
      throw new Error(errorMessage);
    }

    if (response.status === 204) {
      return {} as T;
    }

    return response.json();
  } catch (error) {
    throw error;
  }
}

// 3. Convenience REST Helpers
export const api = {
  get: <T>(endpoint: string, options?: RequestOptions) =>
    apiClient<T>(endpoint, { ...options, method: 'GET' }),

  post: <T>(endpoint: string, body?: any, options?: RequestOptions) =>
    apiClient<T>(endpoint, { ...options, method: 'POST', body }),

  put: <T>(endpoint: string, body?: any, options?: RequestOptions) =>
    apiClient<T>(endpoint, { ...options, method: 'PUT', body }),

  patch: <T>(endpoint: string, body?: any, options?: RequestOptions) =>
    apiClient<T>(endpoint, { ...options, method: 'PATCH', body }),

  delete: <T>(endpoint: string, options?: RequestOptions) =>
    apiClient<T>(endpoint, { ...options, method: 'DELETE' }),
};


