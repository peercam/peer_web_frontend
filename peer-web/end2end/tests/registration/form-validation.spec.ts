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
