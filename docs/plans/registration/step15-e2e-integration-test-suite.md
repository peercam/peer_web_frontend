# Step 15 — End-to-End Integration Test Suite

> Part of the [Registration Leptos Migration Plan](registration-leptos-migration.md).

**Goal:** Create an automated E2E test suite using Playwright that exercises the full registration happy path and key error paths against the mock backend. The suite runs in CI on every PR and completes in under 30 seconds.

---

## 15.1 — Prerequisites

Before starting this step, confirm:

| Requirement | Verification | Expected |
|-------------|--------------|----------|
| Steps 0–14 complete | `cargo leptos build` | Compiles with no errors; registration flow is functional end-to-end |
| Mock backend starts | `cd packages/mock_backend && node server.js` | Prints "Mock Peer backend running at http://localhost:4000/graphql" |
| Leptos app starts | `cd peer-web && cargo leptos watch` | Serves at `http://localhost:3000` |
| Existing Playwright scaffold | `cat end2end/package.json` | `@playwright/test` is already a dev dependency |
| All browsers installed | `cd end2end && npx playwright install` | Chromium, Firefox, and WebKit binaries present |
| Visual parity confirmed (Step 14) | Manual inspection | Leptos `/register` page matches the PHP version |

---

## 15.2 — Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          E2E Test Run                                       │
│                                                                             │
│  ┌──────────────────────┐     ┌──────────────────────┐                     │
│  │  globalSetup.ts       │     │  globalTeardown.ts    │                    │
│  │                        │     │                        │                   │
│  │  1. Start mock backend │     │  1. Kill mock backend  │                   │
│  │     (port 4000)        │     │     (port 4000)        │                   │
│  │  2. Start Leptos app   │     │  2. Kill Leptos app    │                   │
│  │     (port 3000)        │     │     (port 3000)        │                   │
│  │  3. Wait for readiness │     │  3. Cleanup temp state  │                   │
│  └──────────┬─────────────┘     └──────────┬─────────────┘                  │
│             │                               │                                │
│             ▼                               ▲                                │
│  ┌──────────────────────────────────────────┴──────────────────────────────┐│
│  │  Playwright Test Runner                                                 ││
│  │                                                                          ││
│  │  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐      ││
│  │  │  registration/    │  │  registration/    │  │  registration/    │     ││
│  │  │  happy-path.      │  │  referral.        │  │  form-           │     ││
│  │  │  spec.ts          │  │  spec.ts          │  │  validation.     │     ││
│  │  │                    │  │                    │  │  spec.ts         │     ││
│  │  │  T1: Full happy   │  │  T2: Invalid fmt  │  │  T5: Weak pw     │     ││
│  │  │      path         │  │  T3: Server reject│  │  T6: Mismatch pw │     ││
│  │  │  T8: URL prefill  │  │                    │  │  T7: Unchecked   │     ││
│  │  │  T10: Auto-redir  │  │                    │  │      checkboxes  │     ││
│  │  └──────────────────┘  └──────────────────┘  └──────────────────┘      ││
│  │                                                                          ││
│  │  ┌──────────────────┐  ┌──────────────────┐                             ││
│  │  │  registration/    │  │  registration/    │                            ││
│  │  │  server-errors.   │  │  navigation.      │                            ││
│  │  │  spec.ts          │  │  spec.ts          │                            ││
│  │  │                    │  │                    │                            ││
│  │  │  T4: Duplicate    │  │  T9: Back button  │                            ││
│  │  │      email        │  │      preserves    │                            ││
│  │  │                    │  │      state        │                            ││
│  │  └──────────────────┘  └──────────────────┘                             ││
│  └──────────────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────────────┘
```

### Key Design Decisions

1. **Playwright over fantoccini** — the existing scaffold already uses Playwright, and it gives us cross-browser coverage, excellent selector APIs, built-in waiting, and first-class CI support
2. **Separate spec files per concern** — referral, form validation, server errors, navigation, and happy path are isolated so failures are easy to diagnose
3. **Shared helpers** — `MockServerManager` and page-object utilities avoid duplication across test files
4. **Global setup/teardown** — both the mock backend and the Leptos app are started once per test run, not per-test, keeping total execution under 30 seconds
5. **Mock state reset** — a REST endpoint (`POST /reset`) is added to the mock backend to clear registered emails and verified users between tests, ensuring isolation without full restart

---

## 15.3 — Mock Backend Enhancement: State Reset Endpoint

Before the E2E tests can run reliably in sequence, the mock backend needs an endpoint to reset its in-memory state between tests.

### 15.3.1 — Add reset endpoint to `packages/mock_backend/server.js`

Add this **before** the `app.all("/graphql", ...)` line:

```javascript
// Reset in-memory state — used by E2E tests between test cases
app.post("/reset", (req, res) => {
  state.registeredEmails.clear();
  state.verifiedUsers.clear();
  res.json({ status: "ok" });
});
```

### 15.3.2 — Verify

```bash
curl -X POST http://localhost:4000/reset
# → {"status":"ok"}
```

---

## 15.4 — File Structure

```
end2end/
├── playwright.config.ts              ← Updated: baseURL, globalSetup/Teardown, projects
├── package.json                      ← Updated: add test scripts
├── tsconfig.json                     ← Existing (no changes)
├── global-setup.ts                   ← NEW: starts mock backend + Leptos app
├── global-teardown.ts                ← NEW: kills both processes
├── helpers/
│   ├── mock-server.ts                ← NEW: MockServerManager utility
│   ├── registration-page.ts          ← NEW: page object for /register
│   └── wait-for-server.ts           ← NEW: TCP port readiness check
└── tests/
    ├── example.spec.ts               ← Existing (can be removed or kept)
    └── registration/
        ├── happy-path.spec.ts        ← NEW: T1, T8, T10
        ├── referral.spec.ts          ← NEW: T2, T3
        ├── form-validation.spec.ts   ← NEW: T5, T6, T7
        ├── server-errors.spec.ts     ← NEW: T4
        └── navigation.spec.ts        ← NEW: T9
