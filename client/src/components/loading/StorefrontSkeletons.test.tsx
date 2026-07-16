import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import {
  AccountOverviewSkeleton,
  AddressBookSkeleton,
  CartSkeleton,
  CheckoutSkeleton,
  OrderDetailSkeleton,
  OrderHistorySkeleton,
} from "./StorefrontSkeletons";

afterEach(cleanup);

describe("storefront loading skeletons", () => {
  it.each([
    ["account", <AccountOverviewSkeleton />, "Loading your account"],
    ["addresses", <AddressBookSkeleton />, "Loading saved addresses"],
    ["orders", <OrderHistorySkeleton />, "Loading order history"],
    ["order details", <OrderDetailSkeleton />, "Loading order details"],
    ["cart", <CartSkeleton />, "Loading your bag"],
    ["checkout", <CheckoutSkeleton />, "Preparing secure checkout"],
  ])("exposes the %s loading state to assistive technology", (_, component, label) => {
    const { container } = render(component);

    expect(screen.getByRole("status").getAttribute("aria-label")).toBe(label);
    expect(screen.getByRole("status").getAttribute("aria-busy")).toBe("true");
    expect(container.querySelectorAll(".skeleton").length).toBeGreaterThan(0);
  });
});
