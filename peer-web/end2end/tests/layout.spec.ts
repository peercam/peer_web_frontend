import { test, expect, Page } from "@playwright/test";
import { resetMockState } from "../helpers/mock-server";

/**
 * E2E coverage for the shared layout shell (Phase 5 of the layout-shell
 * implementation plan).
 *
 * Cases:
 *   1. /dashboard → mobile-footer Home link is `active`.
 *   2. /wallet    → no mobile-footer link is `active` (canonical Dashboard
 *                   preset omits Wallet — Open Question 1's accepted default).
 *   3. /profile/abc → mobile-footer Profile link is `active` (prefix match).
 *      The seeded routes don't include `/profile/abc`, so this exercises the
 *      404/profile-prefix branch — the page may render an error/404 surface
 *      but the shared `MobileFooter` should still resolve and reactively
 *      mark the Profile entry active.
 *   4. /chat → header `<h1>` text == "Chat".
 *   5. Selector smoke on /wallet — `aside.right-sidebar`, `aside.left-sidebar`,
 *      `header.site-header.header-wallet`, `footer.mobile-footer` all resolve
 *      in the DOM.
 *
 * The `mobile-footer` is `display:none` at desktop widths via responsive
 * SCSS, so footer assertions check attachment + class — the contract this
 * component owns — rather than visibility, which belongs to the SCSS layer.
 *
 * Seed credentials (see tests/mock_backend/src/seed.rs):
 *   - test@peer.com / TestPass123
 */

const VERIFIED_EMAIL = "test@peer.com";
const VERIFIED_PASSWORD = "TestPass123";

async function login(page: Page): Promise<void> {
  await page.goto("/login");
  await page.locator("#loginEmail").fill(VERIFIED_EMAIL);
  await page.locator("#loginPassword").fill(VERIFIED_PASSWORD);
  await page.getByRole("button", { name: /log in|sign in/i }).click();
  await page.waitForURL(/dashboard|chat|wallet|\/$/);
}

test.describe("Layout shell", () => {
  test.beforeEach(async () => {
    await resetMockState();
  });

  test("T1: /dashboard highlights the Home nav item", async ({ page }) => {
    await login(page);
    await page.goto("/dashboard");

    const home = page.locator('footer.mobile-footer a[href="/dashboard"]');
    await expect(home).toBeAttached();
    await expect(home).toHaveClass(/(?:^|\s)active(?:\s|$)/);
  });

  test("T2: /wallet has no active mobile-footer item (Dashboard preset)", async ({
    page,
  }) => {
    await login(page);
    await page.goto("/wallet");

    await expect(page.locator("footer.mobile-footer")).toBeAttached();
    await expect(page.locator("footer.mobile-footer a.active")).toHaveCount(0);
  });

  test("T3: /profile/abc highlights Profile (prefix match)", async ({ page }) => {
    await login(page);
    await page.goto("/profile/abc");

    const profile = page.locator('footer.mobile-footer a[href="/profile"]');
    await expect(profile).toBeAttached();
    await expect(profile).toHaveClass(/(?:^|\s)active(?:\s|$)/);
  });

  test("T4: /chat header h1 reads 'Chat'", async ({ page }) => {
    await login(page);
    await page.goto("/chat");

    const heading = page.locator("header.site_header h1");
    await expect(heading).toBeVisible();
    await expect(heading).toHaveText("Chat");
  });

  test("T5: /wallet shell selectors all resolve", async ({ page }) => {
    await login(page);
    await page.goto("/wallet");

    await expect(page.locator("aside.left-sidebar")).toBeAttached();
    await expect(page.locator("aside.right-sidebar")).toBeAttached();
    await expect(page.locator("header.site-header.header-wallet")).toBeVisible();
    await expect(page.locator("footer.mobile-footer")).toBeAttached();
  });
});