```

---

## 15.5 — Implementation: Playwright Configuration Update

### 15.5.1 — Update `end2end/playwright.config.ts`

```typescript
import { defineConfig, devices } from "@playwright/test";

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
```

### Key changes from the existing config

| Setting | Before | After | Reason |
|---------|--------|-------|--------|
| `baseURL` | commented out | `http://localhost:3000` | All `page.goto("/register")` calls use relative paths |
| `fullyParallel` | `true` | `false` | Tests share mock backend state; reset happens between tests |
| `workers` | `undefined` | `1` | Single worker avoids port conflicts and race conditions |
| `globalSetup` | absent | `./global-setup` | Starts mock backend + Leptos app before all tests |
| `globalTeardown` | absent | `./global-teardown` | Kills both processes after all tests |
| `reporter` | `"html"` | CI-aware | `"github"` for CI annotations, `"html"` for local debugging |
| `screenshot` | absent | `"only-on-failure"` | Captures visual evidence when tests fail |
| `video` | absent | `"retain-on-failure"` | Records video for failed test diagnosis |

---

## 15.6 — Implementation: Update `package.json` Scripts

### 15.6.1 — Update `end2end/package.json`

```json
{
  "name": "end2end",
  "version": "1.0.0",
  "description": "E2E tests for peer-web Leptos registration flow",
  "scripts": {
    "test": "playwright test",
    "test:headed": "playwright test --headed",
    "test:debug": "playwright test --debug",
    "test:ui": "playwright test --ui",
    "test:report": "playwright show-report",
    "install:browsers": "playwright install --with-deps"
  },
  "keywords": [],
  "author": "",
  "license": "ISC",
  "devDependencies": {
    "@playwright/test": "^1.44.1",
    "@types/node": "^20.12.12",
    "typescript": "^5.4.5",
    "tree-kill": "^1.2.2"
  }
}
```

The `tree-kill` package is added to reliably kill process trees (mock backend + Leptos dev server) during teardown, including child processes.

---

## 15.7 — Implementation: Helpers

### 15.7.1 — `helpers/wait-for-server.ts`

A TCP-level readiness check that polls a port until it accepts connections:

```typescript
import net from "net";

/**
 * Wait until a TCP server is accepting connections on the given port.
 * Retries every `intervalMs` until `timeoutMs` is reached.
 */
export async function waitForServer(
  port: number,
  timeoutMs = 30_000,
  intervalMs = 500
): Promise<void> {
  const start = Date.now();

  while (Date.now() - start < timeoutMs) {
    const isUp = await new Promise<boolean>((resolve) => {
      const socket = net.createConnection({ port, host: "127.0.0.1" });
      socket.once("connect", () => {
        socket.destroy();
        resolve(true);
      });
      socket.once("error", () => {
        socket.destroy();
        resolve(false);
      });
    });

    if (isUp) return;
    await new Promise((r) => setTimeout(r, intervalMs));
  }

  throw new Error(`Server on port ${port} did not start within ${timeoutMs}ms`);
}
```

### 15.7.2 — `helpers/mock-server.ts`

Utility for resetting mock backend state via the new `/reset` endpoint:

```typescript
const MOCK_BACKEND_URL = "http://localhost:4000";

/**
 * Reset mock backend state (registered emails, verified users).
 * Call this in `beforeEach` to ensure test isolation.
 */
export async function resetMockState(): Promise<void> {
  const res = await fetch(`${MOCK_BACKEND_URL}/reset`, { method: "POST" });
  if (!res.ok) {
    throw new Error(`Failed to reset mock state: ${res.status} ${res.statusText}`);
  }
}

/**
 * Pre-register an email in the mock backend so subsequent registration
 * attempts with this email return a duplicate error (30601).
 *
 * Uses the GraphQL register mutation directly.
 */
export async function preRegisterEmail(email: string): Promise<void> {
  const query = `
    mutation {
      register(input: {
        email: "${email}",
        password: "TestPass123!",
        username: "preregistered_user",
        referralUuid: "85d5f836-b1f5-4c4e-9381-1b058e13df93"
      }) {
        status
        ResponseCode
      }
    }
  `;

  const res = await fetch(`${MOCK_BACKEND_URL}/graphql`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ query }),
  });

  if (!res.ok) {
    throw new Error(`Failed to pre-register email: ${res.status}`);
  }
}
```

### 15.7.3 — `helpers/registration-page.ts`

Page object model encapsulating all registration page interactions:

