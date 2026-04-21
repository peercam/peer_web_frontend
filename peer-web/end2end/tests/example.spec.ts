import { test, expect } from "@playwright/test";

test("homepage has title and heading text", async ({ page }) => {
  await page.goto("/");

  await expect(page).toHaveTitle("Welcome to Peer");

  await expect(page.locator("h1")).toHaveText("Welcome to Peer");
});
