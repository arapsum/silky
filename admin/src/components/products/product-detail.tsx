"use client";

import type React from "react";
import { useState } from "react";
import {
  ArrowLeftIcon,
  BarcodeIcon,
  CheckCircleIcon,
  CubeIcon,
  ImagesIcon,
  PackageIcon,
  PencilSimpleIcon,
  PlusIcon,
  StackIcon,
  TagIcon,
  WarningCircleIcon,
} from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";

import {
  getProduct,
  productsQueryKey,
  type ProductDetail,
  type ProductOption,
  type ProductPicture,
  type ProductVariantDetail,
} from "#/api/products.ts";
import { titleCase } from "#/components/catalogue/string-utils";
import { EmptyState } from "#/components/empty-state";
import { ErrorState } from "#/components/error-state";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Skeleton } from "#/components/ui/skeleton";
import { cn } from "#/lib/utils";

function money(value: string) {
  return Intl.NumberFormat(undefined, {
    style: "currency",
    currency: "USD",
  }).format(Number(value));
}

function number(value: number) {
  return Intl.NumberFormat().format(value);
}

function dateTime(value: string | null) {
  if (!value) return "N/A";

  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value));
}

function productImages(product: ProductDetail) {
  const images = [...product.pictures];

  for (const variant of product.variants) {
    images.push(...variant.pictures);
  }

  return images;
}

function defaultVariant(product: ProductDetail) {
  return product.variants.find((variant) => variant.isDefault) ?? product.variants[0];
}

function totalStock(product: ProductDetail) {
  return product.variants.reduce((total, variant) => total + variant.stockQuantity, 0);
}

function Field({ label, value }: { label: string; value: string | number }) {
  return (
    <div className="min-w-0">
      <dt className="text-xs font-medium tracking-wide text-muted-foreground uppercase">{label}</dt>
      <dd className="mt-1.5 break-words text-sm font-medium">{value}</dd>
    </div>
  );
}

function Timestamp({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center gap-1.5">
      <dt className="font-medium">{label}</dt>
      <dd>{value}</dd>
    </div>
  );
}

function Metric({
  icon,
  label,
  value,
  detail,
}: {
  icon: React.ReactNode;
  label: string;
  value: string;
  detail: string;
}) {
  return (
    <div className="flex min-w-0 items-start gap-3 border bg-background/80 p-4 shadow-sm backdrop-blur-sm">
      <div className="flex size-9 shrink-0 items-center justify-center bg-primary/10 text-primary">
        {icon}
      </div>
      <div className="min-w-0">
        <p className="text-xs font-medium text-muted-foreground">{label}</p>
        <p className="mt-0.5 truncate text-lg font-bold tracking-tight">{value}</p>
        <p className="mt-0.5 truncate text-xs text-muted-foreground">{detail}</p>
      </div>
    </div>
  );
}