```typescript
import { Page, Locator, expect } from "@playwright/test";

/**
 * Page object for the /register page.
 *
 * Encapsulates selectors and common interactions for the multi-step
 * registration flow so test files stay readable and DRY.
 */
export class RegistrationPage {
  readonly page: Page;

  // ── Step 1: Referral ──────────────────────────────────
  readonly referralInput: Locator;
  readonly verifyCodeButton: Locator;
  readonly referralValidation: Locator;
  readonly referralStep: Locator;
  readonly useDefaultCodeLink: Locator;

  // ── Step 2: Registration Form ─────────────────────────
  readonly registrationStep: Locator;
  readonly emailInput: Locator;
  readonly usernameInput: Locator;
  readonly passwordInput: Locator;
  readonly confirmPasswordInput: Locator;
  readonly privacyCheckbox: Locator;
  readonly eulaCheckbox: Locator;
  readonly registerButton: Locator;

  // ── Validation displays ───────────────────────────────
  readonly emailValidation: Locator;
  readonly usernameValidation: Locator;
  readonly passwordValidation: Locator;
  readonly confirmPasswordValidation: Locator;
  readonly checkboxValidation: Locator;
  readonly passwordStrength: Locator;

  // ── Step 3: Success ───────────────────────────────────
  readonly successStep: Locator;
  readonly loginLink: Locator;

  // ── Shared ────────────────────────────────────────────
  readonly backButton: Locator;
  readonly toast: Locator;

  constructor(page: Page) {
    this.page = page;

    // Step 1
    this.referralInput = page.locator("#referralCode");
    this.verifyCodeButton = page.locator("#verifyReferralBtn");
    this.referralValidation = page.locator("#referralCodeValidation");
    this.referralStep = page.locator('[data-step="1"]');
    this.useDefaultCodeLink = page.locator("#useDefaultCodeBtn");

    // Step 2
    this.registrationStep = page.locator('[data-step="2"]');
    this.emailInput = page.locator("#email");
    this.usernameInput = page.locator("#username");
    this.passwordInput = page.locator("#password");
    this.confirmPasswordInput = page.locator("#confirmPassword");
    this.privacyCheckbox = page.locator("#readPrivacy");
    this.eulaCheckbox = page.locator("#agreementEULA");
    this.registerButton = page.locator("#registerBtn");

    // Validation
    this.emailValidation = page.locator("#emailValidation");
    this.usernameValidation = page.locator("#usernameValidation");
    this.passwordValidation = page.locator("#passwordValidation");
    this.confirmPasswordValidation = page.locator("#confirmPasswordValidation");
    this.checkboxValidation = page.locator("#checkboxValidation");
    this.passwordStrength = page.locator("#passwordStrength");

    // Step 3
    this.successStep = page.locator('[data-step="3"]');
    this.loginLink = this.successStep.locator("a");

    // Shared
    this.backButton = page.locator("#backBtn");
    this.toast = page.locator(".toast");
  }

  /** Navigate to the registration page. */
  async goto(params?: string) {
    const url = params ? `/register?${params}` : "/register";
    await this.page.goto(url);
  }

  /** Complete step 1 with a valid referral code. */
  async completeReferralStep(code = "85d5f836-b1f5-4c4e-9381-1b058e13df93") {
    await this.referralInput.fill(code);
    await this.verifyCodeButton.click();
    await this.registrationStep.waitFor({ state: "visible" });
  }

  /** Fill all step 2 fields with valid data. */
  async fillRegistrationForm(overrides?: {
    email?: string;
    username?: string;
    password?: string;
    confirmPassword?: string;
  }) {
    const email = overrides?.email ?? "newuser@example.com";
    const username = overrides?.username ?? "peer_tester";
    const password = overrides?.password ?? "SecurePass123!";
    const confirmPassword = overrides?.confirmPassword ?? password;

    await this.emailInput.fill(email);
    await this.usernameInput.fill(username);
    await this.passwordInput.fill(password);
    await this.confirmPasswordInput.fill(confirmPassword);
    await this.privacyCheckbox.check();
    await this.eulaCheckbox.check();
  }

  /** Complete the entire registration flow (step 1 + step 2 + submit). */
  async completeFullRegistration(overrides?: {
    referralCode?: string;
    email?: string;
    username?: string;
    password?: string;
  }) {
    await this.completeReferralStep(overrides?.referralCode);
    await this.fillRegistrationForm(overrides);
    await this.registerButton.click();
    await this.successStep.waitFor({ state: "visible" });
  }

  /** Assert that a toast with the given text (substring) is visible. */
  async expectToast(textSubstring: string) {
    await expect(this.toast).toBeVisible();
    await expect(this.toast).toContainText(textSubstring);
  }

  /** Assert which step is currently active/visible. */
  async expectActiveStep(step: 1 | 2 | 3) {
    const stepLocator = this.page.locator(`[data-step="${step}"]`);
    await expect(stepLocator).toBeVisible();
  }
}
```

> **Note on selectors:** The selectors above use IDs and `data-step` attributes matching the current `register.php` markup. When the Leptos components are built (steps 6–10), they must produce elements with these same IDs and attributes. If the Leptos markup uses different selectors, update this page object accordingly. The key principle is that the page object is the **single source of truth** for selectors — test files never contain raw selectors.

---

## 15.8 — Implementation: Global Setup & Teardown

### 15.8.1 — `global-setup.ts`

