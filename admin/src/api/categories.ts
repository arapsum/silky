import { apiRequest } from "#/api/client.ts";

export const categoriesQueryKey = ["categories"] as const;

export type Category = {
  id: number;
  pid: string;
  name: string;
  slug: string;
  imageLink: string;
  mediaAssetPid?: string;
  description: string | null;
  parentId: number | null;
  parentName?: string | null;
  productCount?: number;
  createdAt: string;
  updatedAt: string;
  deletedAt: string | null;
};

export type CategoryInput = {
  name: string;
  slug?: string;
  imageLink: string;
  mediaAssetPid?: string;
  parentId?: number;
  description?: string;
};

export type UpdateCategoryInput = {
  name?: string;
  slug?: string;
  imageLink?: string;
  mediaAssetPid?: string;
  parentId?: number;
  clearParent?: boolean;
  description?: string;
};

export type CategoryChild = Pick<Category, "id" | "pid" | "name" | "slug" | "imageLink"> & {
  productCount: number;
};

export type CategoryAttribute = {
  id: number;
  pid: string;
  name: string;
  description: string | null;
};

export type CategoryTopProduct = {
  pid: string;
  name: string;
  imageLink: string | null;
  sku: string | null;
  stockQuantity: number;
};

export type CategoryDetail = {
  category: Category;
  children: CategoryChild[];
  attributes: CategoryAttribute[];
  topProducts: CategoryTopProduct[];
  totalVariants: number;
  totalStock: number;
};

export type Pagination = {
  page: number;
  limit: number;
  totalItems: number;
  totalPages: number;
  hasNext: boolean;
  hasPrev: boolean;
};

export type PaginatedCategories = {
  data: Category[];
  pagination: Pagination;
};

export type CategoryListParams = {
  limit?: number;
  page?: number;
  search?: string;
  name?: string;
  slug?: string;
  parentId?: number;
  hasParent?: boolean;
  includeDeleted?: boolean;
};

function categorySearchParams(params: CategoryListParams) {
  const search = new URLSearchParams();

  Object.entries(params).forEach(([key, value]) => {
    if (value === undefined || value === null || value === "") return;
    search.set(key, String(value));
  });

  const query = search.toString();

  return query ? `?${query}` : "";
}

export function listCategories(params: CategoryListParams = {}) {
  return apiRequest<PaginatedCategories>(`/categories${categorySearchParams(params)}`, {
    fallback: "Unable to load categories",
  });
}

export function createCategory(input: CategoryInput) {
  return apiRequest<Category>("/categories", {
    method: "POST",
    body: JSON.stringify(input),
    fallback: "Unable to create category",
  });
}

export function getCategory(pid: string) {
  return apiRequest<Category>(`/categories/${pid}`, {
    fallback: "Unable to load category",
  });
}

export function updateCategory(pid: string, input: UpdateCategoryInput) {
  return apiRequest<Category>(`/categories/${pid}`, {
    method: "PATCH",
    body: JSON.stringify(input),
    fallback: "Unable to update category",
  });
}

export function getCategoryDetail(pid: string) {
  return apiRequest<CategoryDetail>(`/categories/${pid}/detail`, {
    fallback: "Unable to load category details",
  });
}

export function updateCategoryAttributes(pid: string, attributePids: string[]) {
  return apiRequest<CategoryDetail>(`/categories/${pid}/attributes`, {
    method: "PUT",
    body: JSON.stringify({ attributePids }),
    fallback: "Unable to update category attributes",
  });
}

export function deleteCategory(pid: string) {
  return apiRequest<void>(`/categories/${pid}`, {
    method: "DELETE",
    fallback: "Unable to delete category",
  });
}
