import { apiRequest } from "#/api/client.ts";

export const productsQueryKey = ["products"] as const;
export const productAttributesQueryKey = ["products", "attributes"] as const;

export type Pagination = {
  page: number;
  limit: number;
  totalItems: number;
  totalPages: number;
  hasNext: boolean;
  hasPrev: boolean;
};

export type ProductCategorySummary = {
  id: number;
  pid: string;
  name: string;
  slug: string;
};

export type ProductVariantSummary = {
  pid: string;
  sku: string;
  price: string;
  stockQuantity: number;
};

export type ProductPicture = {
  id: number;
  pid: string;
  imageLink: string;
  displayOrder: number | null;
  createdAt: string;
  updatedAt: string;
};

export type ProductOption = {
  id: number;
  pid: string;
  attributeId: number;
  attributePid: string;
  attributeName: string;
  displayOrder: number | null;
  createdAt: string;
};

export type ProductVariantOption = {
  id: number;
  pid: string;
  attributeId: number;
  attributePid: string;
  attributeName: string;
  attributeValueId: number;
  attributeValuePid: string;
  value: string;
  createdAt: string;
};

export type ProductVariantDetail = {
  id: number;
  pid: string;
  sku: string;
  price: string;
  stockQuantity: number;
  isDefault: boolean;
  options: ProductVariantOption[];
  pictures: ProductPicture[];
  createdAt: string;
  updatedAt: string;
  deletedAt: string | null;
};

export type ProductDetail = {
  id: number;
  pid: string;
  name: string;
  description: string | null;
  category: ProductCategorySummary;
  pictures: ProductPicture[];
  options: ProductOption[];
  variants: ProductVariantDetail[];
  createdAt: string;
  updatedAt: string;
  deletedAt: string | null;
};

export type ProductListItem = {
  pid: string;
  name: string;
  description: string | null;
  category: ProductCategorySummary;
  primaryImage: string | null;
  defaultVariant: ProductVariantSummary | null;
  variantCount: number;
  optionCount: number;
  totalStock: number;
  createdAt: string;
  updatedAt: string;
  deletedAt: string | null;
};

export type PaginatedProducts = {
  data: ProductListItem[];
  pagination: Pagination;
};

export type ProductListParams = {
  limit?: number;
  page?: number;
  search?: string;
  name?: string;
  categoryId?: number;
  categorySlug?: string;
  sku?: string;
  minPrice?: string;
  maxPrice?: string;
  stockStatus?: "inStock" | "outOfStock";
  includeDeleted?: boolean;
};

export type ProductPictureInput = {
  imageLink: string;
  displayOrder?: number;
};

export type ProductVariantOptionInput = {
  attributeId: number;
  attributeValueId: number;
  displayOrder?: number;
};

export type ProductVariantInput = {
  sku: string;
  price: string;
  stockQuantity: number;
  isDefault: boolean;
  options?: ProductVariantOptionInput[];
  pictures?: ProductPictureInput[];
};

export type ProductInput = {
  categoryId: number;
  name: string;
  description?: string;
  pictures?: ProductPictureInput[];
  variants: ProductVariantInput[];
};

export type ProductAttribute = {
  id: number;
  pid: string;
  name: string;
  createdAt: string;
  updatedAt: string;
};

export type ProductAttributeValue = {
  id: number;
  pid: string;
  attributeId: number;
  value: string;
  createdAt: string;
  updatedAt: string;
};

export type ProductAttributeWithValues = {
  attribute: ProductAttribute;
  values: ProductAttributeValue[];
};

function productSearchParams(params: ProductListParams) {
  const search = new URLSearchParams();

  Object.entries(params).forEach(([key, value]) => {
    if (value === undefined || value === null || value === "") return;
    search.set(key, String(value));
  });

  const query = search.toString();

  return query ? `?${query}` : "";
}

export function listProducts(params: ProductListParams = {}) {
  return apiRequest<PaginatedProducts>(`/products${productSearchParams(params)}`, {
    fallback: "Unable to load products",
  });
}

export function getProduct(pid: string) {
  return apiRequest<ProductDetail>(`/products/${pid}`, {
    fallback: "Unable to load product",
  });
}

export function deleteProduct(pid: string) {
  return apiRequest<void>(`/products/${pid}`, {
    method: "DELETE",
    fallback: "Unable to delete product",
  });
}

export function createProduct(input: ProductInput) {
  return apiRequest<unknown>("/products", {
    method: "POST",
    body: JSON.stringify(input),
    fallback: "Unable to create product",
  });
}

export function listProductAttributes() {
  return apiRequest<ProductAttributeWithValues[]>("/products/attributes", {
    fallback: "Unable to load product attributes",
  });
}