```typescript
import { FullConfig } from "@playwright/test";
import { spawn, ChildProcess } from "child_process";
import path from "path";
import fs from "fs";
import { waitForServer } from "./helpers/wait-for-server";

const PID_FILE = path.join(__dirname, ".test-pids.json");

interface ProcessPids {
  mockBackend: number;
  leptosApp: number;
}

async function globalSetup(config: FullConfig) {
  console.log("\n🔧 Starting mock backend...");

  const mockBackend = spawn("node", ["server.js"], {
    cwd: path.resolve(__dirname, "../../packages/mock_backend"),
    stdio: "pipe",
    detached: true,
  });

  mockBackend.stdout?.on("data", (data: Buffer) => {
    if (process.env.DEBUG) console.log(`[mock] ${data.toString().trim()}`);
  });
  mockBackend.stderr?.on("data", (data: Buffer) => {
    console.error(`[mock:err] ${data.toString().trim()}`);
  });

  await waitForServer(4000, 15_000);
  console.log("✅ Mock backend ready on :4000");

  console.log("🔧 Starting Leptos app...");

  const leptosApp = spawn("cargo", ["leptos", "serve", "--release"], {
    cwd: path.resolve(__dirname, "../"),
    stdio: "pipe",
    detached: true,
    env: {
      ...process.env,
      GRAPHQL_ENDPOINT: "http://localhost:4000/graphql",
    },
  });

  leptosApp.stdout?.on("data", (data: Buffer) => {
    if (process.env.DEBUG) console.log(`[leptos] ${data.toString().trim()}`);
  });
  leptosApp.stderr?.on("data", (data: Buffer) => {
    // cargo leptos logs to stderr
    if (process.env.DEBUG) console.log(`[leptos] ${data.toString().trim()}`);
  });

  await waitForServer(3000, 120_000); // Leptos compile can take a while
  console.log("✅ Leptos app ready on :3000");

  // Save PIDs for teardown
  const pids: ProcessPids = {
    mockBackend: mockBackend.pid!,
    leptosApp: leptosApp.pid!,
  };
  fs.writeFileSync(PID_FILE, JSON.stringify(pids));

  // Unref so this process can exit while children keep running
  mockBackend.unref();
  leptosApp.unref();
}

export default globalSetup;
```

### 15.8.2 — `global-teardown.ts`

```typescript
import { FullConfig } from "@playwright/test";
import path from "path";
import fs from "fs";
import kill from "tree-kill";

const PID_FILE = path.join(__dirname, ".test-pids.json");

function killProcess(pid: number): Promise<void> {
  return new Promise((resolve) => {
    kill(pid, "SIGTERM", (err) => {
      if (err) {
        // Process may already be dead — that's fine
        console.warn(`⚠️  Could not kill PID ${pid}: ${err.message}`);
      }
      resolve();
    });
  });
}

async function globalTeardown(config: FullConfig) {
  if (!fs.existsSync(PID_FILE)) {
    console.warn("⚠️  PID file not found — processes may already be stopped");
    return;
  }

  const pids = JSON.parse(fs.readFileSync(PID_FILE, "utf-8"));

  console.log("\n🧹 Stopping Leptos app...");
  await killProcess(pids.leptosApp);

  console.log("🧹 Stopping mock backend...");
  await killProcess(pids.mockBackend);

  fs.unlinkSync(PID_FILE);
  console.log("✅ All processes stopped");
}

export default globalTeardown;
```

### 15.8.3 — Add `.test-pids.json` to `.gitignore`

Append to `end2end/.gitignore`:

```
.test-pids.json
test-results/
playwright-report/
```

---

## 15.9 — Implementation: Test Specs

### 15.9.1 — `tests/registration/happy-path.spec.ts` (T1, T8, T10)

```typescript
import { test, expect } from "@playwright/test";
import { RegistrationPage } from "../../helpers/registration-page";
import { resetMockState } from "../../helpers/mock-server";

test.describe("Registration — Happy Path", () => {
  let regPage: RegistrationPage;

  test.beforeEach(async ({ page }) => {
    await resetMockState();
    regPage = new RegistrationPage(page);
  });

  // ── T1: Full happy path ──────────────────────────────────────────────────
  test("T1: valid referral → valid form → success step visible", async ({ page }) => {
    await regPage.goto();

    // Step 1: Enter valid referral code
    await regPage.completeReferralStep();

    // Step 2: Fill registration form
    await regPage.fillRegistrationForm({
      email: "happypath@example.com",
      username: "happy_user",
      password: "StrongPass99!",
    });

    // Submit
    await regPage.registerButton.click();

    // Step 3: Success screen
    await regPage.expectActiveStep(3);
    await expect(regPage.successStep).toContainText("Welcome to peer!");

    // Session storage should have the email
    const storedEmail = await page.evaluate(() =>
      sessionStorage.getItem("newUserEmail")
    );
    expect(storedEmail).toBe("happypath@example.com");

    // "Go to Login" link should be visible and correct
    await expect(regPage.loginLink).toBeVisible();
    await expect(regPage.loginLink).toHaveAttribute("href", /login/);
  });

  // ── T8: URL parameter prefill ────────────────────────────────────────────
  test("T8: ?ref= query parameter auto-fills the referral input", async () => {
    const testCode = "85d5f836-b1f5-4c4e-9381-1b058e13df93";
    await regPage.goto(`ref=${testCode}`);

    await expect(regPage.referralInput).toHaveValue(testCode);
  });

  // ── T10: Auto-redirect if already logged in ──────────────────────────────
  test("T10: logged-in user visiting /register is redirected to /dashboard", async ({ page }) => {
    // Simulate being logged in by setting the auth token/cookie
    // that the Leptos app checks. The exact mechanism depends on
    // how Steps 1–14 implemented auth detection.
    //
    // Option A: Set a cookie before navigation
    await page.context().addCookies([
      {
        name: "peer_session",
        value: "mock-valid-session-token",
        domain: "localhost",
        path: "/",
      },
    ]);

    await page.goto("/register");

    // Should redirect to dashboard
    await page.waitForURL("**/dashboard**");
    expect(page.url()).toContain("/dashboard");
  });
});
```

### 15.9.2 — `tests/registration/referral.spec.ts` (T2, T3)

