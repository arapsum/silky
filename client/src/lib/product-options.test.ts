import { describe, expect, it } from "vitest";
import type { ProductVariant, VariantOption } from "@/lib/api/types";
import { optionValueIsAvailable, resolveVariantSelection } from "@/lib/product-options";

const variants = [variant("white-42", 9, "White", "42"), variant("black-43", 7, "Black", "43")];

describe("product option selection", () => {
  it("switches the other option when the requested value belongs to another variant", () => {
    expect(
      resolveVariantSelection(variants, { Colour: "White", Size: "42" }, "Colour", "Black"),
    ).toEqual({ Colour: "Black", Size: "43" });

    expect(
      resolveVariantSelection(variants, { Colour: "White", Size: "42" }, "Size", "43"),
    ).toEqual({ Colour: "Black", Size: "43" });
  });

  it("preserves compatible selections when more than one variant contains the value", () => {
    const catalogue = [...variants, variant("black-42", 5, "Black", "42")];

    expect(
      resolveVariantSelection(catalogue, { Colour: "White", Size: "42" }, "Colour", "Black"),
    ).toEqual({ Colour: "Black", Size: "42" });
  });

  it("disables values that are only present on out-of-stock variants", () => {
    const catalogue = [...variants, variant("red-44", 0, "Red", "44")];

    expect(optionValueIsAvailable(catalogue, "Colour", "Black")).toBe(true);
    expect(optionValueIsAvailable(catalogue, "Colour", "Red")).toBe(false);
    expect(optionValueIsAvailable(catalogue, "Size", "44")).toBe(false);
  });
});

function variant(pid: string, stockQuantity: number, colour: string, size: string): ProductVariant {
  return {
    id: 1,
    pid,
    sku: pid.toUpperCase(),
    price: "89.99",
    stockQuantity,
    isDefault: pid === "white-42",
    options: [option("Colour", colour), option("Size", size)],
    pictures: [],
    createdAt: "2026-07-16T00:00:00Z",
    updatedAt: "2026-07-16T00:00:00Z",
  };
}

function option(attributeName: string, value: string): VariantOption {
  return {
    id: 1,
    pid: `${attributeName}-${value}`,
    attributeId: 1,
    attributePid: attributeName,
    attributeName,
    attributeDescription: null,
    attributeValueId: 1,
    attributeValuePid: value,
    value,
    createdAt: "2026-07-16T00:00:00Z",
  };
}
