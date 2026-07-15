import { expect, test } from "@playwright/test";

test("renders the empty bag with a path back to the collection", async ({ page }) => {
  await page.goto("/cart");
  await expect(page.getByRole("heading", { name: "Start with something useful." })).toBeVisible();
  await expect(page.getByRole("link", { name: "Browse the collection" })).toHaveAttribute("href", "/shop");
});

test("keeps the desktop cart and account controls aligned", async ({ page }, testInfo) => {
  test.skip(testInfo.project.name === "mobile", "Desktop navigation assertion");
  await page.goto("/auth/login");
  const cart = await page.getByLabel(/Shopping bag with/).boundingBox();
  const account = await page.getByLabel("Your account").boundingBox();
  expect(cart).not.toBeNull();
  expect(account).not.toBeNull();
  expect(Math.abs((cart?.y ?? 0) - (account?.y ?? 0))).toBeLessThan(3);
});

test("shows a usable customer sign-in form", async ({ page }) => {
  await page.goto("/auth/login");
  await expect(page.getByRole("heading", { name: "Sign in to continue." })).toBeVisible();
  await expect(page.getByLabel("Email address")).toBeVisible();
  await expect(page.getByLabel("Password")).toBeVisible();
});
