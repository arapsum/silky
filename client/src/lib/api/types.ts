export interface Pagination {
  page: number;
  limit: number;
  totalItems: number;
  totalPages: number;
  hasNext: boolean;
  hasPrev: boolean;
}

export interface PaginatedResponse<T> {
  data: T[];
  pagination: Pagination;
}

export interface ProductCategorySummary {
  id: number;
  pid: string;
  name: string;
  slug: string;
}

export interface ProductVariantSummary {
  pid: string;
  sku: string;
  price: string;
  stockQuantity: number;
}

export interface ProductListItem {
  pid: string;
  slug: string;
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
}

export interface CategoryListItem {
  id: number;
  pid: string;
  name: string;
  slug: string;
  imageLink: string;
  description: string | null;
  parentId: number | null;
  parentName: string | null;
  productCount: number;
}

export interface ApiErrorPayload {
  error?: string;
  code?: string;
}
