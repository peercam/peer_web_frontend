import { test, expect } from "@playwright/test";

/**
 * PWA smoke coverage for the DoD in docs/plans/pwa/pwa-implementation.md:
 *   - manifest served at /manifest.webmanifest with the correct MIME
 *   - service worker registers and takes control after a reload
 *   - offline navigation falls back to /offline.html
 *   - coming back online restores normal routing
 */

test.describe("PWA", () => {
  test("manifest is served with application/manifest+json", async ({
    request,
  }) => {
    const res = await request.get("/manifest.webmanifest");
    expect(res.status()).toBe(200);
    expect(res.headers()["content-type"]).toContain("application/manifest+json");
    const body = await res.json();
    expect(body.id).toBe("/");
    expect(body.start_url).toContain("/dashboard");
    expect(Array.isArray(body.icons)).toBe(true);
    expect(body.icons.length).toBeGreaterThanOrEqual(4);
  });

  test("service worker registers and controls the page after reload", async ({
    page,
  }) => {
    await page.goto("/dashboard");
    // Wait for registration to complete.
    await page.waitForFunction(
      async () => {
        const reg = await navigator.serviceWorker.getRegistration();
        return !!reg && (!!reg.active || !!reg.installing || !!reg.waiting);
      },
      null,
      { timeout: 10_000 },
    );

    // Reload so the SW is in control on the second navigation.
    await page.reload();
    const controlled = await page.evaluate(
      () => navigator.serviceWorker.controller !== null,
    );
    expect(controlled).toBe(true);
  });

  test("offline navigation falls back to the offline shell", async ({
    page,
    context,
  }) => {
    await page.goto("/dashboard");
    await page.waitForFunction(
      async () => (await navigator.serviceWorker.getRegistration())?.active !== undefined,
      null,
      { timeout: 10_000 },
    );
    await page.reload(); // ensure controller is set

    await context.setOffline(true);
    const res = await page.goto("/dashboard", { waitUntil: "domcontentloaded" });
    // When the SW is controlling, the fallback is served with 200 from the cache.
    expect(res).not.toBeNull();
    await expect(page.locator("h1")).toHaveText(/offline/i);

    await context.setOffline(false);
    await page.goto("/dashboard", { waitUntil: "domcontentloaded" });
    // Back online — offline heading should no longer be the primary h1.
    await expect(page.locator("h1")).not.toHaveText(/offline/i);
  });
});