function ProductGallery({
  product,
  pictures,
}: {
  product: ProductDetail;
  pictures: ProductPicture[];
}) {
  const [selectedPid, setSelectedPid] = useState(pictures[0]?.pid);
  const selectedPicture = pictures.find((picture) => picture.pid === selectedPid) ?? pictures[0];

  return (
    <div className="min-w-0">
      <div className="relative flex aspect-[4/3] items-center justify-center overflow-hidden border bg-muted/70">
        {selectedPicture ? (
          <img
            src={selectedPicture.imageLink}
            alt={product.name}
            className="h-full w-full object-cover transition-transform duration-500 hover:scale-[1.02]"
            loading="eager"
          />
        ) : (
          <div className="flex flex-col items-center gap-3 text-muted-foreground">
            <div className="flex size-16 items-center justify-center bg-background/80 shadow-sm">
              <PackageIcon className="size-8" />
            </div>
            <span className="text-sm font-medium">No product image</span>
          </div>
        )}

        <div className="absolute top-4 left-4">
          <Badge
            variant={product.deletedAt ? "destructive" : "outline"}
            className={cn(
              "h-7 bg-background/90 px-2.5 font-semibold backdrop-blur-sm",
              !product.deletedAt &&
                "border-emerald-200 text-emerald-700 dark:border-emerald-900 dark:text-emerald-300",
            )}
          >
            {product.deletedAt ? (
              <WarningCircleIcon className="size-3.5" weight="fill" />
            ) : (
              <CheckCircleIcon className="size-3.5" weight="fill" />
            )}
            {product.deletedAt ? "Deleted" : "Active"}
          </Badge>
        </div>
      </div>

      {pictures.length > 0 && (
        <div className="mt-3 flex gap-2 overflow-x-auto pb-1">
          {pictures.map((picture, index) => {
            const isSelected = picture.pid === selectedPicture?.pid;

            return (
              <button
                key={picture.pid}
                type="button"
                aria-label={`View product image ${index + 1}`}
                aria-pressed={isSelected}
                className={cn(
                  "size-16 shrink-0 overflow-hidden border-2 bg-muted p-0.5 transition",
                  isSelected
                    ? "border-primary shadow-sm"
                    : "border-transparent opacity-65 hover:opacity-100",
                )}
                onClick={() => setSelectedPid(picture.pid)}
              >
                <img src={picture.imageLink} alt="" className="h-full w-full object-cover" />
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}

function ProductHero({ product }: { product: ProductDetail }) {
  const images = productImages(product);
  const variant = defaultVariant(product);
  const stock = totalStock(product);

  return (
    <section className="overflow-hidden border bg-gradient-to-br from-card via-card to-muted/50 p-4 shadow-sm sm:p-6 lg:p-8">
      <div className="grid gap-8 lg:grid-cols-[minmax(20rem,0.9fr)_minmax(0,1.1fr)] lg:items-center">
        <ProductGallery key={product.pid} product={product} pictures={images} />

        <div className="min-w-0 lg:py-4">
          <div className="flex flex-wrap items-center gap-2 text-sm font-medium text-primary">
            <TagIcon className="size-4" weight="fill" />
            <span>{titleCase(product.category.name)}</span>
            <span className="text-muted-foreground">/</span>
            <span className="text-muted-foreground">{product.category.slug}</span>
          </div>

          <h1 className="mt-4 max-w-3xl text-3xl font-bold tracking-tight text-balance sm:text-4xl">
            {product.name}
          </h1>
          <p className="mt-4 max-w-2xl text-sm leading-6 text-muted-foreground sm:text-base">
            {product.description || "No description has been added for this product."}
          </p>

          <div className="mt-6 flex flex-wrap gap-2">
            <Badge
              variant="outline"
              className={cn(
                "h-7 px-2.5 font-semibold",
                stock > 0
                  ? "border-emerald-200 bg-emerald-50 text-emerald-700 dark:border-emerald-900 dark:bg-emerald-950/50 dark:text-emerald-300"
                  : "border-amber-200 bg-amber-50 text-amber-700 dark:border-amber-900 dark:bg-amber-950/50 dark:text-amber-300",
              )}
            >
              {stock > 0 ? (
                <CheckCircleIcon className="size-3.5" weight="fill" />
              ) : (
                <WarningCircleIcon className="size-3.5" weight="fill" />
              )}
              {stock > 0 ? `${number(stock)} units available` : "Out of stock"}
            </Badge>
            {variant && (
              <Badge variant="secondary" className="h-7 px-2.5">
                Default SKU · {variant.sku}
              </Badge>
            )}
          </div>

          <div className="mt-8 grid gap-3 sm:grid-cols-3">
            <Metric
              icon={<TagIcon className="size-4" weight="bold" />}
              label="Default price"
              value={variant ? money(variant.price) : "N/A"}
              detail={variant ? variant.sku : "No default variant"}
            />
            <Metric
              icon={<CubeIcon className="size-4" weight="bold" />}
              label="Inventory"
              value={number(stock)}
              detail={stock > 0 ? "Units across all SKUs" : "Restock required"}
            />
            <Metric
              icon={<StackIcon className="size-4" weight="bold" />}
              label="Variants"
              value={number(product.variants.length)}
              detail={`${number(product.options.length)} option dimensions`}
            />
          </div>

          <dl className="mt-6 grid grid-cols-2 gap-x-6 gap-y-5 border-t pt-6 sm:grid-cols-3">
            <Field label="Category" value={titleCase(product.category.name)} />
            <Field label="Category slug" value={product.category.slug} />
            <Field label="Media" value={`${number(images.length)} images`} />
          </dl>

          <dl className="mt-5 flex flex-wrap gap-x-5 gap-y-2 border-t pt-4 text-xs text-muted-foreground">
            <Timestamp label="Created" value={dateTime(product.createdAt)} />
            <Timestamp label="Updated" value={dateTime(product.updatedAt)} />
            {product.deletedAt && <Timestamp label="Deleted" value={dateTime(product.deletedAt)} />}
          </dl>
        </div>
      </div>
    </section>
  );
}

function Panel({
  title,
  description,
  action,
  children,
  className,
}: {
  title: string;
  description?: string;
  action?: React.ReactNode;
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <section className={cn("border bg-card shadow-sm", className)}>
      <div className="flex flex-col gap-3 border-b px-5 py-5 sm:flex-row sm:items-start sm:justify-between sm:px-6">
        <div>
          <h2 className="text-base font-semibold">{title}</h2>
          {description && <p className="mt-1 text-sm text-muted-foreground">{description}</p>}
        </div>
        {action}
      </div>
      <div className="p-5 sm:p-6">{children}</div>
    </section>
  );
}

function VariantOptions({ variant }: { variant: ProductVariantDetail }) {
  if (!variant.options.length) {
    return <span className="text-xs text-muted-foreground">No differentiating options</span>;
  }

  return (
    <div className="flex flex-wrap gap-2">
      {variant.options.map((option) => (
        <Badge
          key={option.pid}
          variant="secondary"
          className="h-auto border border-border px-2.5 py-1 font-normal"
        >
          <span className="text-muted-foreground">{titleCase(option.attributeName)}</span>
          <span className="font-semibold">{option.value}</span>
        </Badge>
      ))}
    </div>
  );
}

function VariantList({ variants }: { variants: ProductVariantDetail[] }) {
  if (!variants.length) {
    return (
      <EmptyState
        icon={<BarcodeIcon className="size-8" />}
        title="No variants"
        description="Create at least one SKU to sell this product."
        className="min-h-56 bg-muted/30"
      />
    );
  }

  return (
    <div className="grid gap-3">
      {variants.map((variant) => {
        const image = variant.pictures[0];

        return (
          <article
            key={variant.pid}
            className="grid gap-4 border bg-background p-4 transition-colors hover:border-foreground/20 sm:grid-cols-[4.5rem_minmax(0,1fr)_auto] sm:items-center"
          >
            <div className="flex aspect-square items-center justify-center overflow-hidden bg-muted">
              {image ? (
                <img src={image.imageLink} alt="" className="h-full w-full object-cover" />
              ) : (
                <PackageIcon className="size-5 text-muted-foreground" />
              )}
            </div>

            <div className="min-w-0">
              <div className="flex flex-wrap items-center gap-2">
                <h3 className="truncate font-mono text-sm font-semibold">{variant.sku}</h3>
                {variant.isDefault && <Badge variant="secondary">Default</Badge>}
                {variant.deletedAt && <Badge variant="destructive">Deleted</Badge>}
              </div>
              <div className="mt-2.5">
                <VariantOptions variant={variant} />
              </div>
              <p className="mt-2 text-xs text-muted-foreground">
                Updated {dateTime(variant.updatedAt)}
              </p>
            </div>

            <div className="flex items-center justify-between gap-6 border-t pt-4 sm:block sm:border-t-0 sm:border-l sm:py-1 sm:pl-6 sm:text-right">
              <div>
                <p className="text-xs text-muted-foreground">Price</p>
                <p className="mt-1 text-base font-bold">{money(variant.price)}</p>
              </div>
              <div className="sm:mt-3">
                <p className="text-xs text-muted-foreground">Inventory</p>
                <p
                  className={cn(
                    "mt-1 text-sm font-semibold",
                    variant.stockQuantity === 0 && "text-amber-600 dark:text-amber-400",
                  )}
                >
                  {number(variant.stockQuantity)} units
                </p>
              </div>
            </div>
          </article>
        );
      })}
    </div>
  );
}

function OptionsList({ options }: { options: ProductOption[] }) {
  if (!options.length) {
    return <p className="text-sm text-muted-foreground">No product options configured.</p>;
  }

  return (
    <div className="grid gap-3">
      {options.map((option) => (
        <div key={option.pid} className="flex items-center justify-between gap-4 bg-muted/50 p-3">
          <div className="flex min-w-0 items-center gap-3">
            <div className="flex size-9 shrink-0 items-center justify-center bg-background text-muted-foreground shadow-sm">
              <StackIcon className="size-4" />
            </div>
            <div className="min-w-0">
              <p className="truncate text-sm font-semibold">{titleCase(option.attributeName)}</p>
              <p className="mt-0.5 text-xs text-muted-foreground">
                {option.attributeDescription || "Variant dimension"}
              </p>
            </div>
          </div>
          <span className="shrink-0 text-xs text-muted-foreground">
            Order {option.displayOrder ?? "—"}
          </span>
        </div>
      ))}
    </div>
  );
}

function ProductSidebar({ product }: { product: ProductDetail }) {
  return (
    <aside className="grid content-start gap-5">
      <Panel
        title="Product options"
        description="Dimensions used to build this product's variants."
      >
        <OptionsList options={product.options} />
      </Panel>
    </aside>
  );
}

function ProductDetailSkeleton() {
  return (
    <div className="grid gap-6 pb-10">
      <div className="flex items-center justify-between">
        <Skeleton className="h-9 w-28" />
        <Skeleton className="h-9 w-56" />
      </div>
      <div className="grid gap-8 border p-6 lg:grid-cols-2">
        <Skeleton className="aspect-[4/3] w-full" />
        <div className="grid content-center gap-5">
          <Skeleton className="h-5 w-48" />
          <Skeleton className="h-12 w-4/5" />
          <Skeleton className="h-20 w-full" />
          <div className="grid gap-3 sm:grid-cols-3">
            <Skeleton className="h-24" />
            <Skeleton className="h-24" />
            <Skeleton className="h-24" />
          </div>
        </div>
      </div>
      <div className="grid gap-5 xl:grid-cols-[minmax(0,1.65fr)_minmax(20rem,0.75fr)]">
        <Skeleton className="h-96" />
        <Skeleton className="h-96" />
      </div>
    </div>
  );
}

export default function ProductDetailPage({ pid }: { pid: string }) {
  const productQuery = useQuery({
    queryKey: [...productsQueryKey, pid],
    queryFn: () => getProduct(pid),
  });

  const product = productQuery.data;

  if (productQuery.isLoading) {
    return <ProductDetailSkeleton />;
  }

  if (productQuery.isError) {
    return (
      <ErrorState
        title="Product could not be loaded"
        description={productQuery.error.message}
        onRetry={() => productQuery.refetch()}
      />
    );
  }

  if (!product) {
    return (
      <EmptyState
        title="Product not found"
        description="The requested product no longer exists or is unavailable."
        action={
          <Button variant="outline" render={<Link to="/products" />}>
            Back to Products
          </Button>
        }
      />
    );
  }

  return (
    <div className="w-full pb-10">
      <div className="mb-5 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <Button variant="ghost" className="w-fit px-0" render={<Link to="/products" />}>
          <ArrowLeftIcon className="size-4" />
          Products
        </Button>

        <div className="flex flex-wrap gap-2">
          <Button variant="outline" render={<Link to="/products/$pid/edit" params={{ pid }} />}>
            <PencilSimpleIcon className="size-4" />
            Edit product
          </Button>
          <Button render={<Link to="/products/create" />}>
            <PlusIcon className="size-4" />
            Add product
          </Button>
        </div>
      </div>

      <ProductHero product={product} />

      <div className="mt-6 grid gap-5 xl:grid-cols-[minmax(0,1.65fr)_minmax(20rem,0.75fr)] xl:items-start">
        <Panel
          title="Variants and inventory"
          description="Pricing, option combinations, and stock for every sellable SKU."
          action={
            <div className="flex items-center gap-2 text-xs font-medium text-muted-foreground">
              <ImagesIcon className="size-4" />
              {number(productImages(product).length)} images
            </div>
          }
        >
          <VariantList variants={product.variants} />
        </Panel>

        <ProductSidebar product={product} />
      </div>
    </div>
  );
}
