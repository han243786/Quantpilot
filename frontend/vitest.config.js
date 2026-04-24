import { defineConfig, configDefaults } from "vitest/config";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: "./src/test/setup.js",
    css: true,
    exclude: [...configDefaults.exclude, "tests/e2e/**"],
    restoreMocks: true,
    clearMocks: true
  }
});
