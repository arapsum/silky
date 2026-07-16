import type { ProductVariant } from "@/lib/api/types";

export function selectionsForVariant(variant: ProductVariant | undefined) {
  return Object.fromEntries(
    variant?.options.map((option) => [option.attributeName, option.value]) ?? [],
  );
}

export function optionValueIsAvailable(
  variants: ProductVariant[],
  attributeName: string,
  value: string,
) {
  return variants.some(
    (variant) =>
      variant.stockQuantity > 0 &&
      variant.options.some(
        (option) => option.attributeName === attributeName && option.value === value,
      ),
  );
}

export function resolveVariantSelection(
  variants: ProductVariant[],
  current: Record<string, string>,
  attributeName: string,
  value: string,
) {
  const compatible = variants.filter(
    (variant) =>
      variant.stockQuantity > 0 &&
      variant.options.some(
        (option) => option.attributeName === attributeName && option.value === value,
      ),
  );

  const bestMatch = compatible.reduce<ProductVariant | undefined>((best, candidate) => {
    if (!best) return candidate;

    const score = selectionScore(candidate, current, attributeName);
    const bestScore = selectionScore(best, current, attributeName);
    return score > bestScore ? candidate : best;
  }, undefined);

  return bestMatch ? selectionsForVariant(bestMatch) : current;
}

function selectionScore(
  variant: ProductVariant,
  current: Record<string, string>,
  changedAttribute: string,
) {
  return variant.options.reduce(
    (score, option) =>
      option.attributeName !== changedAttribute && current[option.attributeName] === option.value
        ? score + 1
        : score,
    0,
  );
}
