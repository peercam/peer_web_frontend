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
    await regPage.passwordInput.click();
    await regPage.passwordInput.fill("abcd");
    await regPage.passwordInput.evaluate((el: HTMLInputElement) => {
      el.dispatchEvent(new Event("input", { bubbles: true }));
    });

    // Password strength meter should be visible and show weak state
    await expect(regPage.passwordStrength).toBeVisible();

    // Requirements should indicate what's missing
    const strengthText = await regPage.passwordStrength.textContent();
    expect(strengthText).toMatch(/weak|very weak/i);

    // Fill rest of form — submit should remain blocked because password is weak
    await regPage.emailInput.fill("test@example.com");
    await regPage.usernameInput.fill("test_user");
    await regPage.confirmPasswordInput.fill("abcd");
    await regPage.privacyCheckbox.check();
    await regPage.eulaCheckbox.check();

    // Submit button stays disabled while the form is invalid
    await expect(regPage.registerButton).toBeDisabled();

    // Should remain on step 2 — password too weak
    await regPage.expectActiveStep(2);
  });

  test("T5b: strong password shows strong/excellent indicator", async () => {
    await regPage.passwordInput.click();
    await regPage.passwordInput.fill("S3cur3P@ssw0rd!");
    // Re-dispatch an input event to ensure Leptos sees the value even if
    // Playwright's `fill` set it before hydration finished wiring the handler.
    await regPage.passwordInput.evaluate((el: HTMLInputElement) => {
      el.dispatchEvent(new Event("input", { bubbles: true }));
    });

    await expect(regPage.passwordStrength).toBeVisible();
    const strengthText = await regPage.passwordStrength.textContent();
    expect(strengthText).toMatch(/good|excellent|strong/i);
  });

  // ── T6: Mismatched confirm password ─────────────────────────────────────
  test("T6: mismatched confirm password shows error", async () => {
    await regPage.passwordInput.fill("SecurePass123!");
    await regPage.confirmPasswordInput.fill("DifferentPass456!");

    // Validation is reactive — message appears as soon as inputs mismatch
    await expect(regPage.confirmPasswordValidation).toContainText(
      /passwords do not match/i
    );

    // Submit is blocked while the form is invalid
    await expect(regPage.registerButton).toBeDisabled();

    // Should remain on step 2
    await regPage.expectActiveStep(2);
  });

  // ── T7: Unchecked checkboxes ────────────────────────────────────────────
  test("T7: unchecked checkboxes show error message", async ({ page }) => {
    await regPage.fillRegistrationForm();

    // Uncheck the checkboxes that fillRegistrationForm checked
    await regPage.privacyCheckbox.uncheck();
    await regPage.eulaCheckbox.uncheck();

    // Submit button is disabled; trigger the form's submit handler directly
    // to exercise the checkbox validation path.
    await expect(regPage.registerButton).toBeDisabled();
    await page.evaluate(() => {
      const form = document.getElementById("registrationForm") as HTMLFormElement | null;
      form?.requestSubmit();
    });

    await expect(regPage.checkboxValidation).toBeVisible();
    await expect(regPage.checkboxValidation).toContainText(
      /privacy policy|eula|accept both/i
    );

    await regPage.expectActiveStep(2);
  });

  test("T7b: single unchecked checkbox still shows error", async ({ page }) => {
    await regPage.fillRegistrationForm();
    await regPage.eulaCheckbox.uncheck(); // only uncheck EULA

    await expect(regPage.registerButton).toBeDisabled();
    await page.evaluate(() => {
      const form = document.getElementById("registrationForm") as HTMLFormElement | null;
      form?.requestSubmit();
    });

    await expect(regPage.checkboxValidation).toBeVisible();
    await regPage.expectActiveStep(2);
  });
});
