import { apiRequest } from "#/api/client.ts";

export const productsQueryKey = ["products"] as const;
export const productAttributesQueryKey = ["products", "attributes"] as const;

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
