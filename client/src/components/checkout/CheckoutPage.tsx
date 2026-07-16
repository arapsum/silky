import { ArrowLeftIcon, ArrowRightIcon, CheckIcon, PlusIcon } from "@phosphor-icons/react";
import { useEffect, useState } from "react";
import { AddressForm } from "@/components/account/AddressForm";
import { addressApi, ApiError, cartApi, orderApi, sessionApi } from "@/lib/api/browser";
import type { Address, CartQuote, UserSession } from "@/lib/api/types";
import { formatCurrency } from "@/lib/format";
import { CheckoutSkeleton } from "@/components/loading/StorefrontSkeletons";
import { useCartStore } from "@/stores/cart";
import { toast } from "@/lib/toast";

export function CheckoutPage() {
  const items = useCartStore((state) => state.items);
  const hydrated = useCartStore((state) => state.hydrated);
  const checkoutAttempt = useCartStore((state) => state.checkoutAttempt);
  const pending = useCartStore((state) => state.pendingCheckout);
  const setCheckoutAttempt = useCartStore((state) => state.setCheckoutAttempt);
  const setPending = useCartStore((state) => state.setPendingCheckout);
  const [user, setUser] = useState<UserSession | null>(null);
  const [addresses, setAddresses] = useState<Address[]>([]);
  const [shippingPid, setShippingPid] = useState("");
  const [billingPid, setBillingPid] = useState("");
  const [sameAddress, setSameAddress] = useState(true);
  const [quote, setQuote] = useState<CartQuote | null>(null);
  const [addingAddress, setAddingAddress] = useState(false);
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!hydrated) return;
    if (items.length === 0) {
      setLoading(false);
      return;
    }
    Promise.all([
      sessionApi.me(),
      addressApi.list(),
      cartApi.quote(items.map(({ variantPid, quantity }) => ({ variantPid, quantity }))),
    ])
      .then(([session, savedAddresses, cartQuote]) => {
        setUser(session);
        setAddresses(savedAddresses);
        setQuote(cartQuote);
        const shipping = savedAddresses.find((address) => address.addressType === "shipping" && address.isDefault)
          ?? savedAddresses.find((address) => address.addressType === "shipping")
          ?? savedAddresses[0];
        const billing = savedAddresses.find((address) => address.addressType === "billing" && address.isDefault)
          ?? savedAddresses.find((address) => address.addressType === "billing");
        setShippingPid(shipping?.pid ?? "");
        setBillingPid(billing?.pid ?? "");
      })
      .catch((requestError) => setError(requestError instanceof ApiError ? requestError.message : "Checkout could not be loaded."))
      .finally(() => setLoading(false));
  }, [hydrated]);

  async function startCheckout() {
    if (!shippingPid || !quote?.canCheckout) return;
    setSubmitting(true);
    setError(null);
    const attempt = checkoutAttempt ?? {
      checkoutKey: crypto.randomUUID(),
      shippingAddressPid: shippingPid,
      billingAddressPid: sameAddress ? undefined : billingPid || undefined,
      items: items.map(({ variantPid, quantity }) => ({ variantPid, quantity })),
      createdAt: Date.now(),
    };
    if (!checkoutAttempt) setCheckoutAttempt(attempt);
    try {
      const checkout = await orderApi.checkout({
        checkoutKey: attempt.checkoutKey,
        shippingAddressPid: attempt.shippingAddressPid,
        billingAddressPid: attempt.billingAddressPid,
        items: attempt.items,
      });
      setPending({ checkoutKey: attempt.checkoutKey, orderPid: checkout.orderPid, checkoutUrl: checkout.checkoutUrl, expiresAt: checkout.expiresAt });
      setCheckoutAttempt(null);
      window.location.assign(checkout.checkoutUrl);
    } catch (requestError) {
      if (requestError instanceof ApiError && requestError.status < 500) {
        setCheckoutAttempt(null);
      }
      const message = requestError instanceof ApiError ? requestError.message : "Payment could not be started.";
      setError(message);
      toast.error("Payment could not be started", message);
      setSubmitting(false);
    }
  }

  async function cancelPending() {
    if (!pending) return;
    setSubmitting(true);
    try {
      await orderApi.cancel(pending.orderPid);
      setPending(null);
      toast.success("Checkout cancelled", "Your bag is ready whenever you want to try again.");
    } catch (requestError) {
      const message = requestError instanceof ApiError ? requestError.message : "The pending checkout could not be cancelled.";
      setError(message);
      toast.error("Checkout not cancelled", message);
    } finally {
      setSubmitting(false);
    }
  }

  if (!hydrated || loading) return <CheckoutSkeleton />;
  if (items.length === 0) return <div className="checkout-state"><h1>Your bag is empty.</h1><a className="button-primary" href="/shop">Browse the collection</a></div>;
  if (error && !quote) return <div className="checkout-state"><h1>Checkout needs a moment.</h1><p>{error}</p><a className="button-secondary" href="/cart">Return to your bag</a></div>;

  if (pending && new Date(pending.expiresAt).getTime() > Date.now()) {
    return (
      <section className="checkout-pending">
        <p className="eyebrow">Payment in progress</p>
        <h1>Continue where you left off.</h1>
        <p>Your pieces are reserved until {new Intl.DateTimeFormat("en", { hour: "numeric", minute: "2-digit" }).format(new Date(pending.expiresAt))}.</p>
        <div>
          <a className="button-primary" href={pending.checkoutUrl}>Resume payment <ArrowRightIcon aria-hidden size={17} /></a>
          <button className="button-secondary" disabled={submitting} onClick={cancelPending} type="button">Cancel this checkout</button>
        </div>
      </section>
    );
  }

  return (
    <div className="checkout-layout">
      <section className="checkout-main">
        <a className="checkout-back" href="/cart"><ArrowLeftIcon aria-hidden size={15} /> Back to bag</a>
        <p className="eyebrow">Secure checkout</p>
        <h1>Where should it go?</h1>
        {user && <p className="checkout-user">Checking out as <strong>{user.email}</strong></p>}

        {addresses.length > 0 && (
          <div className="address-options" role="radiogroup" aria-label="Shipping address">
            {addresses.filter((address) => address.addressType !== "billing").map((address) => (
              <label className={shippingPid === address.pid ? "is-selected" : ""} key={address.pid}>
                <input checked={shippingPid === address.pid} name="shipping" onChange={() => setShippingPid(address.pid)} type="radio" value={address.pid} />
                <span><strong>{address.label ?? address.recipientName}</strong>{address.lineOne}<br />{address.city}{address.region ? `, ${address.region}` : ""} · {address.countryCode}</span>
                {shippingPid === address.pid && <CheckIcon aria-hidden size={18} weight="bold" />}
              </label>
            ))}
          </div>
        )}
        {!addingAddress && <button className="add-address" onClick={() => setAddingAddress(true)} type="button"><PlusIcon aria-hidden size={16} /> Add a delivery address</button>}
        {(addingAddress || addresses.length === 0) && (
          <AddressForm compact onCancel={addresses.length ? () => setAddingAddress(false) : undefined} onSaved={(address) => {
            setAddresses((current) => [...current, address]);
            setShippingPid(address.pid);
            setAddingAddress(false);
          }} />
        )}

        {addresses.some((address) => address.addressType === "billing") && (
          <div className="billing-choice">
            <label className="check-field"><input checked={sameAddress} onChange={(event) => setSameAddress(event.target.checked)} type="checkbox" /><span>Billing address is the same as delivery</span></label>
            {!sameAddress && <label className="form-field"><span>Billing address</span><select onChange={(event) => setBillingPid(event.target.value)} value={billingPid}>{addresses.filter((address) => address.addressType === "billing").map((address) => <option key={address.pid} value={address.pid}>{address.label ?? address.lineOne}</option>)}</select></label>}
          </div>
        )}
      </section>

      <aside className="order-summary checkout-summary">
        <p className="eyebrow">Your order</p>
        <h2>{items.length} {items.length === 1 ? "piece" : "pieces"}</h2>
        <ul>{items.map((item) => <li key={item.variantPid}><span>{item.productName} <small>× {item.quantity}</small></span><strong>{formatCurrency(Number(item.unitPrice) * item.quantity)}</strong></li>)}</ul>
        <dl>
          <div><dt>Subtotal</dt><dd>{quote && formatCurrency(quote.subtotal)}</dd></div>
          <div><dt>Shipping</dt><dd>{quote && formatCurrency(quote.shippingTotal)}</dd></div>
          <div><dt>Tax</dt><dd>{quote && formatCurrency(quote.taxTotal)}</dd></div>
          <div className="order-summary__total"><dt>Total</dt><dd>{quote && formatCurrency(quote.grandTotal)}</dd></div>
        </dl>
        <button className="button-primary order-summary__checkout" disabled={submitting || !shippingPid || !quote?.canCheckout} onClick={startCheckout} type="button">
          {submitting ? "Opening secure payment" : checkoutAttempt ? "Retry secure payment" : "Continue to payment"}<ArrowRightIcon aria-hidden size={17} />
        </button>
        <p className="order-summary__note">Payment is completed securely with Stripe.</p>
      </aside>
    </div>
  );
}
