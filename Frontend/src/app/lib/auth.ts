export type AuthSession = {
  id: string;
  email: string;
  name: string;
  role: string;
  token: string;
};

const AUTH_STORAGE_KEY = "aeroledger-auth-session";
const AUTH_EVENT_NAME = "aeroledger-auth-updated";

function isBrowser() {
  return typeof window !== "undefined";
}

function dispatchAuthUpdate() {
  if (isBrowser()) {
    window.dispatchEvent(new Event(AUTH_EVENT_NAME));
  }
}

export function authEventName() {
  return AUTH_EVENT_NAME;
}

/**
 * Decode a JWT payload without verifying its signature (client-side sanity
 * check only). Returns null for anything that is not a well-formed JWT.
 */
function decodeJwtPayload(token: string): Record<string, unknown> | null {
  const parts = token.split(".");
  if (parts.length !== 3) {
    return null;
  }

  try {
    let base64 = parts[1].replace(/-/g, "+").replace(/_/g, "/");
    // Restore base64 padding stripped by base64url encoding.
    while (base64.length % 4 !== 0) {
      base64 += "=";
    }
    const json = atob(base64);
    const payload = JSON.parse(json) as unknown;
    return typeof payload === "object" && payload !== null ? (payload as Record<string, unknown>) : null;
  } catch (_error) {
    return null;
  }
}

/** True when the token carries an `exp` claim that is already in the past. */
function isTokenExpired(token: string): boolean {
  const payload = decodeJwtPayload(token);
  if (!payload) {
    return false; // Not a decodable JWT — leave validity to the backend.
  }

  const exp = payload.exp;
  if (typeof exp !== "number") {
    return false; // No expiry claim — treat as non-expiring here.
  }

  const nowSeconds = Math.floor(Date.now() / 1000);
  return exp <= nowSeconds;
}

export function getAuthSession(): AuthSession | null {
  if (!isBrowser()) {
    return null;
  }

  const raw = window.localStorage.getItem(AUTH_STORAGE_KEY);
  if (!raw) {
    return null;
  }

  try {
    const parsed = JSON.parse(raw) as AuthSession;
    if (parsed?.token && parsed?.id && parsed?.email) {
      // Clear and signal an expired session so the app can react (e.g. redirect).
      if (isTokenExpired(parsed.token)) {
        window.localStorage.removeItem(AUTH_STORAGE_KEY);
        dispatchAuthUpdate();
        return null;
      }
      return parsed;
    }
  } catch (_error) {
    // Ignore malformed auth state.
  }

  window.localStorage.removeItem(AUTH_STORAGE_KEY);
  return null;
}

export function setAuthSession(session: AuthSession) {
  if (!isBrowser()) {
    return;
  }

  window.localStorage.setItem(AUTH_STORAGE_KEY, JSON.stringify(session));
  dispatchAuthUpdate();
}

export function clearAuthSession() {
  if (!isBrowser()) {
    return;
  }

  window.localStorage.removeItem(AUTH_STORAGE_KEY);
  dispatchAuthUpdate();
}

export function isAuthenticated() {
  return getAuthSession() !== null;
}
