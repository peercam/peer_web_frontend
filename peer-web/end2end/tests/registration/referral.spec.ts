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
