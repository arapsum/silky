import { ArrowRightIcon, MinusIcon, PlusIcon, TrashIcon } from "@phosphor-icons/react";
import { useEffect, useMemo, useState } from "react";
import { cartApi } from "@/lib/api/browser";
import { formatCurrency } from "@/lib/format";
import { hydrateCartStore, useCartStore } from "@/stores/cart";

export function CartPage() {
  const items = useCartStore((state) => state.items);
  const hydrated = useCartStore((state) => state.hydrated);
  const quote = useCartStore((state) => state.quote);
  const setQuote = useCartStore((state) => state.setQuote);
  const setQuantity = useCartStore((state) => state.setQuantity);
  const remove = useCartStore((state) => state.remove);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const requestKey = useMemo(
    () => items.map((item) => `${item.variantPid}:${item.quantity}`).join("|"),
    [items],
  );

  useEffect(() => {
    void hydrateCartStore();
  }, []);

  useEffect(() => {
    if (!hydrated || items.length === 0) {
      setQuote(null);
      return;
    }
    const controller = new AbortController();
    const timeout = window.setTimeout(async () => {
      setLoading(true);
      setError(null);
      try {
        setQuote(
          await cartApi.quote(items.map(({ variantPid, quantity }) => ({ variantPid, quantity }))),
        );
      } catch (requestError) {
        if (!controller.signal.aborted) {
          setError(
            requestError instanceof Error ? requestError.message : "Your bag could not be checked.",
          );
        }
      } finally {
        if (!controller.signal.aborted) setLoading(false);
      }
    }, 250);
    return () => {
      controller.abort();
      window.clearTimeout(timeout);
    };
  }, [hydrated, requestKey]);

  if (!hydrated) {
    return (
      <div className="cart-state" aria-live="polite">
        Loading your bag...
      </div>
    );
  }
  if (items.length === 0) {
    return (
      <section className="cart-state">
        <p className="eyebrow">Your bag is empty</p>
        <h1>Start with something useful.</h1>
        <p>Your Silk pieces will stay here for 30 days on this device.</p>
        <a className="button-primary" href="/shop">
          Browse the collection
        </a>
      </section>
    );
  }

  return (
    <div className="cart-layout">
      <section aria-labelledby="bag-heading">
        <div className="cart-heading">
          <div>
            <p className="eyebrow">Your selection</p>
            <h1 id="bag-heading">Shopping bag</h1>
          </div>
          <span>
            {items.length} {items.length === 1 ? "piece" : "pieces"}
          </span>
        </div>

        <div className="cart-lines">
          {items.map((item) => {
            const line = quote?.items.find((entry) => entry.variantPid === item.variantPid);
            return (
              <article className="cart-line" key={item.variantPid}>
                <a className="cart-line__image" href={`/products/${item.productSlug}`}>
                  {item.imageUrl ? (
                    <img alt="" height="320" loading="lazy" src={item.imageUrl} width="256" />
                  ) : (
                    <span>Silk</span>
                  )}
                </a>
                <div className="cart-line__body">
                  <div>
                    <a className="cart-line__name" href={`/products/${item.productSlug}`}>
                      {item.productName}
                    </a>
                    <p>
                      {Object.entries(item.selectedOptions)
                        .map(([name, value]) => `${name}: ${value}`)
                        .join(" · ")}
                    </p>
                    <p className="cart-line__sku">{item.sku}</p>
                  </div>
                  <strong>{formatCurrency(line?.unitPrice ?? item.unitPrice)}</strong>
                  <div className="cart-line__footer">
                    <div className="quantity-control">
                      <button
                        aria-label={`Reduce ${item.productName} quantity`}
                        disabled={item.quantity <= 1}
                        onClick={() => setQuantity(item.variantPid, item.quantity - 1)}
                        type="button"
                      >
                        <MinusIcon aria-hidden size={14} />
                      </button>
                      <output>{item.quantity}</output>
                      <button
                        aria-label={`Increase ${item.productName} quantity`}
                        disabled={
                          item.quantity >= (line?.availableQuantity ?? item.availableQuantity)
                        }
                        onClick={() => setQuantity(item.variantPid, item.quantity + 1)}
                        type="button"
                      >
                        <PlusIcon aria-hidden size={14} />
                      </button>
                    </div>
                    <button
                      className="cart-line__remove"
                      onClick={() => remove(item.variantPid)}
                      type="button"
                    >
                      <TrashIcon aria-hidden size={16} /> Remove
                    </button>
                  </div>
                  {line && line.status !== "available" && (
                    <p className="form-error">
                      {line.status === "insufficient_stock"
                        ? `Only ${line.availableQuantity} available. Adjust the quantity to continue.`
                        : "This variant is no longer available. Remove it to continue."}
                    </p>
                  )}
                </div>
              </article>
            );
          })}
        </div>
      </section>

      <aside className="order-summary" aria-labelledby="summary-heading">
        <p className="eyebrow">Order summary</p>
        <h2 id="summary-heading">Ready when you are.</h2>
        <dl>
          <div>
            <dt>Subtotal</dt>
            <dd>{quote ? formatCurrency(quote.subtotal) : "Checking..."}</dd>
          </div>
          <div>
            <dt>Shipping</dt>
            <dd>{quote ? formatCurrency(quote.shippingTotal) : "Checking..."}</dd>
          </div>
          <div>
            <dt>Tax</dt>
            <dd>{quote ? formatCurrency(quote.taxTotal) : "Checking..."}</dd>
          </div>
          <div className="order-summary__total">
            <dt>Total</dt>
            <dd>{quote ? formatCurrency(quote.grandTotal) : "Checking..."}</dd>
          </div>
        </dl>
        {error && (
          <p className="form-error" role="alert">
            {error}
          </p>
        )}
        <a
          aria-disabled={!quote?.canCheckout || loading}
          className="button-primary order-summary__checkout"
          href={quote?.canCheckout && !loading ? "/checkout" : undefined}
        >
          {loading ? "Checking your bag" : "Continue to checkout"}
          <ArrowRightIcon aria-hidden size={17} />
        </a>
        <p className="order-summary__note">Prices and availability are confirmed before payment.</p>
      </aside>
    </div>
  );
}
