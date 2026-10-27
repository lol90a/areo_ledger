/**
 * Test setup.
 *
 * Node 22 ships an experimental global `localStorage` that takes precedence over
 * the one jsdom installs, and it is a stub: it has no `clear()` and warns about
 * a missing `--localstorage-file`. Anything exercising the cart would fail on
 * the environment rather than on the code.
 *
 * Installing a plain in-memory Storage keeps the tests deterministic and
 * independent of which Node ships what. It is per-process, so tests that care
 * about isolation still clear it themselves.
 */
class MemoryStorage implements Storage {
  private store = new Map<string, string>();

  get length() {
    return this.store.size;
  }

  clear() {
    this.store.clear();
  }

  getItem(key: string) {
    return this.store.has(key) ? this.store.get(key)! : null;
  }

  key(index: number) {
    return Array.from(this.store.keys())[index] ?? null;
  }

  removeItem(key: string) {
    this.store.delete(key);
  }

  setItem(key: string, value: string) {
    this.store.set(key, String(value));
  }
}

const storage = new MemoryStorage();

for (const target of [globalThis, globalThis.window].filter(Boolean)) {
  Object.defineProperty(target, "localStorage", {
    value: storage,
    configurable: true,
    writable: true,
  });
}
