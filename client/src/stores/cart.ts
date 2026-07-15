import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";
import type { CartQuote } from "@/lib/api/types";

const CART_LIFETIME_MS = 30 * 24 * 60 * 60 * 1000;

export interface CartItem {
  variantPid: string;
  productPid: string;
  productSlug: string;
  productName: string;
  sku: string;
  imageUrl: string | null;
  selectedOptions: Record<string, string>;
  unitPrice: string;
  quantity: number;
  availableQuantity: number;
}

export interface PendingCheckout {
  orderPid: string;
  checkoutKey: string;
  checkoutUrl: string;
  expiresAt: string;
}

interface CartState {
  items: CartItem[];
  quote: CartQuote | null;
  pendingCheckout: PendingCheckout | null;
  expiresAt: number;
  hydrated: boolean;
  add: (item: CartItem) => void;
  remove: (variantPid: string) => void;
  setQuantity: (variantPid: string, quantity: number) => void;
  setQuote: (quote: CartQuote | null) => void;
  setPendingCheckout: (checkout: PendingCheckout | null) => void;
  clear: () => void;
  setHydrated: (hydrated: boolean) => void;
}

function activeItems(items: CartItem[], expiresAt: number) {
  return expiresAt > Date.now() ? items : [];
}

export const useCartStore = create<CartState>()(
  persist(
    (set) => ({
      items: [],
      quote: null,
      pendingCheckout: null,
      expiresAt: Date.now() + CART_LIFETIME_MS,
      hydrated: false,
      add: (item) =>
        set((state) => {
          const items = activeItems(state.items, state.expiresAt);
          const existing = items.find((entry) => entry.variantPid === item.variantPid);
          const nextQuantity = Math.min(
            item.availableQuantity,
            (existing?.quantity ?? 0) + item.quantity,
          );
          return {
            items: existing
              ? items.map((entry) =>
                  entry.variantPid === item.variantPid
                    ? { ...entry, ...item, quantity: nextQuantity }
                    : entry,
                )
              : [...items, { ...item, quantity: nextQuantity }],
            quote: null,
            expiresAt: Date.now() + CART_LIFETIME_MS,
          };
        }),
      remove: (variantPid) =>
        set((state) => ({
          items: state.items.filter((item) => item.variantPid !== variantPid),
          quote: null,
          expiresAt: Date.now() + CART_LIFETIME_MS,
        })),
      setQuantity: (variantPid, quantity) =>
        set((state) => ({
          items: state.items.map((item) =>
            item.variantPid === variantPid
              ? { ...item, quantity: Math.max(1, Math.min(quantity, item.availableQuantity)) }
              : item,
          ),
          quote: null,
          expiresAt: Date.now() + CART_LIFETIME_MS,
        })),
      setQuote: (quote) => set({ quote }),
      setPendingCheckout: (pendingCheckout) => set({ pendingCheckout }),
      clear: () =>
        set({
          items: [],
          quote: null,
          pendingCheckout: null,
          expiresAt: Date.now() + CART_LIFETIME_MS,
        }),
      setHydrated: (hydrated) => set({ hydrated }),
    }),
    {
      name: "silk-cart",
      version: 1,
      storage: createJSONStorage(() => localStorage),
      partialize: ({ items, pendingCheckout, expiresAt }) => ({ items, pendingCheckout, expiresAt }),
      merge: (persisted, current) => {
        const saved = persisted as Partial<CartState>;
        const valid = (saved.expiresAt ?? 0) > Date.now();
        return {
          ...current,
          items: valid ? (saved.items ?? []) : [],
          pendingCheckout: valid ? (saved.pendingCheckout ?? null) : null,
          expiresAt: valid ? (saved.expiresAt ?? current.expiresAt) : current.expiresAt,
        };
      },
      onRehydrateStorage: () => (state) => state?.setHydrated(true),
    },
  ),
);

export function cartItemCount(items: CartItem[]) {
  return items.reduce((total, item) => total + item.quantity, 0);
}