```typescript
import { test, expect } from "@playwright/test";
import { RegistrationPage } from "../../helpers/registration-page";
import { resetMockState } from "../../helpers/mock-server";

test.describe("Registration — Referral Code Validation", () => {
  let regPage: RegistrationPage;

  test.beforeEach(async ({ page }) => {
    await resetMockState();
    regPage = new RegistrationPage(page);
    await regPage.goto();
  });

  // ── T2: Invalid referral code format ─────────────────────────────────────
  test("T2: malformed referral code shows client-side error", async () => {
    await regPage.referralInput.fill("abc123");

    // Validation message should show error
    await expect(regPage.referralValidation).toBeVisible();
    await expect(regPage.referralValidation).toContainText(
      /referral code doesn't seem to work|invalid/i
    );

    // Should remain on step 1
    await regPage.expectActiveStep(1);
  });

  test("T2b: empty referral code prevents submission", async () => {
    await regPage.verifyCodeButton.click();

    // Should remain on step 1 with a validation message
    await regPage.expectActiveStep(1);
    await expect(regPage.referralValidation).toBeVisible();
  });

  // ── T3: Unknown referral code (server rejects) ──────────────────────────
  test("T3: valid-format but unknown referral code shows error toast", async () => {
    // This UUID has valid format but is not in the mock backend's known set
    const unknownCode = "00000000-0000-0000-0000-000000000000";
    await regPage.referralInput.fill(unknownCode);
    await regPage.verifyCodeButton.click();

    // Error toast should appear
    await regPage.expectToast(/invalid|referral/i);

    // Should remain on step 1
    await regPage.expectActiveStep(1);
  });

  test("T3b: valid known referral code advances to step 2", async () => {
    await regPage.completeReferralStep();
    await regPage.expectActiveStep(2);
  });
});
```

### 15.9.3 — `tests/registration/form-validation.spec.ts` (T5, T6, T7)

```typescript
import { test, expect } from "@playwright/test";
import { RegistrationPage } from "../../helpers/registration-page";
import { resetMockState } from "../../helpers/mock-server";

test.describe("Registration — Form Validation", () => {
  let regPage: RegistrationPage;

  test.beforeEach(async ({ page }) => {
    await resetMockState();
    regPage = new RegistrationPage(page);
    await regPage.goto();
    // Advance to step 2
    await regPage.completeReferralStep();
  });

  // ── T5: Weak password rejected ──────────────────────────────────────────
  test("T5: weak password shows strength indicator and requirements", async () => {
    await regPage.passwordInput.fill("abcd");

    // Password strength meter should be visible and show weak state
    await expect(regPage.passwordStrength).toBeVisible();

    // Requirements should indicate what's missing
    const strengthText = await regPage.passwordStrength.textContent();
    expect(strengthText).toMatch(/weak|very weak/i);

    // Fill rest of form and try to submit — should be blocked
    await regPage.emailInput.fill("test@example.com");
    await regPage.usernameInput.fill("test_user");
    await regPage.confirmPasswordInput.fill("abcd");
    await regPage.privacyCheckbox.check();
    await regPage.eulaCheckbox.check();
    await regPage.registerButton.click();

    // Should remain on step 2 — password too weak
    await regPage.expectActiveStep(2);
    await expect(regPage.passwordValidation).toBeVisible();
  });

  test("T5b: strong password shows strong/excellent indicator", async () => {
    await regPage.passwordInput.fill("S3cur3P@ssw0rd!");

    await expect(regPage.passwordStrength).toBeVisible();
    const strengthText = await regPage.passwordStrength.textContent();
    expect(strengthText).toMatch(/good|excellent|strong/i);
  });

  // ── T6: Mismatched confirm password ─────────────────────────────────────
  test("T6: mismatched confirm password shows error", async () => {
    await regPage.passwordInput.fill("SecurePass123!");
    await regPage.confirmPasswordInput.fill("DifferentPass456!");

    // Trigger validation by tabbing away or clicking submit
    await regPage.registerButton.click();

    await expect(regPage.confirmPasswordValidation).toBeVisible();
    await expect(regPage.confirmPasswordValidation).toContainText(
      /passwords do not match/i
    );

    // Should remain on step 2
    await regPage.expectActiveStep(2);
  });

  // ── T7: Unchecked checkboxes ────────────────────────────────────────────
  test("T7: unchecked checkboxes show error message", async () => {
    await regPage.fillRegistrationForm();

    // Uncheck the checkboxes that fillRegistrationForm checked
    await regPage.privacyCheckbox.uncheck();
    await regPage.eulaCheckbox.uncheck();

    await regPage.registerButton.click();

    await expect(regPage.checkboxValidation).toBeVisible();
    await expect(regPage.checkboxValidation).toContainText(
      /privacy policy|eula|accept both/i
    );

    await regPage.expectActiveStep(2);
  });

  test("T7b: single unchecked checkbox still shows error", async () => {
    await regPage.fillRegistrationForm();
    await regPage.eulaCheckbox.uncheck(); // only uncheck EULA

    await regPage.registerButton.click();

    await expect(regPage.checkboxValidation).toBeVisible();
    await regPage.expectActiveStep(2);
  });
});
```

### 15.9.4 — `tests/registration/server-errors.spec.ts` (T4)

