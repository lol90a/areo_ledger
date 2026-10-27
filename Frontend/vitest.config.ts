import { defineConfig } from "vitest/config";

// Kept separate from vite.config.ts so the dev/build config stays untouched.
export default defineConfig({
  test: {
    // The cart persists through localStorage and the wallet client reads
    // window, so these tests need a DOM rather than bare Node.
    environment: "jsdom",
    setupFiles: ["src/test/setup.ts"],
    include: ["src/**/*.{test,spec}.{ts,tsx}"],
  },
});
