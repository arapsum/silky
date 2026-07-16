import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { CartItem } from "@/stores/cart";
import { useCartStore } from "@/stores/cart";
import { CheckoutStatus } from "./CheckoutStatus";

const api = vi.hoisted(() => ({
  loadSession: vi.fn(),
}));

vi.mock("@/lib/api/browser", async (importOriginal) => {
  const original = await importOriginal<typeof import("@/lib/api/browser")>();
  return {
    ...original,
    orderApi: { ...original.orderApi, session: api.loadSession },
  };
});

const item: CartItem = {
  variantPid: "variant-one",
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
  api.loadSession.mockReset();
  useCartStore.setState({
    items: [item],
    quote: null,
    checkoutAttempt: null,
    pendingCheckout: {
      orderPid: "order-one",
      checkoutKey: "checkout-one",
      checkoutUrl: "https://checkout.stripe.com/example",
      expiresAt: "2026-07-16T12:00:00Z",
    },
    expiresAt: Date.now() + 60_000,
    hydrated: true,
  });
});

afterEach(cleanup);

describe("CheckoutStatus", () => {
  it("clears the cart only after Silk reports a paid order", async () => {
    api.loadSession.mockResolvedValue({
      orderPid: "order-one",
      orderStatus: "confirmed",
      paymentStatus: "paid",
      attemptStatus: "succeeded",
      checkoutUrl: null,
      expiresAt: "2026-07-16T12:00:00Z",
    });

    render(<CheckoutStatus mode="success" orderPid="order-one" />);

    expect(await screen.findByRole("heading", { name: "Your order is in." })).toBeTruthy();
    await waitFor(() => expect(useCartStore.getState().items).toEqual([]));
    expect(useCartStore.getState().pendingCheckout).toBeNull();
  });

  it("preserves the cart when payment reaches a terminal failure", async () => {
    api.loadSession.mockResolvedValue({
      orderPid: "order-one",
      orderStatus: "cancelled",
      paymentStatus: "failed",
      attemptStatus: "failed",
      checkoutUrl: null,
      expiresAt: "2026-07-16T12:00:00Z",
    });

    render(<CheckoutStatus mode="success" orderPid="order-one" />);

    await screen.findByRole("heading", { name: "Payment was not completed." });
    expect(useCartStore.getState().items).toEqual([item]);
  });
});
