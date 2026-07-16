import { ArrowLeftIcon, ArrowRightIcon } from "@phosphor-icons/react";
import { useEffect, useState } from "react";
import { ApiError, orderApi } from "@/lib/api/browser";
import { OrderHistorySkeleton } from "@/components/loading/StorefrontSkeletons";
import type { OrderSummary, Pagination } from "@/lib/api/types";
import { formatCurrency } from "@/lib/format";

export function OrderHistory() {
  const [orders, setOrders] = useState<OrderSummary[]>([]);
  const [pagination, setPagination] = useState<Pagination | null>(null);
  const [page, setPage] = useState(1);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    setLoading(true);
    orderApi.list(page).then((result) => { setOrders(result.data); setPagination(result.pagination); setError(null); }).catch((requestError) => setError(requestError instanceof ApiError ? requestError.message : "Orders could not be loaded.")).finally(() => setLoading(false));
  }, [page]);
  return (
    <div>
      <header className="account-header"><p className="eyebrow">Order history</p><h1>Every Silk order.</h1><p>Track payment, fulfilment, and the pieces in each order.</p></header>
      {error && <p className="form-error" role="alert">{error}</p>}
      {loading ? <OrderHistorySkeleton /> : orders.length ? <div className="order-list">{orders.map((order) => <a href={`/account/orders/${order.pid}`} key={order.pid}><span><strong>ORD-{String(order.orderNumber).padStart(6, "0")}</strong><small>{new Intl.DateTimeFormat("en", { dateStyle: "medium" }).format(new Date(order.createdAt))}</small></span><span><small>Payment</small><span className="status-pill">{order.paymentStatus}</span></span><span><small>Order</small><span className="status-pill">{order.status}</span></span><strong>{formatCurrency(order.grandTotal, order.currency)}</strong><ArrowRightIcon aria-hidden size={16} /></a>)}</div> : <p className="empty-copy">No orders yet. Your first one will appear here.</p>}
      {pagination && pagination.totalPages > 1 && <nav className="account-pagination" aria-label="Order pages"><button disabled={!pagination.hasPrev} onClick={() => setPage((value) => value - 1)} type="button"><ArrowLeftIcon aria-hidden size={14} /> Previous</button><span>Page {pagination.page} of {pagination.totalPages}</span><button disabled={!pagination.hasNext} onClick={() => setPage((value) => value + 1)} type="button">Next <ArrowRightIcon aria-hidden size={14} /></button></nav>}
    </div>
  );
}
