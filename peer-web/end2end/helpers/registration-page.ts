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
  async expectToast(textSubstring: string | RegExp) {
    // Toasts auto-dismiss after ~3s; earlier "info" toasts (e.g. "Referral
    // information loaded.") can linger from a previous step. Poll with a
    // generous timeout so we wait until the expected toast replaces any
    // stale one rather than matching the wrong text.
    await expect(this.toast).toBeVisible({ timeout: 5_000 });
    await expect(this.toast).toContainText(textSubstring, { timeout: 8_000 });
  }

  /** Assert which step is currently active/visible. */
  async expectActiveStep(step: 1 | 2 | 3) {
    const stepLocator = this.page.locator(`[data-step="${step}"]`);
    await expect(stepLocator).toBeVisible();
  }
}
