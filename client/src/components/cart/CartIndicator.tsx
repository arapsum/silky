import { ShoppingCartIcon } from "@phosphor-icons/react";
import { useEffect } from "react";
import { cartItemCount, hydrateCartStore, useCartStore } from "@/stores/cart";

interface CartIndicatorProps {
  initialCount?: number;
}

export function CartIndicator({ initialCount = 0 }: CartIndicatorProps) {
  const items = useCartStore((state) => state.items);
  const hydrated = useCartStore((state) => state.hydrated);
  const count = hydrated ? cartItemCount(items) : initialCount;

  useEffect(() => {
    void hydrateCartStore();
  }, []);

  return (
    <a
      aria-label={`Shopping bag with ${count} ${count === 1 ? "item" : "items"}`}
      className="cart-indicator"
      href="/cart"
    >
      <ShoppingCartIcon aria-hidden size={22} weight="bold" />
      <span className="cart-indicator__label">Bag ({count > 99 ? "99+" : count})</span>
    </a>
  );
}
