import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
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

  // ── SSR Verification ─────────────────────────────────────────────────────
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

  // ── Accessibility Test ───────────────────────────────────────────────────
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
});
