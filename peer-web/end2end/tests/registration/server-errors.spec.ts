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
