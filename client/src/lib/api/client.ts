import type {
  ApiErrorPayload,
  CategoryListItem,
  PaginatedResponse,
  ProductDetail,
  ProductListItem,
} from "./types";

const DEFAULT_API_URL = "http://127.0.0.1:7150/api";

export class StorefrontApiError extends Error {
  readonly status: number;
  readonly code?: string;

  constructor(message: string, status: number, code?: string) {
    super(message);
    this.name = "StorefrontApiError";
    this.status = status;
    this.code = code;
  }
}

function getApiUrl(): string {
  return (import.meta.env.API_URL ?? DEFAULT_API_URL).replace(/\/$/, "");
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${getApiUrl()}${path}`, {
    ...init,
    headers: {
      Accept: "application/json",
      ...init?.headers,
    },
  });

  if (!response.ok) {
    const payload = await response
      .json()
      .catch(() => ({})) as ApiErrorPayload;
    throw new StorefrontApiError(
      payload.error ?? "Silk could not load this content right now.",
      response.status,
      payload.code,
    );
  }

  return response.json() as Promise<T>;
}

export function getProducts(params: URLSearchParams = new URLSearchParams()) {
  const query = params.size > 0 ? `?${params.toString()}` : "";
  return request<PaginatedResponse<ProductListItem>>(`/products${query}`);
}

export function getCategories(params: URLSearchParams = new URLSearchParams()) {
  const query = params.size > 0 ? `?${params.toString()}` : "";
  return request<PaginatedResponse<CategoryListItem>>(`/categories${query}`);
}

export function getProductBySlug(slug: string) {
  return request<ProductDetail>(`/products/by-slug/${encodeURIComponent(slug)}`);
}
