import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { CartItem } from "@/stores/cart";
import { useCartStore } from "@/stores/cart";
import { CheckoutPage } from "./CheckoutPage";

const api = vi.hoisted(() => ({
  checkout: vi.fn(),
  listAddresses: vi.fn(),
  loadSession: vi.fn(),
  quote: vi.fn(),
}));

vi.mock("@/lib/api/browser", async (importOriginal) => {
  const original = await importOriginal<typeof import("@/lib/api/browser")>();
  return {
    ...original,
    addressApi: { ...original.addressApi, list: api.listAddresses },
    cartApi: { quote: api.quote },
    orderApi: { ...original.orderApi, checkout: api.checkout },
    sessionApi: { ...original.sessionApi, me: api.loadSession },
  };
});

const item: CartItem = {
  variantPid: "db365773-2ac1-49aa-a4b9-03dcf8ac3401",
  productPid: "product-one",
  productSlug: "court-leather-sneakers",
  productName: "Court Leather Sneakers",
  sku: "SNEAKER-WHT-42",
  imageUrl: null,
  selectedOptions: { Colour: "White", Size: "42" },
  unitPrice: "89.99",
  quantity: 1,
  availableQuantity: 3,
};

beforeEach(() => {
  api.checkout.mockReset();
  api.listAddresses.mockResolvedValue([
    {
      pid: "shipping-one",
      addressType: "shipping",
      label: "Home",
      recipientName: "John Doe",
      company: null,
      lineOne: "10 Market Street",
      lineTwo: null,
      city: "Nairobi",
      region: null,
      postalCode: null,
      countryCode: "KE",
      email: null,
      phone: null,
      isDefault: true,
      createdAt: "2026-07-16T09:00:00Z",
      updatedAt: "2026-07-16T09:00:00Z",
      deletedAt: null,
    },
  ]);
  api.loadSession.mockResolvedValue({ email: "john@example.com" });
  api.quote.mockResolvedValue({
    currency: "USD",
    subtotal: "89.99",
    shippingTotal: "0",
    taxTotal: "0",
    grandTotal: "89.99",
    canCheckout: true,
    items: [],
  });
  useCartStore.setState({
    items: [item],
    quote: null,
    checkoutAttempt: null,
    pendingCheckout: null,
    expiresAt: Date.now() + 60_000,
    hydrated: true,
  });
});

afterEach(cleanup);

describe("CheckoutPage", () => {
  it("reuses the exact checkout request after an ambiguous failure", async () => {
    const user = userEvent.setup();
    vi.spyOn(crypto, "randomUUID").mockReturnValue("3b15553e-ea25-44b8-949e-17974fe90cc6c");
    api.checkout.mockRejectedValue(new Error("connection closed"));
    render(<CheckoutPage />);

    await user.click(await screen.findByRole("button", { name: /continue to payment/i }));
    await screen.findByRole("button", { name: /retry secure payment/i });
    await user.click(screen.getByRole("button", { name: /retry secure payment/i }));

    await waitFor(() => expect(api.checkout).toHaveBeenCalledTimes(2));
    expect(api.checkout.mock.calls[0]?.[0]).toEqual(api.checkout.mock.calls[1]?.[0]);
    expect(api.checkout.mock.calls[0]?.[0]).toMatchObject({
      checkoutKey: "3b15553e-ea25-44b8-949e-17974fe90cc6c",
      shippingAddressPid: "shipping-one",
      items: [{ variantPid: item.variantPid, quantity: 1 }],
    });
  });
});
