import { ArrowRightIcon, CheckCircleIcon, SpinnerGapIcon, XCircleIcon } from "@phosphor-icons/react";
import { useEffect, useRef, useState } from "react";
import { ApiError, orderApi } from "@/lib/api/browser";
import type { CheckoutSession } from "@/lib/api/types";
import { useCartStore } from "@/stores/cart";
import { toast } from "@/lib/toast";

interface CheckoutStatusProps { orderPid: string; mode: "success" | "cancel" }

export function CheckoutStatus({ orderPid, mode }: CheckoutStatusProps) {
  const [session, setSession] = useState<CheckoutSession | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [cancelling, setCancelling] = useState(false);
  const paymentReported = useRef(false);
  const clear = useCartStore((state) => state.clear);
  const setPending = useCartStore((state) => state.setPendingCheckout);

  useEffect(() => {
    if (!orderPid) return;
    let active = true;
    let attempts = 0;
    const poll = async () => {
      try {
        const next = await orderApi.session(orderPid);
        if (!active) return;
        setSession(next);
        if (next.paymentStatus === "paid") {
          clear();
          if (!paymentReported.current) {
            paymentReported.current = true;
            toast.success("Payment confirmed", "Your Silk order is now confirmed.");
          }
          return;
        }
        attempts += 1;
        if (attempts < 12 && ["initiated", "session_created", "processing"].includes(next.attemptStatus)) {
          window.setTimeout(poll, 2000);
        }
      } catch (requestError) {
        if (active) {
          const message = requestError instanceof ApiError ? requestError.message : "Payment status could not be confirmed.";
          setError(message);
          toast.error("Could not confirm payment", message);
        }
      }
    };
    void poll();
    return () => { active = false; };
  }, [orderPid]);

  async function cancel() {
    setCancelling(true);
    setError(null);
    try {
      await orderApi.cancel(orderPid);
      setPending(null);
      toast.flash.success("Checkout cancelled", "Your bag is ready for another checkout.");
      window.location.assign("/cart");
    } catch (requestError) {
      const message = requestError instanceof ApiError ? requestError.message : "The checkout could not be cancelled.";
      setError(message);
      toast.error("Checkout not cancelled", message);
      setCancelling(false);
    }
  }

  const paid = session?.paymentStatus === "paid";
  const failed = session && ["failed", "expired", "cancelled"].includes(session.attemptStatus);
  const eyebrow = paid
    ? "Payment confirmed"
    : mode === "cancel"
      ? "Payment paused"
      : failed
        ? "Payment unsuccessful"
        : "Confirming payment";
  const title = paid
    ? "Your order is in."
    : mode === "cancel"
      ? "Nothing has been lost."
      : failed
        ? "Payment was not completed."
        : "We are checking with Stripe.";
  const description = paid
    ? "We have your payment and will keep you updated as your order moves."
    : mode === "cancel"
      ? "Your bag is intact. Resume the reserved checkout or cancel it before trying again."
      : failed
        ? "Your bag is still here. Review it and start a new checkout whenever you are ready."
        : "This usually takes only a few seconds. You can safely stay on this page.";
  return (
    <section className="checkout-status">
      {paid ? <CheckCircleIcon aria-hidden size={54} weight="thin" /> : failed || mode === "cancel" ? <XCircleIcon aria-hidden size={54} weight="thin" /> : <SpinnerGapIcon aria-hidden className="is-spinning" size={54} weight="thin" />}
      <p className="eyebrow">{eyebrow}</p>
      <h1>{title}</h1>
      <p>{description}</p>
      {error && <p className="form-error" role="alert">{error}</p>}
      <div>
        {paid && <a className="button-primary" href={`/account/orders/${orderPid}`}>View order <ArrowRightIcon aria-hidden size={17} /></a>}
        {!paid && session?.checkoutUrl && <a className="button-primary" href={session.checkoutUrl}>Resume payment <ArrowRightIcon aria-hidden size={17} /></a>}
        {!paid && session?.checkoutUrl && <button className="button-secondary" disabled={cancelling} onClick={cancel} type="button">{cancelling ? "Cancelling" : "Cancel checkout"}</button>}
        {!session?.checkoutUrl && !paid && <a className="button-secondary" href="/cart">Return to your bag</a>}
      </div>
    </section>
  );
}
