const API_BASE_URL = (import.meta.env.VITE_API_BASE_URL ?? "http://127.0.0.1:8081").replace(/\/$/, "");
const AUTH_STORAGE_KEY = "aeroledger-auth-session";
const AUTH_EVENT_NAME = "aeroledger-auth-updated";

export class ApiError extends Error {
  status: number;

  constructor(message: string, status: number) {
    super(message);
    this.name = "ApiError";
    this.status = status;
  }
}

type RequestOptions = Omit<RequestInit, "body"> & {
  body?: BodyInit | Record<string, unknown> | unknown[];
  token?: string;
};

export function apiBaseUrl() {
  return API_BASE_URL;
}

export async function apiRequest<T>(path: string, options: RequestOptions = {}): Promise<T> {
  const headers = new Headers(options.headers);
  headers.set("Accept", "application/json");

  const isFormData = typeof FormData !== "undefined" && options.body instanceof FormData;
  if (options.body !== undefined && !isFormData && !headers.has("Content-Type")) {
    headers.set("Content-Type", "application/json");
  }

  if (options.token) {
    headers.set("Authorization", `Bearer ${options.token}`);
  }

  // Exclude our custom `body`/`token` fields from the spread; `body` is set
  // explicitly below with a type the fetch API accepts.
  const { body: _body, token: _token, ...fetchInit } = options;

  const response = await fetch(`${API_BASE_URL}${path}`, {
    ...fetchInit,
    headers,
    body:
      options.body === undefined
        ? undefined
        : isFormData
          ? (options.body as BodyInit)
          : JSON.stringify(options.body),
  });

  const text = await response.text();
  let data: unknown = undefined;
  if (text) {
    try {
      data = JSON.parse(text) as unknown;
    } catch {
      data = text;
    }
  }

  if (!response.ok) {
    // Only a request that carried an auth token can have its session expire.
    const hadToken = Boolean(options.token);

    if (response.status === 401 && hadToken && typeof window !== "undefined") {
      window.localStorage.removeItem(AUTH_STORAGE_KEY);
      window.dispatchEvent(new Event(AUTH_EVENT_NAME));
    }

    const message =
      typeof data === "object" && data !== null && "error" in data && typeof data.error === "string"
        ? data.error
        : typeof data === "string"
          ? data
          : `Request failed with status ${response.status}`;

    // Rewrite to the "session expired" copy only for authenticated requests.
    // Unauthenticated requests (login/signup) should surface the backend error
    // body so wrong-password / invalid-credential messages read correctly.
    throw new ApiError(
      response.status === 401 && hadToken
        ? "Your session expired or is no longer valid. Sign in again and retry."
        : message,
      response.status,
    );
  }

  return data as T;
}
