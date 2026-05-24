import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./tests/e2e",
  testIgnore: [
    "**/perf-*-review.spec.js",
    "**/visual-responsive-review.spec.js"
  ],
  timeout: 30_000,
  fullyParallel: false,
  use: {
    baseURL: "http://127.0.0.1:4173",
    trace: "on-first-retry"
  },
  webServer: [
    {
      name: "quantpilot-api",
      command: "cargo run --quiet --bin quantpilot",
      cwd: "..",
      url: "http://127.0.0.1:3000/api/health",
      env: {
        QUANTPILOT_BIND_ADDR: "127.0.0.1",
        QUANTPILOT_PORT: "3000",
        QUANTPILOT_DEV: "1"
      },
      reuseExistingServer: !process.env.CI,
      timeout: 180_000
    },
    {
      name: "vite",
      command: "npm run dev -- --host 127.0.0.1 --port 4173",
      port: 4173,
      reuseExistingServer: !process.env.CI,
      timeout: 120_000
    }
  ],
  projects: [
    {
      name: "msedge",
      use: {
        ...devices["Desktop Edge"],
        channel: "msedge"
      }
    }
  ]
});
