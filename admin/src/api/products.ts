import { apiRequest } from "#/api/client.ts";

export const productsQueryKey = ["products"] as const;
export const productAttributesQueryKey = ["products", "attributes"] as const;
export const productTagsQueryKey = ["products", "tags"] as const;

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
  attributeDescription: string | null;
  displayOrder: number | null;
  createdAt: string;
};

export type ProductVariantOption = {
  id: number;
  pid: string;
  attributeId: number;
  attributePid: string;
  attributeName: string;
  attributeDescription: string | null;
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
  information: Record<string, string>;
  tags: ProductTag[];
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
  information: Record<string, string>;
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
  mediaAssetPid?: string;
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
  information?: Record<string, string>;
  tagPids?: string[];
  pictures?: ProductPictureInput[];
  variants: ProductVariantInput[];
};

export type ProductUpdateInput = {
  categoryId?: number;
  name?: string;
  description?: string | null;
  information?: Record<string, string>;
};

export type ProductVariantUpdateInput = {
  sku?: string;
  price?: string;
  stockQuantity?: number;
};

export type ProductPictureUpdateInput = {
  displayOrder?: number | null;
};

export type ProductAttribute = {
  id: number;
  pid: string;
  name: string;
  description: string | null;
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

export type ProductTag = {
  id: number;
  pid: string;
  name: string;
  createdAt: string;
  updatedAt: string;
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

export function updateProduct(pid: string, input: ProductUpdateInput) {
  return apiRequest<ProductDetail>(`/products/${pid}`, {
    method: "PATCH",
    body: JSON.stringify(input),
    fallback: "Unable to update product",
  });
}

export function createProductVariant(pid: string, input: ProductVariantInput) {
  return apiRequest<ProductDetail>(`/products/${pid}/variants`, {
    method: "POST",
    body: JSON.stringify(input),
    fallback: "Unable to create variant",
  });
}

export function updateProductVariant(
  pid: string,
  variantPid: string,
  input: ProductVariantUpdateInput,
) {
  return apiRequest<ProductDetail>(`/products/${pid}/variants/${variantPid}`, {
    method: "PATCH",
    body: JSON.stringify(input),
    fallback: "Unable to update variant",
  });
}

export function deleteProductVariant(pid: string, variantPid: string) {
  return apiRequest<ProductDetail>(`/products/${pid}/variants/${variantPid}`, {
    method: "DELETE",
    fallback: "Unable to delete variant",
  });
}

export function setDefaultProductVariant(pid: string, variantPid: string) {
  return apiRequest<ProductDetail>(`/products/${pid}/variants/${variantPid}/default`, {
    method: "POST",
    fallback: "Unable to set default variant",
  });
}

export function addProductPicture(pid: string, input: ProductPictureInput) {
  return apiRequest<ProductPicture>(`/products/${pid}/pictures`, {
    method: "POST",
    body: JSON.stringify(input),
    fallback: "Unable to add product picture",
  });
}

export function updateProductPicture(
  pid: string,
  picturePid: string,
  input: ProductPictureUpdateInput,
) {
  return apiRequest<ProductPicture>(`/products/${pid}/pictures/${picturePid}`, {
    method: "PATCH",
    body: JSON.stringify(input),
    fallback: "Unable to update product picture",
  });
}

export function deleteProductPicture(pid: string, picturePid: string) {
  return apiRequest<void>(`/products/${pid}/pictures/${picturePid}`, {
    method: "DELETE",
    fallback: "Unable to delete product picture",
  });
}

export function addVariantPicture(pid: string, variantPid: string, input: ProductPictureInput) {
  return apiRequest<ProductPicture>(`/products/${pid}/variants/${variantPid}/pictures`, {
    method: "POST",
    body: JSON.stringify(input),
    fallback: "Unable to add variant picture",
  });
}

export function updateVariantPicture(
  pid: string,
  variantPid: string,
  picturePid: string,
  input: ProductPictureUpdateInput,
) {
  return apiRequest<ProductPicture>(
    `/products/${pid}/variants/${variantPid}/pictures/${picturePid}`,
    {
      method: "PATCH",
      body: JSON.stringify(input),
      fallback: "Unable to update variant picture",
    },
  );
}

export function deleteVariantPicture(pid: string, variantPid: string, picturePid: string) {
  return apiRequest<void>(`/products/${pid}/variants/${variantPid}/pictures/${picturePid}`, {
    method: "DELETE",
    fallback: "Unable to delete variant picture",
  });
}

export function listProductAttributes() {
  return apiRequest<ProductAttributeWithValues[]>("/products/attributes", {
    fallback: "Unable to load product attributes",
  });
}

export function listProductTags() {
  return apiRequest<ProductTag[]>("/products/tags", {
    fallback: "Unable to load product tags",
  });
}

export function createProductTag(name: string) {
  return apiRequest<ProductTag>("/products/tags", {
    method: "POST",
    body: JSON.stringify({ name }),
    fallback: "Unable to create product tag",
  });
}
