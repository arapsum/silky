import { ArrowLeftIcon } from "@phosphor-icons/react";
import { useEffect, useState } from "react";
import { ApiError, orderApi } from "@/lib/api/browser";
import type { OrderDetail } from "@/lib/api/types";
import { formatCurrency } from "@/lib/format";

export function OrderView({ pid }: { pid: string }) {
  const [detail, setDetail] = useState<OrderDetail | null>(null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => { orderApi.one(pid).then(setDetail).catch((requestError) => setError(requestError instanceof ApiError ? requestError.message : "This order could not be loaded.")); }, [pid]);
  if (!detail && !error) return <div className="account-state">Loading order...</div>;
  if (!detail) return <div className="account-state"><h1>Order unavailable.</h1><p>{error}</p></div>;
  const { order, items } = detail;
  const address = order.shippingAddressSnapshot;
  return (
    <div>
      <a className="checkout-back" href="/account/orders"><ArrowLeftIcon aria-hidden size={15} /> All orders</a>
      <header className="account-header account-header--action"><div><p className="eyebrow">Order details</p><h1>ORD-{String(order.orderNumber).padStart(6, "0")}</h1><p>Placed {new Intl.DateTimeFormat("en", { dateStyle: "long" }).format(new Date(order.createdAt))}</p></div><div className="order-statuses"><span className="status-pill">{order.status}</span><span className="status-pill">{order.paymentStatus}</span><span className="status-pill">{order.fulfillmentStatus}</span></div></header>
      <section className="account-panel order-items"><div className="account-panel__heading"><div><p className="eyebrow">Pieces</p><h2>{items.length} {items.length === 1 ? "item" : "items"}</h2></div></div>{items.map((item) => <article key={item.pid}>{item.imageUrl ? <img alt="" height="180" src={item.imageUrl} width="144" /> : <div className="order-item__placeholder">Silk</div>}<div><a href={item.productSlug ? `/products/${item.productSlug}` : undefined}>{item.productName}</a><p>{Object.entries(item.selectedOptions).map(([name, value]) => `${name}: ${value}`).join(" · ")}</p><small>{item.sku} · Qty {item.quantity}</small></div><strong>{formatCurrency(item.lineTotal, order.currency)}</strong></article>)}</section>
      <div className="order-detail-grid"><section className="account-panel"><div className="account-panel__heading"><div><p className="eyebrow">Delivery</p><h2>Shipping address</h2></div></div><p className="address-copy">{String(address.recipientName ?? "")}<br />{String(address.lineOne ?? "")}{address.lineTwo ? <><br />{String(address.lineTwo)}</> : null}<br />{String(address.city ?? "")}{address.region ? `, ${String(address.region)}` : ""}<br />{String(address.countryCode ?? "")}</p></section><section className="account-panel totals-panel"><div className="account-panel__heading"><div><p className="eyebrow">Payment</p><h2>Order total</h2></div></div><dl><div><dt>Subtotal</dt><dd>{formatCurrency(order.subtotal, order.currency)}</dd></div><div><dt>Shipping</dt><dd>{formatCurrency(order.shippingTotal, order.currency)}</dd></div><div><dt>Tax</dt><dd>{formatCurrency(order.taxTotal, order.currency)}</dd></div><div><dt>Total</dt><dd>{formatCurrency(order.grandTotal, order.currency)}</dd></div></dl></section></div>
    </div>
  );
}
