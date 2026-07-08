"use client";

import type React from "react";
import {
  ArrowLeftIcon,
  ImagesIcon,
  PackageIcon,
  PencilSimpleIcon,
  PlusIcon,
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

function optionSummary(options: ProductOption[]) {
  if (!options.length) return "No options";

  return options.map((option) => titleCase(option.attributeName)).join(", ");
}

function Field({ label, value }: { label: string; value: string | number }) {
  return (
    <div>
      <dt className="text-xs font-medium uppercase text-muted-foreground">{label}</dt>
      <dd className="mt-1 break-words text-sm font-medium">{value}</dd>
    </div>
  );
}

function StatusPill({
  children,
  tone,
}: {
  children: string;
  tone: "neutral" | "success" | "warn";
}) {
  return (
    <span
      className={cn(
        "inline-flex h-6 items-center border px-2 text-xs font-medium",
        tone === "success" && "border-emerald-200 bg-emerald-50 text-emerald-700",
        tone === "warn" && "border-amber-200 bg-amber-50 text-amber-700",
        tone === "neutral" && "border-border bg-muted text-muted-foreground",
      )}
    >
      {children}
    </span>
  );
}

function Section({
  title,
  description,
  children,
}: {
  title: string;
  description: string;
  children: React.ReactNode;
}) {
  return (
    <section className="grid grid-cols-1 gap-6 border-t py-8 lg:grid-cols-3">
      <div>
        <h2 className="text-base font-semibold">{title}</h2>
        <p className="mt-1 max-w-sm text-sm text-muted-foreground">{description}</p>
      </div>
      <div className="lg:col-span-2">{children}</div>
    </section>
  );
}

function ProductHero({ product }: { product: ProductDetail }) {
  const images = productImages(product);
  const primaryImage = images[0];
  const variant = defaultVariant(product);
  const stock = totalStock(product);

  return (
    <div className="grid grid-cols-1 gap-6 lg:grid-cols-[20rem_1fr]">
      <div className="flex aspect-square items-center justify-center overflow-hidden border bg-muted">
        {primaryImage ? (
          <img
            src={primaryImage.imageLink}
            alt=""
            className="h-full w-full object-cover"
            loading="eager"
          />
        ) : (
          <PackageIcon className="size-10 text-muted-foreground" />
        )}
      </div>

      <div className="min-w-0">
        <div className="flex flex-wrap items-center gap-2">
          <StatusPill tone={product.deletedAt ? "warn" : "success"}>
            {product.deletedAt ? "Deleted" : "Active"}
          </StatusPill>
          <StatusPill tone={stock > 0 ? "success" : "warn"}>
            {stock > 0 ? "In stock" : "Out of stock"}
          </StatusPill>
        </div>

        <h1 className="mt-4 text-2xl font-bold tracking-tight">{product.name}</h1>
        <p className="mt-2 max-w-3xl text-sm text-muted-foreground">
          {product.description || "No description has been added for this product."}
        </p>

        <dl className="mt-6 grid grid-cols-2 gap-5 md:grid-cols-4">
          <Field label="Category" value={titleCase(product.category.name)} />
          <Field label="Variants" value={number(product.variants.length)} />
          <Field label="Total stock" value={number(stock)} />
          <Field label="Default price" value={variant ? money(variant.price) : "N/A"} />
        </dl>
      </div>
    </div>
  );
}

function MediaGrid({ pictures }: { pictures: ProductPicture[] }) {
  if (!pictures.length) {
    return (
      <EmptyState
        icon={<ImagesIcon className="size-8" />}
        title="No product media"
        description="Product and variant images will appear here after upload."
        className="min-h-48"
      />
    );
  }

  return (
    <div className="grid grid-cols-2 gap-3 md:grid-cols-4">
      {pictures.map((picture) => (
        <div key={picture.pid} className="overflow-hidden border bg-muted">
          <div className="aspect-square">
            <img src={picture.imageLink} alt="" className="h-full w-full object-cover" />
          </div>
          <div className="border-t bg-background px-3 py-2 text-xs text-muted-foreground">
            Order {picture.displayOrder ?? "N/A"}
          </div>
        </div>
      ))}
    </div>
  );
}

function OptionsList({ options }: { options: ProductOption[] }) {
  if (!options.length) {
    return (
      <EmptyState
        title="No product options"
        description="This product does not define option dimensions such as colour or size."
        className="min-h-40"
      />
    );
  }

  return (
    <div className="grid gap-3">
      {options.map((option) => (
        <div key={option.pid} className="grid grid-cols-1 gap-3 border px-4 py-3 md:grid-cols-3">
          <Field label="Attribute" value={titleCase(option.attributeName)} />
          <Field label="Display order" value={option.displayOrder ?? "N/A"} />
          <Field label="Attribute PID" value={option.attributePid} />
        </div>
      ))}
    </div>
  );
}

function VariantOptions({ variant }: { variant: ProductVariantDetail }) {
  if (!variant.options.length) {
    return <span className="text-muted-foreground">No differentiating options</span>;
  }

  return (
    <div className="flex flex-wrap gap-2">
      {variant.options.map((option) => (
        <span
          key={option.pid}
          className="inline-flex items-center gap-1 border bg-muted px-2 py-1 text-xs"
        >
          <span className="text-muted-foreground">{titleCase(option.attributeName)}</span>
          <span className="font-medium">{option.value}</span>
        </span>
      ))}
    </div>
  );
}

function VariantList({ variants }: { variants: ProductVariantDetail[] }) {
  if (!variants.length) {
    return (
      <EmptyState
        title="No variants"
        description="Create at least one SKU to sell this product."
        className="min-h-40"
      />
    );
  }

  return (
    <div className="grid gap-4">
      {variants.map((variant) => (
        <div key={variant.pid} className="border bg-background">
          <div className="grid grid-cols-1 gap-4 border-b p-4 lg:grid-cols-[1fr_auto]">
            <div className="min-w-0">
              <div className="flex flex-wrap items-center gap-2">
                <h3 className="font-mono text-sm font-semibold">{variant.sku}</h3>
                {variant.isDefault && <StatusPill tone="neutral">Default</StatusPill>}
                <StatusPill tone={variant.stockQuantity > 0 ? "success" : "warn"}>
                  {variant.stockQuantity > 0 ? "In stock" : "Out of stock"}
                </StatusPill>
              </div>
              <div className="mt-3">
                <VariantOptions variant={variant} />
              </div>
            </div>

            <dl className="grid grid-cols-2 gap-5 text-right">
              <Field label="Price" value={money(variant.price)} />
              <Field label="Stock" value={number(variant.stockQuantity)} />
            </dl>
          </div>

          <div className="grid grid-cols-1 gap-4 p-4 lg:grid-cols-3">
            <Field label="Variant PID" value={variant.pid} />
            <Field label="Created" value={dateTime(variant.createdAt)} />
            <Field label="Updated" value={dateTime(variant.updatedAt)} />
          </div>

          {variant.pictures.length > 0 && (
            <div className="border-t p-4">
              <MediaGrid pictures={variant.pictures} />
            </div>
          )}
        </div>
      ))}
    </div>
  );
}

function ProductDetailSkeleton() {
  return (
    <div className="grid gap-6">
      <Skeleton className="h-10 w-64" />
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-[20rem_1fr]">
        <Skeleton className="aspect-square w-full" />
        <div className="grid content-start gap-4">
          <Skeleton className="h-8 w-80" />
          <Skeleton className="h-20 w-full" />
          <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
            <Skeleton className="h-16" />
            <Skeleton className="h-16" />
            <Skeleton className="h-16" />
            <Skeleton className="h-16" />
          </div>
        </div>
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
      <div className="mb-6 flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
        <Button variant="ghost" className="w-fit px-0" render={<Link to="/products" />}>
          <ArrowLeftIcon className="size-4" />
          Products
        </Button>

        <div className="flex flex-wrap gap-3">
          <Button variant="outline" disabled>
            <PencilSimpleIcon className="size-4" />
            Edit Product
          </Button>
          <Button className="bg-blue-600 hover:bg-blue-700" render={<Link to="/products/create" />}>
            <PlusIcon className="size-4" />
            Add Product
          </Button>
        </div>
      </div>

      <ProductHero product={product} />

      <Section
        title="Identifiers"
        description="Stable product IDs and catalogue references for administrative support."
      >
        <dl className="grid grid-cols-1 gap-5 md:grid-cols-2">
          <Field label="Internal ID" value={product.id} />
          <Field label="Public ID" value={product.pid} />
          <Field label="Category ID" value={product.category.id} />
          <Field label="Category PID" value={product.category.pid} />
        </dl>
      </Section>

      <Section
        title="Product options"
        description={`Variant dimensions used by this product: ${optionSummary(product.options)}.`}
      >
        <OptionsList options={product.options} />
      </Section>

      <Section
        title="Media"
        description="Product-level images and variant-specific media available to shoppers."
      >
        <MediaGrid pictures={productImages(product)} />
      </Section>

      <Section
        title="Variants"
        description="Each SKU with the options that differentiate it, plus pricing and stock."
      >
        <VariantList variants={product.variants} />
      </Section>

      <Section
        title="Audit"
        description="Creation, update, and soft-delete timestamps for this product."
      >
        <dl className="grid grid-cols-1 gap-5 md:grid-cols-3">
          <Field label="Created" value={dateTime(product.createdAt)} />
          <Field label="Updated" value={dateTime(product.updatedAt)} />
          <Field label="Deleted" value={dateTime(product.deletedAt)} />
        </dl>
      </Section>
    </div>
  );
}
