import { CheckIcon, MinusIcon, PlusIcon, ShoppingBagIcon } from "@phosphor-icons/react";
import { useMemo, useState } from "react";
import type { ProductDetail, ProductVariant } from "@/lib/api/types";
import { formatCurrency, titleCase } from "@/lib/format";
import { useCartStore } from "@/stores/cart";

interface ProductPurchaseProps {
  product: ProductDetail;
  fallbackImage: string | null;
}

function selectionsFor(variant: ProductVariant | undefined) {
  return Object.fromEntries(variant?.options.map((option) => [option.attributeName, option.value]) ?? []);
}

export function ProductPurchase({ product, fallbackImage }: ProductPurchaseProps) {
  const defaultVariant = product.variants.find((variant) => variant.isDefault) ?? product.variants[0];
  const [selections, setSelections] = useState<Record<string, string>>(() => selectionsFor(defaultVariant));
  const [quantity, setQuantity] = useState(1);
  const [added, setAdded] = useState(false);
  const add = useCartStore((state) => state.add);
  const optionGroups = useMemo(() => {
    const groups = new Map<string, string[]>();
    for (const variant of product.variants) {
      for (const option of variant.options) {
        const values = groups.get(option.attributeName) ?? [];
        if (!values.includes(option.value)) values.push(option.value);
        groups.set(option.attributeName, values);
      }
    }
    return [...groups.entries()];
  }, [product.variants]);
  const variant = product.variants.find((candidate) =>
    candidate.options.every((option) => selections[option.attributeName] === option.value),
  );
  const available = (variant?.stockQuantity ?? 0) > 0;

  function optionIsAvailable(name: string, value: string) {
    return product.variants.some(
      (candidate) =>
        candidate.stockQuantity > 0 &&
        candidate.options.every((option) =>
          option.attributeName === name
            ? option.value === value
            : !selections[option.attributeName] || selections[option.attributeName] === option.value,
        ),
    );
  }

  function addToCart() {
    if (!variant || !available) return;
    const image = variant.pictures[0]?.imageLink ?? fallbackImage;
    add({
      variantPid: variant.pid,
      productPid: product.pid,
      productSlug: product.slug,
      productName: product.name,
      sku: variant.sku,
      imageUrl: image,
      selectedOptions: selectionsFor(variant),
      unitPrice: variant.price,
      quantity,
      availableQuantity: variant.stockQuantity,
    });
    setAdded(true);
    window.setTimeout(() => setAdded(false), 2200);
  }

  return (
    <div className="purchase-panel">
      <p className="eyebrow">{titleCase(product.category.name)}</p>
      <h1>{product.name}</h1>
      <p className="purchase-panel__price">
        {variant ? formatCurrency(variant.price) : "Choose your options"}
      </p>
      {product.description && <p className="purchase-panel__description">{product.description}</p>}

      {optionGroups.map(([name, values]) => (
        <fieldset className="purchase-panel__options" key={name}>
          <legend>{titleCase(name)}</legend>
          <div>
            {values.map((value) => {
              const selected = selections[name] === value;
              return (
                <button
                  aria-pressed={selected}
                  className={selected ? "is-selected" : ""}
                  disabled={!optionIsAvailable(name, value)}
                  key={value}
                  onClick={() => {
                    setSelections((current) => ({ ...current, [name]: value }));
                    setQuantity(1);
                  }}
                  type="button"
                >
                  {selected && <CheckIcon aria-hidden size={13} weight="bold" />}
                  {value}
                </button>
              );
            })}
          </div>
        </fieldset>
      ))}

      <div className="purchase-panel__actions">
        <div className="quantity-control" aria-label="Quantity selector">
          <button aria-label="Reduce quantity" disabled={quantity <= 1} onClick={() => setQuantity((value) => value - 1)} type="button">
            <MinusIcon aria-hidden size={15} />
          </button>
          <output aria-live="polite">{quantity}</output>
          <button aria-label="Increase quantity" disabled={!variant || quantity >= variant.stockQuantity} onClick={() => setQuantity((value) => value + 1)} type="button">
            <PlusIcon aria-hidden size={15} />
          </button>
        </div>
        <button className="button-primary purchase-panel__add" disabled={!available} onClick={addToCart} type="button">
          {added ? <CheckIcon aria-hidden size={18} weight="bold" /> : <ShoppingBagIcon aria-hidden size={18} />}
          {added ? "Added to bag" : available ? "Add to bag" : "Unavailable"}
        </button>
      </div>
      <p className={`purchase-panel__availability ${available ? "" : "is-out"}`}>
        {available && variant ? `${variant.stockQuantity} available` : "This combination is unavailable"}
      </p>
    </div>
  );
}
