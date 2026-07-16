import { beforeEach, describe, expect, it } from "vitest";
import { CART_COUNT_COOKIE, cartItemCount, type CartItem, useCartStore } from "./cart";

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
  document.cookie = `${CART_COUNT_COOKIE}=; Path=/; Max-Age=0`;
  useCartStore.setState({
    items: [],
    quote: null,
    checkoutAttempt: null,
    pendingCheckout: null,
    expiresAt: Date.now() + 60_000,
    hydrated: true,
  });
});

describe("cart store", () => {
  it("merges the same variant and clamps it to available stock", () => {
    useCartStore.getState().add(item);
    useCartStore.getState().add({ ...item, quantity: 3 });

    expect(useCartStore.getState().items).toEqual([{ ...item, quantity: 3 }]);
    expect(cartItemCount(useCartStore.getState().items)).toBe(3);
    expect(document.cookie).toContain(`${CART_COUNT_COOKIE}=3`);
  });

  it("invalidates a quote when quantity changes", () => {
    useCartStore.getState().add(item);
    useCartStore.setState({
      quote: {
        currency: "USD",
        subtotal: "89.99",
        shippingTotal: "0",
        taxTotal: "0",
        grandTotal: "89.99",
        canCheckout: true,
        items: [],
      },
    });

    useCartStore.getState().setQuantity(item.variantPid, 2);

    expect(useCartStore.getState().items[0]?.quantity).toBe(2);
    expect(useCartStore.getState().quote).toBeNull();
  });

  it("keeps a pending checkout until the customer explicitly clears it", () => {
    const pending = {
      orderPid: "order-one",
      checkoutKey: "checkout-one",
      checkoutUrl: "https://checkout.stripe.com/example",
      expiresAt: new Date(Date.now() + 60_000).toISOString(),
    };
    useCartStore.getState().setPendingCheckout(pending);
    expect(useCartStore.getState().pendingCheckout).toEqual(pending);
    useCartStore.getState().setPendingCheckout(null);
    expect(useCartStore.getState().pendingCheckout).toBeNull();
  });

  it("keeps an in-flight checkout request available for an idempotent retry", () => {
    const attempt = {
      checkoutKey: "checkout-one",
      shippingAddressPid: "shipping-one",
      billingAddressPid: "billing-one",
      items: [{ variantPid: item.variantPid, quantity: item.quantity }],
      createdAt: Date.now(),
    };

    useCartStore.getState().setCheckoutAttempt(attempt);

    expect(useCartStore.getState().checkoutAttempt).toEqual(attempt);
    useCartStore.getState().setCheckoutAttempt(null);
    expect(useCartStore.getState().checkoutAttempt).toBeNull();
  });
});