```typescript
import { test, expect } from "@playwright/test";
import { RegistrationPage } from "../../helpers/registration-page";
import { resetMockState, preRegisterEmail } from "../../helpers/mock-server";

test.describe("Registration — Server Error Handling", () => {
  let regPage: RegistrationPage;

  test.beforeEach(async ({ page }) => {
    await resetMockState();
    regPage = new RegistrationPage(page);
    await regPage.goto();
    await regPage.completeReferralStep();
  });

  // ── T4: Duplicate email registration ────────────────────────────────────
  test("T4: registering with an already-used email shows backend error", async () => {
    const duplicateEmail = "taken@example.com";

    // Pre-register this email in the mock backend
    await preRegisterEmail(duplicateEmail);

    // Try to register with the same email
    await regPage.fillRegistrationForm({ email: duplicateEmail });
    await regPage.registerButton.click();

    // Should show error on the email field
    await expect(regPage.emailValidation).toBeVisible();
    await expect(regPage.emailValidation).toContainText(
      /already registered|duplicate/i
    );

    // Toast should also appear with the error
    await regPage.expectToast(/already registered/i);

    // Should remain on step 2
    await regPage.expectActiveStep(2);
  });

  test("T4b: 'fail@' email triggers server error toast", async () => {
    // The mock backend returns a 40601 error for fail@ emails
    await regPage.fillRegistrationForm({ email: "fail@example.com" });
    await regPage.registerButton.click();

    await regPage.expectToast(/couldn't complete|try again|went wrong/i);
    await regPage.expectActiveStep(2);
  });
});
```

### 15.9.5 — `tests/registration/navigation.spec.ts` (T9)

```typescript
import { test, expect } from "@playwright/test";
import { RegistrationPage } from "../../helpers/registration-page";
import { resetMockState } from "../../helpers/mock-server";

test.describe("Registration — Navigation & Back Button", () => {
  let regPage: RegistrationPage;

  test.beforeEach(async ({ page }) => {
    await resetMockState();
    regPage = new RegistrationPage(page);
    await regPage.goto();
  });

  // ── T9: Back navigation preserves state ─────────────────────────────────
  test("T9: going back from step 2 to step 1 preserves the referral code", async () => {
    const referralCode = "85d5f836-b1f5-4c4e-9381-1b058e13df93";

    // Complete step 1
    await regPage.completeReferralStep(referralCode);
    await regPage.expectActiveStep(2);

    // Click back button
    await regPage.backButton.click();

    // Should return to step 1
    await regPage.expectActiveStep(1);

    // Referral code should still be in the input
    await expect(regPage.referralInput).toHaveValue(referralCode);
  });

  test("T9b: back button on step 1 navigates to login", async () => {
    await regPage.expectActiveStep(1);
    await regPage.backButton.click();

    // Should navigate to login page
    await regPage.page.waitForURL(/login/);
  });

  test("T9c: back button is hidden on step 3 (success)", async ({ page }) => {
    await regPage.completeFullRegistration({
      email: "navtest@example.com",
      username: "nav_tester",
    });

    await regPage.expectActiveStep(3);

    // Back button should not be visible
    await expect(regPage.backButton).not.toBeVisible();
  });

  test("T9d: step 2 form data is preserved after back+forward navigation", async () => {
    // Complete step 1
    await regPage.completeReferralStep();

    // Partially fill step 2
    await regPage.emailInput.fill("partial@example.com");
    await regPage.usernameInput.fill("partial_user");

    // Go back to step 1
    await regPage.backButton.click();
    await regPage.expectActiveStep(1);

    // Go forward to step 2 again
    await regPage.verifyCodeButton.click();
    await regPage.expectActiveStep(2);

    // Form data should be preserved
    await expect(regPage.emailInput).toHaveValue("partial@example.com");
    await expect(regPage.usernameInput).toHaveValue("partial_user");
  });
});
```

---

## 15.10 — Test Case Mapping

| # | Test ID | Test Description | Spec File | Assertions |
|---|---------|------------------|-----------|------------|
| 1 | T1 | Happy path: valid referral → valid form → success | `happy-path.spec.ts` | Step 3 visible, success message text, sessionStorage has email, login link present |
| 2 | T2 | Invalid referral code format | `referral.spec.ts` | Client-side validation message visible, remains on step 1 |
| 3 | T3 | Unknown referral code (server rejects) | `referral.spec.ts` | Error toast displayed, remains on step 1 |
| 4 | T4 | Duplicate email registration | `server-errors.spec.ts` | Email field shows backend error, toast displayed, remains on step 2 |
| 5 | T5 | Weak password rejected | `form-validation.spec.ts` | Password strength indicator shows weak, requirements visible, submission blocked |
| 6 | T6 | Mismatched confirm password | `form-validation.spec.ts` | "Passwords do not match" message visible, remains on step 2 |
| 7 | T7 | Unchecked checkboxes | `form-validation.spec.ts` | Checkbox error message visible, remains on step 2 |
| 8 | T8 | URL param prefill (`?ref=...`) | `happy-path.spec.ts` | Referral input auto-filled with the query parameter value |
| 9 | T9 | Back navigation preserves state | `navigation.spec.ts` | Return to step 1 keeps referral code, back on step 1 goes to login, hidden on step 3 |
| 10 | T10 | Auto-redirect if already logged in | `happy-path.spec.ts` | URL changes to `/dashboard` |

---

## 15.11 — Running the Tests

### 15.11.1 — Local Development

```bash
# From the end2end directory
cd end2end

# Install dependencies (first time only)
npm install
npx playwright install

# Run all tests (starts servers via globalSetup)
npm test

# Run in headed mode (see the browser)
npm run test:headed

# Run with Playwright UI (interactive debugging)
npm run test:ui

# Run a single test file
npx playwright test tests/registration/happy-path.spec.ts

# Run a single test by title
npx playwright test -g "T1"

# View the last HTML report
npm run test:report
```

### 15.11.2 — Manual Server Mode

If you prefer to manage the servers yourself (faster iteration during development):

