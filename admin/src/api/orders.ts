import { apiRequest } from "#/api/client.ts";
import type { Pagination } from "#/api/products.ts";

export const ordersQueryKey = ["orders"] as const;

export type Order = {
  pid: string;
  orderNumber: number;
  customerName: string;
  customerEmail: string;
  customerImage: string | null;
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

export type OrderAddressSnapshot = {
  recipientName?: string;
  company?: string | null;
  lineOne?: string;
  lineTwo?: string | null;
  city?: string;
  region?: string | null;
  postalCode?: string | null;
  countryCode?: string;
  email?: string | null;
  phone?: string | null;
};

export type OrderItem = {
  pid: string;
  productPid: string;
  variantPid: string;
  productName: string;
  sku: string;
  selectedOptions: Record<string, unknown>;
  quantity: number;
  unitPrice: string;
  discountTotal: string;
  taxTotal: string;
  lineTotal: string;
  createdAt: string;
  updatedAt: string;
};

export type OrderDetail = Order & {
  billingAddressSnapshot: OrderAddressSnapshot;
  shippingAddressSnapshot: OrderAddressSnapshot;
  customerNote: string | null;
  staffNote: string | null;
};

export type OrderWithItems = {
  order: OrderDetail;
  items: OrderItem[];
};

export type UpdateOrderInput = {
  status?: string;
  paymentStatus?: string;
  fulfillmentStatus?: string;
  staffNote?: string;
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

export function getOrder(pid: string) {
  return apiRequest<OrderWithItems>(`/orders/${pid}`, {
    fallback: "Order could not be loaded",
  });
}

export function updateOrder(pid: string, input: UpdateOrderInput) {
  return apiRequest<OrderDetail>(`/orders/${pid}`, {
    method: "PATCH",
    body: JSON.stringify(input),
    fallback: "Order could not be updated",
  });
}
