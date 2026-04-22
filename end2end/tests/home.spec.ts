import { test, expect, Page } from "@playwright/test";
import { resetMockState } from "../helpers/mock-server";

/**
 * E2E coverage for the `/` (Home / Landing) route.
 *
 * Mirrors the legacy `index.php` behaviour: visiting `/` ends up on
 * `/dashboard` (signed in) or `/login?message=mustLogin` (signed out),
 * with any inbound `?redirect=…` preserved through to the login page.
 *
 * Seed credentials (see tests/mock_backend/src/seed.rs):
 *   - test@peer.com / TestPass123
 */

const VERIFIED_EMAIL = "test@peer.com";
const VERIFIED_PASSWORD = "TestPass123";

async function login(page: Page, email: string, password: string): Promise<void> {
  await page.goto("/login");
  await page.locator("#loginEmail").fill(email);
  await page.locator("#loginPassword").fill(password);
  await page.getByRole("button", { name: /log in|sign in/i }).click();
  await page.waitForURL(/dashboard|chat|wallet|\/$/);
}

test.describe("Home / Landing — `/`", () => {
  test.beforeEach(async () => {
    await resetMockState();
  });

  test("guest visiting / is redirected to /login?message=mustLogin", async ({ page }) => {
    await page.goto("/");
    await page.waitForURL(/\/login\?.*message=mustLogin/);
    expect(page.url()).toMatch(/\/login\?.*message=mustLogin/);
    expect(page.url()).not.toMatch(/redirect=/);
  });

  test("authed visitor at / is redirected to /dashboard", async ({ page }) => {
    await login(page, VERIFIED_EMAIL, VERIFIED_PASSWORD);
    await page.goto("/");
    await page.waitForURL(/\/dashboard$/);
    expect(page.url()).toMatch(/\/dashboard$/);
  });

  test("guest visiting /?redirect=%2Fwallet preserves redirect through to /login", async ({ page }) => {
    await page.goto("/?redirect=%2Fwallet");
    await page.waitForURL(/\/login\?.*redirect=(%2F|\/)wallet/);
    const url = page.url();
    expect(url).toMatch(/message=mustLogin/);
    expect(url).toMatch(/redirect=(%2F|\/)wallet/);
  });
});