```bash
# Terminal 1: Start mock backend
cd packages/mock_backend && node server.js

# Terminal 2: Start Leptos app
cd peer-web && GRAPHQL_ENDPOINT=http://localhost:4000/graphql cargo leptos serve

# Terminal 3: Run tests (skip globalSetup/Teardown)
cd end2end
SKIP_GLOBAL_SETUP=true npx playwright test
```

To support this, add an early-return guard in `global-setup.ts`:

```typescript
if (process.env.SKIP_GLOBAL_SETUP) {
  console.log("⏭️  Skipping global setup (SKIP_GLOBAL_SETUP=true)");
  return;
}
```

And the corresponding guard in `global-teardown.ts`:

```typescript
if (process.env.SKIP_GLOBAL_SETUP) {
  console.log("⏭️  Skipping global teardown (SKIP_GLOBAL_SETUP=true)");
  return;
}
```

### 15.11.3 — Debug Mode

```bash
# Run with Playwright inspector (step through tests)
npm run test:debug

# Run with browser visible + slow motion
npx playwright test --headed --slow-mo=500

# Enable verbose server logs
DEBUG=true npm test
```

---

## 15.12 — CI Pipeline Integration

### 15.12.1 — GitHub Actions Workflow

Create `.github/workflows/e2e.yml`:

```yaml
name: E2E Tests

on:
  pull_request:
    paths:
      - "/**"
      - "packages/mock_backend/**"
  push:
    branches: [main]

jobs:
  e2e:
    runs-on: ubuntu-latest
    timeout-minutes: 30

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Install cargo-leptos
        run: cargo install cargo-leptos

      - name: Cache Rust build
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: peer-web

      - name: Install Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 20

      - name: Install mock backend dependencies
        run: cd packages/mock_backend && npm ci

      - name: Install E2E dependencies
        run: cd end2end && npm ci

      - name: Install Playwright browsers
        run: cd end2end && npx playwright install --with-deps chromium

      - name: Build Leptos app
        run: cd peer-web && cargo leptos build --release
        env:
          GRAPHQL_ENDPOINT: http://localhost:4000/graphql

      - name: Run E2E tests
        run: cd end2end && npm test
        env:
          CI: true

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: playwright-report
          path: end2end/playwright-report/
          retention-days: 7

      - name: Upload failure screenshots
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: test-failures
          path: end2end/test-results/
          retention-days: 7
```

### 15.12.2 — CI-Specific Considerations

| Concern | Solution |
|---------|----------|
| Leptos compile time | Rust cache (`Swatinem/rust-cache`) + `--release` build before tests (not during) |
| Browser install time | Only install Chromium (`--with-deps chromium`) — skip Firefox/WebKit for CI speed |
| Flaky tests | `retries: 2` in CI mode (configured in `playwright.config.ts`) |
| Test timeout | 30-minute job timeout; 15-second per-test timeout |
| Failure artifacts | Screenshots and HTML report uploaded as artifacts for debugging |
| Port conflicts | Single worker, sequential execution |

---

## 15.13 — SSR Verification

The E2E tests must also verify that the Leptos app serves server-side rendered HTML (not a blank WASM shell):

### 15.13.1 — Add SSR check to `happy-path.spec.ts`

```typescript
test("SSR: register page HTML contains form content before hydration", async ({
  request,
}) => {
  // Fetch raw HTML (no JavaScript execution)
  const response = await request.get("/register");
  const html = await response.text();

  // The SSR HTML should contain key registration content
  expect(html).toContain("Welcome to");
  expect(html).toContain("referral code");
  expect(html).toContain('data-step="1"');

  // Form elements should be in the initial HTML
  expect(html).toContain("referralCode");
  expect(html).toContain("Verify Code");
});
```

---

## 15.14 — Accessibility Smoke Test

Integrate `@axe-core/playwright` for a basic accessibility check within the E2E suite:

### 15.14.1 — Install dependency

Add to `package.json` devDependencies:

```json
"@axe-core/playwright": "^4.9.0"
```

### 15.14.2 — Add accessibility test to `happy-path.spec.ts`

```typescript
import AxeBuilder from "@axe-core/playwright";

test("A11y: registration page has no critical accessibility violations", async ({
  page,
}) => {
  await page.goto("/register");

  const results = await new AxeBuilder({ page })
    .withTags(["wcag2a", "wcag2aa"])
    .exclude(".phone") // decorative phone mockup
    .analyze();

  expect(results.violations.filter((v) => v.impact === "critical")).toHaveLength(0);
  expect(results.violations.filter((v) => v.impact === "serious")).toHaveLength(0);
});
```

---

## 15.15 — File Changes Summary

