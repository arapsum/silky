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

export interface ProductTag {
  id: number;
  pid: string;
  name: string;
  createdAt: string;
  updatedAt: string;
}

export interface ProductPicture {
  id: number;
  pid: string;
  imageLink: string;
  displayOrder: number | null;
  createdAt: string;
  updatedAt: string;
}

export interface ProductOption {
  id: number;
  pid: string;
  attributeId: number;
  attributePid: string;
  attributeName: string;
  attributeDescription: string | null;
  displayOrder: number | null;
  createdAt: string;
}

export interface VariantOption {
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
}

export interface ProductVariant {
  id: number;
  pid: string;
  sku: string;
  price: string;
  stockQuantity: number;
  isDefault: boolean;
  options: VariantOption[];
  pictures: ProductPicture[];
  createdAt: string;
  updatedAt: string;
}

export interface ProductDetail {
  id: number;
  pid: string;
  slug: string;
  name: string;
  description: string | null;
  information: Record<string, string>;
  tags: ProductTag[];
  category: ProductCategorySummary;
  pictures: ProductPicture[];
  options: ProductOption[];
  variants: ProductVariant[];
  createdAt: string;
  updatedAt: string;
}

export interface ApiErrorPayload {
  error?: string;
  code?: string;
}
