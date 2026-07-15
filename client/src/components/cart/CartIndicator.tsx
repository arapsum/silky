import { ShoppingCartIcon } from "@phosphor-icons/react";
import { cartItemCount, useCartStore } from "@/stores/cart";

export function CartIndicator() {
  const items = useCartStore((state) => state.items);
  const hydrated = useCartStore((state) => state.hydrated);
  const count = hydrated ? cartItemCount(items) : 0;

  return (
    <a aria-label={`Shopping bag with ${count} ${count === 1 ? "item" : "items"}`} className="cart-indicator" href="/cart">
      <ShoppingCartIcon aria-hidden size={22} weight="bold" />
      {count > 0 && <span>{count > 99 ? "99+" : count}</span>}
    </a>
  );
}
