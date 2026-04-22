import { test, expect, Page } from "@playwright/test";
import { resetMockState } from "../helpers/mock-server";

/**
 * E2E coverage for the wallet page (Task 8 of the wallet completion sprint).
 *
 * Cases:
 *   1. Balance renders with thousand separators.
 *   2. Expanding a transaction reveals the fee breakdown (formatted).
 *   3. Transfer happy path (friend → amount → confirm → success).
 *   4. Shop-account viewer: lazy-loaded delivery panel fires exactly one
 *      `shopOrderDetails` request on first expand.
 *   5. Non-shop viewer: SHOP_PURCHASE row expands but no delivery panel
 *      renders and no `shopOrderDetails` request is fired.
 *   6. Reload balance button refreshes the wallet without a navigation.
 *
 * Seed credentials (see tests/mock_backend/src/seed.rs):
 *   - test@peer.com / TestPass123     (regular user, has 3 P2P+payment txs)
 *   - alice@peer.com / AlicePass123   (regular user, buyer of seeded shop order)
 *   - shop@peer.com / ShopPass123     (Peer Shop operator, sees SHOP_PURCHASE row)
 */

const VERIFIED_EMAIL = "test@peer.com";
const VERIFIED_PASSWORD = "TestPass123";
const ALICE_EMAIL = "alice@peer.com";
const ALICE_PASSWORD = "AlicePass123";
const SHOP_EMAIL = "shop@peer.com";
const SHOP_PASSWORD = "ShopPass123";

async function login(page: Page, email: string, password: string): Promise<void> {
  await page.goto("/login");
  await page.locator("#loginEmail").fill(email);
  await page.locator("#loginPassword").fill(password);
  await page.getByRole("button", { name: /log in|sign in/i }).click();
  await page.waitForURL(/dashboard|chat|wallet|\/$/);
}

test.describe("Wallet — Track 8", () => {
  test.beforeEach(async () => {
    await resetMockState();
  });

  test("T1: balance renders with thousand separators", async ({ page }) => {
    await login(page, VERIFIED_EMAIL, VERIFIED_PASSWORD);
    await page.goto("/wallet");

    const token = page.locator("#token");
    await expect(token).toBeVisible();
    // Seed balance is 1000.0; format_balance() produces "1,000".
    await expect(token).toHaveText(/^\d{1,3}(,\d{3})+(\.\d+)?$/);
  });

  test("T2: expanding a transaction reveals fee breakdown", async ({ page }) => {
    await login(page, VERIFIED_EMAIL, VERIFIED_PASSWORD);
    await page.goto("/wallet");

    const firstRow = page.locator(".tarnsaction_item").first();
    await expect(firstRow).toBeVisible();
    await firstRow.locator(".transaction_record").click();

    const detail = firstRow.locator(".transaction_detail");
    await expect(detail).toBeVisible();
    await expect(detail.getByText("Transaction amount")).toBeVisible();
    await expect(detail.getByText("Fees included")).toBeVisible();
  });

  test("T3: transfer happy path completes successfully", async ({ page }) => {
    await login(page, VERIFIED_EMAIL, VERIFIED_PASSWORD);
    await page.goto("/wallet");

    await page.locator("#openTransferDropdown").click();
    const dropdown = page.locator("#transferDropdown");
    await expect(dropdown).toBeVisible();

    // Pick the first friend from the user list.
    const firstFriend = dropdown.locator(".user-item").first();
    await expect(firstFriend).toBeVisible();
    await firstFriend.click();

    // Enter a small amount and continue to confirmation.
    await dropdown.locator('input[type="number"]').fill("1");
    await dropdown.locator("textarea").fill("Test transfer from E2E");
    await dropdown.getByRole("button", { name: "Continue" }).click();

    // Submit from the confirmation screen.
    await dropdown.getByRole("button", { name: "Submit transfer" }).click();

    // Success screen renders, then OK closes the modal.
    await expect(dropdown.getByText("Transfer Complete!")).toBeVisible();
    await dropdown.getByRole("button", { name: "OK" }).click();
    await expect(dropdown).toBeHidden();
  });

  test("T4: shop-account viewer lazy-loads the delivery panel", async ({ page }) => {
    await login(page, SHOP_EMAIL, SHOP_PASSWORD);

    const shopOrderRequests: string[] = [];
    page.on("request", (req) => {
      if (req.method() === "POST" && /graphql/i.test(req.url())) {
        const body = req.postData() ?? "";
        if (body.includes("shopOrderDetails")) {
          shopOrderRequests.push(body);
        }
      }
    });

    await page.goto("/wallet");

    // Wait for the seeded SHOP_PURCHASE row to appear in the shop's history.
    const shopRow = page.locator(".tarnsaction_item", { hasText: /Peer Shop/i }).first();
    await expect(shopRow).toBeVisible();

    // No delivery request should fire before expanding.
    expect(shopOrderRequests).toHaveLength(0);

    await shopRow.locator(".transaction_record").click();

    // Delivery panel renders after the lazy fetch resolves.
    const panel = shopRow.locator(".delivery_info_container");
    await expect(panel).toBeVisible();
    await expect(panel.getByText("Delivery information")).toBeVisible();
    await expect(panel.getByText("Name")).toBeVisible();
    await expect(panel.getByText("Email")).toBeVisible();
    await expect(panel.getByText("Address")).toBeVisible();

    // Exactly one shopOrderDetails request fired (lazy + cached).
    expect(shopOrderRequests).toHaveLength(1);
  });

  test("T5: non-shop viewer never sees the delivery panel", async ({ page }) => {
    await login(page, ALICE_EMAIL, ALICE_PASSWORD);

    const shopOrderRequests: string[] = [];
    page.on("request", (req) => {
      if (req.method() === "POST" && /graphql/i.test(req.url())) {
        const body = req.postData() ?? "";
        if (body.includes("shopOrderDetails")) {
          shopOrderRequests.push(body);
        }
      }
    });

    await page.goto("/wallet");

    // Alice is the buyer of the seeded shop order, so the SHOP_PURCHASE row
    // appears in her history too — but the UI gate must hide the delivery
    // panel because she is not the Peer Shop account.
    const shopRow = page.locator(".tarnsaction_item", { hasText: /Peer Shop/i }).first();
    await expect(shopRow).toBeVisible();
    await shopRow.locator(".transaction_record").click();

    await expect(shopRow.locator(".transaction_detail")).toBeVisible();
    await expect(shopRow.locator(".delivery_info_container")).toHaveCount(0);
    expect(shopOrderRequests).toHaveLength(0);
  });

  test("T6: reload button refreshes balance in place", async ({ page }) => {
    await login(page, VERIFIED_EMAIL, VERIFIED_PASSWORD);
    await page.goto("/wallet");

    const token = page.locator("#token");
    await expect(token).toBeVisible();
    const before = await token.textContent();

    const balanceRequests: string[] = [];
    page.on("request", (req) => {
      if (req.method() === "POST" && /graphql/i.test(req.url())) {
        const body = req.postData() ?? "";
        if (/\bbalance\b/i.test(body)) {
          balanceRequests.push(body);
        }
      }
    });

    await page.locator("#reloadTransactions").click();

    // At least one fresh balance query fires; balance text remains a grouped
    // number (we do not assert a delta — seed balance is stable).
    await expect.poll(() => balanceRequests.length, { timeout: 5_000 }).toBeGreaterThan(0);
    await expect(token).toBeVisible();
    expect(await token.textContent()).toBe(before);
  });
});
