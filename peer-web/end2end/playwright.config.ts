import { defineConfig, devices } from "@playwright/test";

/**
 * Playwright configuration for peer-web E2E tests.
 * See https://playwright.dev/docs/test-configuration.
 */
export default defineConfig({
  testDir: "./tests",
  timeout: 15_000,
  expect: {
    timeout: 5_000,
  },
  fullyParallel: false,            // run sequentially — shared mock backend state
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1,                      // single worker — avoids port conflicts
  reporter: process.env.CI ? "github" : "html",

  use: {
    baseURL: "http://localhost:3000",
    actionTimeout: 5_000,
    trace: "on-first-retry",
    screenshot: "only-on-failure",
    video: "retain-on-failure",
  },

  globalSetup: require.resolve("./global-setup"),
  globalTeardown: require.resolve("./global-teardown"),

  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
    // Firefox and WebKit can be enabled later for full cross-browser coverage.
    // Keeping Chromium-only for fast iteration during development.
    // {
    //   name: "firefox",
    //   use: { ...devices["Desktop Firefox"] },
    // },
    // {
    //   name: "webkit",
    //   use: { ...devices["Desktop Safari"] },
    // },
  ],
});
