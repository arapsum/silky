import { apiRequest } from "#/api/client.ts";

export const categoriesQueryKey = ["categories"] as const;

export type Category = {
  id: number;
  pid: string;
  name: string;
  slug: string;
  imageLink: string;
  description: string | null;
  parentId: number | null;
  productCount?: number;
  createdAt: string;
  updatedAt: string;
  deletedAt: string | null;
};

export type CategoryInput = {
  name: string;
  slug?: string;
  imageLink: string;
  parentId?: number;
  description?: string;
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

export function deleteCategory(pid: string) {
  return apiRequest<void>(`/categories/${pid}`, {
    method: "DELETE",
    fallback: "Unable to delete category",
  });
}
