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
