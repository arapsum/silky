import type {
  Address,
  AddressInput,
  ApiErrorPayload,
  CartQuote,
  CheckoutResponse,
  CheckoutSession,
  LoginResponse,
  OrderDetail,
  OrderSummary,
  PaginatedResponse,
  UserSession,
} from "./types";

export class ApiError extends Error {
  constructor(
    message: string,
    readonly status: number,
    readonly code?: string,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

async function parse<T>(response: Response): Promise<T> {
  if (response.status === 204) return undefined as T;
  const payload = await response.json().catch(() => ({}));
  if (!response.ok) {
    const error = payload as ApiErrorPayload;
    throw new ApiError(error.error ?? "Silk could not complete that request.", response.status, error.code);
  }
  return payload as T;
}

export async function apiRequest<T>(path: string, init: RequestInit = {}, retry = true): Promise<T> {
  const response = await fetch(`/api${path}`, {
    ...init,
    credentials: "same-origin",
    headers: {
      Accept: "application/json",
      ...(init.body ? { "Content-Type": "application/json" } : {}),
      ...init.headers,
    },
  });

  if (response.status === 401 && retry && path !== "/auth/refresh") {
    const refreshed = await fetch("/api/auth/refresh", {
      method: "POST",
      credentials: "same-origin",
      headers: { Accept: "application/json" },
    });
    if (refreshed.ok) return apiRequest<T>(path, init, false);
  }
  return parse<T>(response);
}

export const sessionApi = {
  me: () => apiRequest<UserSession>("/auth/me"),
  login: (input: { email: string; password: string }) =>
    apiRequest<LoginResponse>("/auth/login", { method: "POST", body: JSON.stringify(input) }, false),
  register: (input: { name: string; email: string; password: string; confirmPassword: string }) =>
    apiRequest<{ message: string }>("/auth/register", { method: "POST", body: JSON.stringify(input) }, false),
  logout: () => apiRequest<void>("/auth/logout", { method: "POST" }, false),
  forgotPassword: (email: string) =>
    apiRequest<{ message: string }>("/auth/forgot-password", { method: "POST", body: JSON.stringify({ email }) }, false),
  resetPassword: (input: { token: string; password: string; confirmPassword: string }) =>
    apiRequest<{ message: string }>("/auth/reset-password", { method: "POST", body: JSON.stringify(input) }, false),
  verify: (token: string) => apiRequest<{ message: string }>(`/auth/verify/${encodeURIComponent(token)}`, {}, false),
  update: (input: { name: string; email: string; image?: string | null; mediaAssetPid?: string }) =>
    apiRequest<UserSession>("/auth/me", { method: "PATCH", body: JSON.stringify(input) }),
};

export const addressApi = {
  list: () => apiRequest<Address[]>("/addresses"),
  create: (input: AddressInput) =>
    apiRequest<Address>("/addresses", { method: "POST", body: JSON.stringify(input) }),
  update: (pid: string, input: AddressInput) =>
    apiRequest<Address>(`/addresses/${pid}`, { method: "PUT", body: JSON.stringify(input) }),
  remove: (pid: string) => apiRequest<void>(`/addresses/${pid}`, { method: "DELETE" }),
};

export const cartApi = {
  quote: (items: Array<{ variantPid: string; quantity: number }>) =>
    apiRequest<CartQuote>("/cart/quote", { method: "POST", body: JSON.stringify({ items }) }, false),
};

export const orderApi = {
  checkout: (input: {
    checkoutKey: string;
    shippingAddressPid: string;
    billingAddressPid?: string;
    customerNote?: string;
    items: Array<{ variantPid: string; quantity: number }>;
  }) => apiRequest<CheckoutResponse>("/orders/checkout", { method: "POST", body: JSON.stringify(input) }),
  list: (page = 1) => apiRequest<PaginatedResponse<OrderSummary>>(`/orders?page=${page}&limit=10`),
  one: (pid: string) => apiRequest<OrderDetail>(`/orders/${pid}`),
  session: (pid: string) => apiRequest<CheckoutSession>(`/orders/${pid}/checkout-session`),
  cancel: (pid: string) => apiRequest<void>(`/orders/${pid}/checkout-session/cancel`, { method: "POST" }),
};
