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
  createdAt: string;
  updatedAt: string;
  deletedAt: string | null;
};

export type CategoryInput = {
  name: string;
  slug: string;
  imageLink: string;
  parentId?: number;
  description?: string;
};

export function createCategory(input: CategoryInput) {
  return apiRequest<Category>("/categories", {
    method: "POST",
    body: JSON.stringify(input),
    fallback: "Unable to create category",
  });
}
