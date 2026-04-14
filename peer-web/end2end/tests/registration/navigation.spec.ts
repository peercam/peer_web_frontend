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