| File | Action | Description |
|------|--------|-------------|
| `packages/mock_backend/server.js` | **Edit** | Add `POST /reset` endpoint for test isolation |
| `end2end/playwright.config.ts` | **Edit** | Update with `baseURL`, `globalSetup`, `globalTeardown`, single worker |
| `end2end/package.json` | **Edit** | Add test scripts, `tree-kill` and `@axe-core/playwright` dependencies |
| `end2end/.gitignore` | **Edit** | Add `.test-pids.json`, `test-results/`, `playwright-report/` |
| `end2end/global-setup.ts` | **Create** | Starts mock backend + Leptos app, saves PIDs |
| `end2end/global-teardown.ts` | **Create** | Kills both processes using saved PIDs |
| `end2end/helpers/wait-for-server.ts` | **Create** | TCP port readiness check utility |
| `end2end/helpers/mock-server.ts` | **Create** | `resetMockState()` and `preRegisterEmail()` utilities |
| `end2end/helpers/registration-page.ts` | **Create** | Page object model for `/register` |
| `end2end/tests/registration/happy-path.spec.ts` | **Create** | T1 (full happy path), T8 (URL prefill), T10 (auth redirect), SSR check, a11y check |
| `end2end/tests/registration/referral.spec.ts` | **Create** | T2 (invalid format), T3 (server rejection) |
| `end2end/tests/registration/form-validation.spec.ts` | **Create** | T5 (weak password), T6 (mismatch), T7 (unchecked checkboxes) |
| `end2end/tests/registration/server-errors.spec.ts` | **Create** | T4 (duplicate email), T4b (server error) |
| `end2end/tests/registration/navigation.spec.ts` | **Create** | T9 (back button preserves state), T9b/c/d (navigation edge cases) |
| `.github/workflows/e2e.yml` | **Create** | GitHub Actions CI pipeline |

---

## 15.16 — Edge Cases & Error Handling

| Edge Case | Handling |
|-----------|----------|
| Leptos app slow to compile on CI | `waitForServer(3000, 120_000)` — 2-minute timeout for the first build |
| Mock backend port already in use | `globalSetup` will fail with `EADDRINUSE` — user must kill the existing process |
| Test order dependence | `resetMockState()` in `beforeEach` ensures each test starts with clean state |
| Flaky network timing | Playwright's built-in auto-waiting + `waitFor({ state: "visible" })` avoids sleep-based waits |
| Toast auto-dismiss before assertion | `expect(toast).toBeVisible()` auto-retries for up to 5 seconds; toast is visible for 3 seconds — sufficient overlap |
| Hydration delay (WASM download) | Page object `waitFor` calls wait for elements to be interactive, not just present in SSR HTML |
| `globalTeardown` crash / skip | `tree-kill` with `SIGTERM` handles child processes; orphan processes on the test ports will cause the next run's `globalSetup` to fail clearly |
| macOS vs Linux path differences | All paths use `path.resolve()` / `path.join()` — no hardcoded separators |

---

## 15.17 — Performance Budget

| Metric | Target | Enforcement |
|--------|--------|-------------|
| Total test run time | < 30 seconds (excluding initial Leptos compile) | `timeout: 15_000` per test, 10 core tests + 6 supplementary |
| Individual test time | < 10 seconds each | Playwright `timeout` setting |
| CI total time (with compile) | < 15 minutes | GitHub Actions `timeout-minutes: 30` (generous buffer) |
| Failure artifact size | < 50 MB | Screenshots only on failure; video only on failure |

---

## 15.18 — Manual Verification Checklist

Before considering step 15 complete, verify:

| # | Check | Command / Action | Expected |
|---|-------|------------------|----------|
| 1 | All 10 core tests pass locally | `cd end2end && npm test` | 10/10 green (plus supplementary tests) |
| 2 | Tests pass in headless mode | `npm test` (default) | Same result as headed |
| 3 | HTML report generates | `npm run test:report` | Opens browser with detailed pass/fail report |
| 4 | Failure screenshots captured | Intentionally break a test, run `npm test` | `test-results/` contains a `.png` screenshot |
| 5 | Mock state resets between tests | Run full suite twice in a row | All pass both times (no stale state) |
| 6 | `SKIP_GLOBAL_SETUP` mode works | Start servers manually, then `SKIP_GLOBAL_SETUP=true npx playwright test` | Tests run against manually-started servers |
| 7 | SSR test passes | `npx playwright test -g "SSR"` | Raw HTML contains form elements |
| 8 | A11y test passes | `npx playwright test -g "A11y"` | Zero critical/serious axe-core violations |
| 9 | CI pipeline succeeds | Push to a PR branch | GitHub Actions job passes |
| 10 | Teardown cleans up | `npm test` then `lsof -i :3000 -i :4000` | No lingering processes on either port |

---

## 15.19 — Dependency Graph

```
Step 0 (Mock Backend)
    │
    ├──► 15.3 — Reset endpoint added to mock backend
    │
Steps 1–14 (Complete Leptos Registration Flow)
    │
    ├──► 15.5 — Playwright config references running Leptos app
    ├──► 15.7.3 — Page object selectors match Leptos-rendered HTML
    ├──► 15.9 — All tests exercise the real Leptos UI
    │
Step 12 (Toast Component)
    │
    ├──► T3, T4, T4b — Assert toast messages from response codes
    │
Step 13 (Accessibility)
    │
    ├──► 15.14 — axe-core smoke test validates WCAG compliance
    │
Step 14 (CSS & Visual Parity)
    │
    └──► All visual assertions (element visibility, step transitions)
```

---

## 15.20 — Future Extensibility

| Extension | Approach |
|-----------|----------|
| Cross-browser testing | Uncomment Firefox and WebKit projects in `playwright.config.ts` |
| Visual regression testing | Add `@playwright/test`'s `toHaveScreenshot()` assertions or integrate Percy/BackstopJS |
| Mobile viewport testing | Add projects with `devices["iPhone 13"]` and `devices["iPad"]` |
| Login flow E2E tests | Reuse `helpers/mock-server.ts` and create a `LoginPage` page object alongside `RegistrationPage` |
| Performance testing | Add `page.evaluate(() => performance.getEntriesByType("navigation"))` assertions for load times |
| API-level integration tests | Add a `tests/api/` directory with direct server-function tests (no browser) using Playwright's `request` API |
| Parallel test execution | Once tests are fully isolated (per-test mock backend instances), set `workers: undefined` for parallel runs |
