import { apiRequest } from "#/api/client.ts";
import type { Pagination } from "#/api/products.ts";

export const ordersQueryKey = ["orders"] as const;

export type Order = {
  pid: string;
  orderNumber: number;
  customerName: string;
  customerEmail: string;
  status: string;
  paymentStatus: string;
  fulfillmentStatus: string;
  currency: string;
  subtotal: string;
  discountTotal: string;
  shippingTotal: string;
  taxTotal: string;
  grandTotal: string;
  placedAt: string | null;
  cancelledAt: string | null;
  completedAt: string | null;
  createdAt: string;
  updatedAt: string;
};

export type OrderListParams = {
  page?: number;
  limit?: number;
  search?: string;
  status?: string;
  paymentStatus?: string;
  fulfillmentStatus?: string;
  createdFrom?: string;
  createdTo?: string;
};

export type PaginatedOrders = {
  data: Order[];
  pagination: Pagination;
};

function orderSearchParams(params: OrderListParams) {
  const search = new URLSearchParams();

  Object.entries(params).forEach(([key, value]) => {
    if (value === undefined || value === null || value === "") return;
    search.set(key, String(value));
  });

  const query = search.toString();
  return query ? `?${query}` : "";
}

export function listOrders(params: OrderListParams = {}) {
  return apiRequest<PaginatedOrders>(`/orders${orderSearchParams(params)}`, {
    fallback: "Orders could not be loaded",
  });
}
