import { test, expect, Page } from "@playwright/test";
import { resetMockState, getResetTokenForEmail } from "../helpers/mock-server";

/**
 * E2E coverage for the forgot-password flow (Task 5 of
 * `docs/plans/forgot-password/forgot-password-completion-sprint.md`).
 *
 * Cases:
 *   T1 — Page renders Step 1 email form.
 *   T2 — Invalid email is rejected client-side.
 *   T3 — Valid email advances to Step 2 (verify code).
 *   T4 — Verify code → Step 3 (new password) using debug-issued token.
 *   T5 — Mismatched passwords are rejected.
 *   T6 — Successful reset advances to Step 4 (success).
 *   T7 — Back button returns from Step 2 to Step 1 (callback path).
 *   T8 — Already-authenticated user is redirected to /dashboard.
 *
 * Seed credentials (see packages/mock_backend/src/seed.rs):
 *   email: test@peer.com  password: TestPass123
 */

const VERIFIED_EMAIL = "test@peer.com";
const VERIFIED_PASSWORD = "TestPass123";
const NEW_PASSWORD = "NewE2EPass456";

async function login(page: Page): Promise<void> {
  await page.goto("/login");
  await page.locator("#loginEmail").fill(VERIFIED_EMAIL);
  await page.locator("#loginPassword").fill(VERIFIED_PASSWORD);
  await page.getByRole("button", { name: /log in|sign in/i }).click();
  await page.waitForURL(/dashboard|chat|wallet|\/$/);
}

test.describe("Forgot Password", () => {
  test.beforeEach(async () => {
    await resetMockState();
  });

  test("T1: Step 1 email form renders", async ({ page }) => {
    await page.goto("/forgotpassword");
    await expect(page.locator("#emailStep")).toHaveClass(/active/);
    await expect(page.locator("#email")).toBeVisible();
  });

  test("T2: invalid email is rejected client-side", async ({ page }) => {
    await page.goto("/forgotpassword");
    await page.locator("#email").fill("not-an-email");
    await page.locator("#email").blur();
    await expect(page.locator("#emailError")).toHaveText(
      /please enter a valid email/i,
    );
    // Submit button stays disabled.
    await expect(
      page.locator("#emailStep button[type='submit']"),
    ).toBeDisabled();
  });

  test("T3: valid email advances to Step 2", async ({ page }) => {
    await page.goto("/forgotpassword");
    await page.locator("#email").fill(VERIFIED_EMAIL);
    await page.locator("#emailStep button[type='submit']").click();

    await expect(page.locator("#verifyCodeStep")).toHaveClass(/active/);
    await expect(page.locator("#verifyCode")).toBeVisible();
    // Masked email rendered in step header.
    await expect(page.locator("#verifyCodeStep .step-header")).toContainText(
      "te*",
    );
  });

  test("T4: verify code advances to Step 3", async ({ page }) => {
    await page.goto("/forgotpassword");
    await page.locator("#email").fill(VERIFIED_EMAIL);
    await page.locator("#emailStep button[type='submit']").click();
    await expect(page.locator("#verifyCodeStep")).toHaveClass(/active/);

    const token = await getResetTokenForEmail(VERIFIED_EMAIL);
    await page.locator("#verifyCode").fill(token);
    await page.locator("#verifyCodeStep button[type='submit']").click();

    await expect(page.locator("#newPasswordStep")).toHaveClass(/active/);
    await expect(page.locator("#password")).toBeVisible();
  });

  test("T5: mismatched passwords are rejected", async ({ page }) => {
    await page.goto("/forgotpassword");
    await page.locator("#email").fill(VERIFIED_EMAIL);
    await page.locator("#emailStep button[type='submit']").click();
    const token = await getResetTokenForEmail(VERIFIED_EMAIL);
    await page.locator("#verifyCode").fill(token);
    await page.locator("#verifyCodeStep button[type='submit']").click();

    await page.locator("#password").fill(NEW_PASSWORD);
    await page.locator("#confirmPassword").fill("DoesNotMatch1");
    await expect(page.locator("#confirmError")).toHaveText(
      /passwords do not match/i,
    );
    await expect(
      page.locator("#newPasswordStep button[type='submit']"),
    ).toBeDisabled();
  });

  test("T6: successful reset reaches Step 4 and new password works", async ({
    page,
  }) => {
    await page.goto("/forgotpassword");
    await page.locator("#email").fill(VERIFIED_EMAIL);
    await page.locator("#emailStep button[type='submit']").click();
    const token = await getResetTokenForEmail(VERIFIED_EMAIL);
    await page.locator("#verifyCode").fill(token);
    await page.locator("#verifyCodeStep button[type='submit']").click();

    await page.locator("#password").fill(NEW_PASSWORD);
    await page.locator("#confirmPassword").fill(NEW_PASSWORD);
    await page.locator("#newPasswordStep button[type='submit']").click();

    await expect(page.locator("#successStep")).toHaveClass(/active/);
    await expect(page.getByRole("link", { name: /continue to login/i })).toBeVisible();

    // Verify the new password actually works.
    await page.goto("/login");
    await page.locator("#loginEmail").fill(VERIFIED_EMAIL);
    await page.locator("#loginPassword").fill(NEW_PASSWORD);
    await page.getByRole("button", { name: /log in|sign in/i }).click();
    await page.waitForURL(/dashboard|chat|wallet|\/$/);
  });

  test("T7: back button returns from Step 2 to Step 1", async ({ page }) => {
    await page.goto("/forgotpassword");
    await page.locator("#email").fill(VERIFIED_EMAIL);
    await page.locator("#emailStep button[type='submit']").click();
    await expect(page.locator("#verifyCodeStep")).toHaveClass(/active/);

    // Step 2 back button is rendered as a <button> (callback mode) — Step 1
    // would render an <a href="/login"> link instead. Use the shared id.
    await page.locator("#backBtn").click();
    await expect(page.locator("#emailStep")).toHaveClass(/active/);
  });

  test("T8: authenticated user is redirected to /dashboard", async ({
    page,
  }) => {
    await login(page);
    await page.goto("/forgotpassword");
    await page.waitForURL(/\/dashboard/);
    await expect(page.url()).toMatch(/\/dashboard/);
  });
});
