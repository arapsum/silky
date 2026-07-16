import { expect, test } from "@playwright/test";

test("renders the empty bag with a path back to the collection", async ({ page }) => {
  await page.goto("/cart");
  await expect(page.getByRole("heading", { name: "Start with something useful." })).toBeVisible();
  await expect(page.getByRole("link", { name: "Browse the collection" })).toHaveAttribute(
    "href",
    "/shop",
  );
});

test("uses focused navigation on customer authentication pages", async ({ page }) => {
  await page.goto("/auth/login");
  await expect(page.getByRole("link", { name: "Silk home" })).toBeVisible();
  await expect(page.getByRole("link", { name: "Back to shop" })).toHaveAttribute("href", "/shop");
  await expect(page.getByLabel(/Shopping bag with/)).toHaveCount(0);
});

test("shows a usable customer sign-in form", async ({ page }) => {
  await page.goto("/auth/login");
  await expect(page.getByRole("heading", { name: "Sign in to continue." })).toBeVisible();
  await expect(page.getByLabel("Email address")).toBeVisible();
  await expect(page.getByLabel("Password")).toBeVisible();
});

test("keeps credentials out of the URL before React hydrates", async ({ browser }) => {
  const context = await browser.newContext({ javaScriptEnabled: false });
  const page = await context.newPage();
  await page.route("**/auth/login", async (route) => {
    if (route.request().method() === "POST") {
      await route.fulfill({ body: "", status: 204 });
      return;
    }
    await route.continue();
  });

  await page.goto("/auth/login");
  await page.getByLabel("Email address").fill("test1@example.com");
  await page.getByLabel("Password").fill("Password!");
  const requestPromise = page.waitForRequest(
    (request) => request.url().endsWith("/auth/login") && request.method() === "POST",
  );
  await page.getByRole("button", { name: "Sign in" }).click();
  const request = await requestPromise;

  expect(request.method()).toBe("POST");
  expect(request.url()).not.toContain("email=");
  expect(request.url()).not.toContain("password=");
  expect(request.postData()).toContain("email=test1%40example.com");
  await context.close();
});

for (const path of ["/auth/login", "/auth/register"]) {
  test(`${path} fits within the initial viewport`, async ({ page }) => {
    await page.goto(path);
    const dimensions = await page.evaluate(() => ({
      documentHeight: document.documentElement.scrollHeight,
      viewportHeight: window.innerHeight,
    }));
    expect(dimensions.documentHeight).toBeLessThanOrEqual(dimensions.viewportHeight + 1);
  });
}
