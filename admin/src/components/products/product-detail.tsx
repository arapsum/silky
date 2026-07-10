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
  StackIcon,
  TagIcon,
  TrashIcon,
  WarningCircleIcon,
} from "@phosphor-icons/react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link, useNavigate } from "@tanstack/react-router";
import { toast } from "sonner";

import {
  deleteProduct,
  getProduct,
  productsQueryKey,
  type ProductDetail,
  type ProductPicture,
  type ProductVariantDetail,
} from "#/api/products.ts";
import { titleCase } from "#/components/catalogue/string-utils";
import { EmptyState } from "#/components/empty-state";
import { ErrorState } from "#/components/error-state";
import { PageHeader } from "#/components/page-header";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "#/components/ui/alert-dialog";
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
  if (!value) return "Not available";

  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value));
}

function productImages(product: ProductDetail) {
  return [...product.pictures, ...product.variants.flatMap((variant) => variant.pictures)];
}

function defaultVariant(product: ProductDetail) {
  return product.variants.find((variant) => variant.isDefault) ?? product.variants[0];
}

function totalStock(product: ProductDetail) {
  return product.variants.reduce((total, variant) => total + variant.stockQuantity, 0);
}

function stockStatus(variant: ProductVariantDetail) {
  if (variant.stockQuantity === 0) return "Out of stock";
  if (variant.stockQuantity < 10) return "Low stock";
  return "In stock";
}

function statusClass(status: string) {
  if (status === "Out of stock") {
    return "border-red-200 bg-red-50 text-red-700 dark:border-red-900 dark:bg-red-950/40 dark:text-red-300";
  }
  if (status === "Low stock") {
    return "border-amber-200 bg-amber-50 text-amber-700 dark:border-amber-900 dark:bg-amber-950/40 dark:text-amber-300";
  }
  return "border-emerald-200 bg-emerald-50 text-emerald-700 dark:border-emerald-900 dark:bg-emerald-950/40 dark:text-emerald-300";
}

function DetailField({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="min-w-0">
      <dt className="text-xs font-medium text-muted-foreground">{label}</dt>
      <dd className="mt-1 min-h-5 break-words text-sm font-medium">{children}</dd>
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
    <div className="flex min-w-0 flex-col-reverse gap-3 sm:flex-row">
      {pictures.length > 0 && (
        <div className="flex shrink-0 gap-2 overflow-x-auto pb-1 sm:max-h-[28rem] sm:flex-col sm:overflow-y-auto sm:pb-0">
          {pictures.map((picture, index) => {
            const isSelected = picture.pid === selectedPicture?.pid;

            return (
              <button
                key={picture.pid}
                type="button"
                aria-label={`View product image ${index + 1}`}
                aria-pressed={isSelected}
                className={cn(
                  "size-14 shrink-0 overflow-hidden border-2 bg-muted p-0.5 transition",
                  isSelected ? "border-primary" : "border-transparent opacity-65 hover:opacity-100",
                )}
                onClick={() => setSelectedPid(picture.pid)}
              >
                <img src={picture.imageLink} alt="" className="size-full object-cover" />
              </button>
            );
          })}
        </div>
      )}

      <div className="relative flex aspect-[4/3] min-w-0 flex-1 items-center justify-center overflow-hidden bg-muted/50 lg:aspect-square">
        {selectedPicture ? (
          <img
            src={selectedPicture.imageLink}
            alt={product.name}
            className="size-full object-cover"
            loading="eager"
          />
        ) : (
          <div className="flex flex-col items-center gap-3 text-muted-foreground">
            <PackageIcon className="size-10" />
            <span className="text-sm">No product image</span>
          </div>
        )}
      </div>
    </div>
  );
}

function ProductMetric({
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
    <div className="flex min-w-0 items-start gap-3 border-b px-4 py-4 last:border-b-0 sm:border-r sm:border-b-0 sm:first:pl-0 sm:last:border-r-0 sm:last:pr-0">
      <span className="flex size-9 shrink-0 items-center justify-center bg-primary/10 text-primary">
        {icon}
      </span>
      <div className="min-w-0">
        <p className="text-xs text-muted-foreground">{label}</p>
        <p className="mt-0.5 truncate text-lg font-semibold tracking-tight">{value}</p>
        <p className="mt-0.5 truncate text-[11px] text-muted-foreground">{detail}</p>
      </div>
    </div>
  );
}

function ProductHero({ product }: { product: ProductDetail }) {
  const images = productImages(product);
  const variant = defaultVariant(product);
  const stock = totalStock(product);
  const prices = product.variants.map((item) => Number(item.price)).filter(Number.isFinite);
  const lowestPrice = prices.length ? Math.min(...prices) : undefined;
  const highestPrice = prices.length ? Math.max(...prices) : undefined;
  const priceRange =
    lowestPrice === undefined
      ? "Not set"
      : lowestPrice === highestPrice
        ? money(String(lowestPrice))
        : `${money(String(lowestPrice))} to ${money(String(highestPrice))}`;

  return (
    <section className="border bg-card p-4 sm:p-5">
      <div className="grid gap-7 lg:grid-cols-[minmax(20rem,.92fr)_minmax(0,1.08fr)] lg:gap-10">
        <ProductGallery key={product.pid} product={product} pictures={images} />

        <div className="flex min-w-0 flex-col">
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant="outline" className="h-7 border-primary/25 bg-primary/5 text-primary">
              <TagIcon className="size-3.5" weight="fill" />
              {titleCase(product.category.name)}
            </Badge>
            <span className="font-mono text-xs text-muted-foreground">
              {variant?.sku ?? "SKU not set"}
            </span>
          </div>

          <p className="mt-5 max-w-2xl text-sm leading-6 text-muted-foreground">
            {product.description || "No product description has been added yet."}
          </p>

          <div className="mt-6 grid border-y sm:grid-cols-3">
            <ProductMetric
              icon={<TagIcon className="size-4" />}
              label="Price"
              value={priceRange}
              detail={variant ? `Default ${money(variant.price)}` : "No default price"}
            />
            <ProductMetric
              icon={<CubeIcon className="size-4" />}
              label="Inventory"
              value={number(stock)}
              detail="Units across all SKUs"
            />
            <ProductMetric
              icon={<StackIcon className="size-4" />}
              label="Variants"
              value={number(product.variants.length)}
              detail={`${number(product.options.length)} option dimensions`}
            />
          </div>

          <dl className="mt-auto grid grid-cols-2 gap-x-8 gap-y-5 pt-6 sm:grid-cols-3">
            <DetailField label="Product type">Simple product</DetailField>
            <DetailField label="Media">{number(images.length)} images</DetailField>
            <DetailField label="Category slug">{product.category.slug}</DetailField>
            <div className="text-xs text-muted-foreground sm:col-span-3">
              Created {dateTime(product.createdAt)}
              <span className="mx-2 text-border">|</span>
              Updated {dateTime(product.updatedAt)}
            </div>
          </dl>
        </div>
      </div>
    </section>
  );
}

function Panel({
  title,
  icon,
  children,
  className,
}: {
  title: string;
  icon: React.ReactNode;
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <section className={cn("border bg-card", className)}>
      <div className="flex items-center gap-2 border-b px-4 py-3.5">
        <span className="text-muted-foreground">{icon}</span>
        <h2 className="text-sm font-semibold">{title}</h2>
      </div>
      <div className="p-4">{children}</div>
    </section>
  );
}

function OptionSummary({ product }: { product: ProductDetail }) {
  if (!product.options.length) {
    return <p className="text-sm text-muted-foreground">No product options configured.</p>;
  }

  return (
    <div className="space-y-4">
      <p className="text-xs leading-5 text-muted-foreground">
        Variants are created from the option dimensions below.
      </p>
      {product.options.map((option) => {
        const values = [
          ...new Set(
            product.variants.flatMap((variant) =>
              variant.options
                .filter((entry) => entry.attributeId === option.attributeId)
                .map((entry) => entry.value),
            ),
          ),
        ];

        return (
          <div key={option.pid}>
            <div className="mb-2 flex items-center justify-between gap-3">
              <p className="text-sm font-medium">{titleCase(option.attributeName)}</p>
              <span className="text-xs text-muted-foreground">{values.length} values</span>
            </div>
            <div className="flex flex-wrap gap-1.5">
              {values.length ? (
                values.map((value) => (
                  <Badge key={value} variant="outline" className="h-7 px-2.5 font-normal">
                    {value}
                  </Badge>
                ))
              ) : (
                <span className="text-xs text-muted-foreground">No values assigned</span>
              )}
            </div>
          </div>
        );
      })}
    </div>
  );
}

function VariantTable({ variants }: { variants: ProductVariantDetail[] }) {
  if (!variants.length) {
    return (
      <EmptyState
        icon={<BarcodeIcon className="size-8" />}
        title="No variants"
        description="Create at least one SKU to sell this product."
        className="min-h-56 bg-muted/20"
      />
    );
  }

  return (
    <div className="overflow-x-auto">
      <table className="w-full min-w-[48rem] text-left text-sm">
        <thead className="border-y bg-muted/30 text-xs text-muted-foreground">
          <tr>
            <th className="px-3 py-3 font-medium">Variant</th>
            <th className="px-3 py-3 font-medium">Options</th>
            <th className="px-3 py-3 font-medium">SKU</th>
            <th className="px-3 py-3 text-right font-medium">Price</th>
            <th className="px-3 py-3 text-right font-medium">Stock</th>
            <th className="px-3 py-3 font-medium">Status</th>
          </tr>
        </thead>
        <tbody>
          {variants.map((variant) => {
            const image = variant.pictures[0];
            const status = variant.deletedAt ? "Deleted" : stockStatus(variant);

            return (
              <tr key={variant.pid} className="border-b last:border-b-0 hover:bg-muted/20">
                <td className="px-3 py-2.5">
                  <div className="flex size-9 items-center justify-center overflow-hidden bg-muted">
                    {image ? (
                      <img src={image.imageLink} alt="" className="size-full object-cover" />
                    ) : (
                      <PackageIcon className="size-4 text-muted-foreground" />
                    )}
                  </div>
                </td>
                <td className="px-3 py-2.5">
                  <div className="flex flex-wrap gap-1">
                    {variant.options.length ? (
                      variant.options.map((option) => (
                        <span key={option.pid} className="text-xs text-muted-foreground">
                          {option.value}
                        </span>
                      ))
                    ) : (
                      <span className="text-xs text-muted-foreground">Not configured</span>
                    )}
                  </div>
                </td>
                <td className="px-3 py-2.5 font-mono text-xs font-medium">{variant.sku}</td>
                <td className="px-3 py-2.5 text-right font-medium">{money(variant.price)}</td>
                <td className="px-3 py-2.5 text-right">{number(variant.stockQuantity)} units</td>
                <td className="px-3 py-2.5">
                  <Badge
                    variant="outline"
                    className={cn("h-5 px-1.5 text-[11px]", statusClass(status))}
                  >
                    {status}
                  </Badge>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

function ProductDetailSkeleton() {
  return (
    <div className="grid gap-6 pb-10">
      <div className="flex items-center justify-between">
        <Skeleton className="h-9 w-72 rounded-none" />
        <Skeleton className="h-9 w-36 rounded-none" />
      </div>
      <div className="grid gap-8 border p-5 lg:grid-cols-2">
        <Skeleton className="aspect-square w-full rounded-none" />
        <div className="grid content-center gap-5">
          <Skeleton className="h-5 w-3/5 rounded-none" />
          <Skeleton className="h-20 w-full rounded-none" />
          <Skeleton className="h-20 w-full rounded-none" />
        </div>
      </div>
      <div className="grid gap-5 lg:grid-cols-3">
        <Skeleton className="h-64 rounded-none" />
        <Skeleton className="h-64 rounded-none" />
        <Skeleton className="h-64 rounded-none" />
      </div>
      <Skeleton className="h-96 rounded-none" />
    </div>
  );
}

export default function ProductDetailPage({ pid }: { pid: string }) {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const productQuery = useQuery({
    queryKey: [...productsQueryKey, pid],
    queryFn: () => getProduct(pid),
  });
  const deleteMutation = useMutation({
    mutationFn: () => deleteProduct(pid),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: productsQueryKey });
      toast.success("Product deleted", { id: "delete-product-success" });
      await navigate({ to: "/products" });
    },
    onError: (error) => {
      toast.error(error.message, { id: "delete-product-error" });
    },
  });

  const product = productQuery.data;

  if (productQuery.isLoading) return <ProductDetailSkeleton />;

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
          <Button className="rounded-none" variant="outline" render={<Link to="/products" />}>
            Back to Products
          </Button>
        }
      />
    );
  }

  const images = productImages(product);
  const information = Object.entries(product.information);

  return (
    <div className="w-full pb-10">
      <PageHeader
        title={
          <>
            <span>{product.name}</span>
            <Badge
              variant="outline"
              className={cn(
                "h-6 px-2 text-xs",
                product.deletedAt
                  ? "border-red-200 bg-red-50 text-red-700 dark:border-red-900 dark:bg-red-950/40 dark:text-red-300"
                  : "border-emerald-200 bg-emerald-50 text-emerald-700 dark:border-emerald-900 dark:bg-emerald-950/40 dark:text-emerald-300",
              )}
            >
              {product.deletedAt ? (
                <WarningCircleIcon weight="fill" />
              ) : (
                <CheckCircleIcon weight="fill" />
              )}
              {product.deletedAt ? "Deleted" : "Active"}
            </Badge>
          </>
        }
        subtitle="Manage catalogue information, media, variants, pricing, and inventory."
        actions={
          <>
            <Button className="rounded-none" variant="outline" render={<Link to="/products" />}>
              <ArrowLeftIcon className="size-4" />
              Back
            </Button>
            <AlertDialog>
              <AlertDialogTrigger
                render={
                  <Button
                    className="rounded-none"
                    variant="outline"
                    disabled={deleteMutation.isPending || Boolean(product.deletedAt)}
                  />
                }
              >
                <TrashIcon className="size-4" />
                Delete
              </AlertDialogTrigger>
              <AlertDialogContent className="rounded-none">
                <AlertDialogHeader>
                  <AlertDialogTitle>Delete product?</AlertDialogTitle>
                  <AlertDialogDescription>
                    {product.name} will be removed from the active catalogue.
                  </AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                  <AlertDialogCancel className="rounded-none" disabled={deleteMutation.isPending}>
                    Cancel
                  </AlertDialogCancel>
                  <AlertDialogAction
                    variant="destructive"
                    className="rounded-none"
                    disabled={deleteMutation.isPending}
                    onClick={() => deleteMutation.mutate()}
                  >
                    Delete product
                  </AlertDialogAction>
                </AlertDialogFooter>
              </AlertDialogContent>
            </AlertDialog>
            <Button
              className="rounded-none"
              render={<Link to="/products/$pid/edit" params={{ pid }} />}
            >
              <PencilSimpleIcon className="size-4" />
              Edit product
            </Button>
          </>
        }
      />

      <ProductHero product={product} />

      <section className="mt-5 border bg-card">
        <div className="flex flex-col gap-3 border-b px-4 py-4 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <h2 className="text-base font-semibold">
              Variants ({number(product.variants.length)})
            </h2>
            <p className="mt-1 text-sm text-muted-foreground">
              Each option combination creates a unique sellable SKU.
            </p>
          </div>
          <div className="flex items-center gap-2 text-xs font-medium text-muted-foreground">
            <ImagesIcon className="size-4" />
            {number(images.length)} images
          </div>
        </div>
        <VariantTable variants={product.variants} />
      </section>

      <div className="mt-5 grid items-start gap-5 lg:grid-cols-[minmax(0,1.15fr)_minmax(18rem,.85fr)]">
        <div className="grid gap-5">
          <Panel title="Product information" icon={<StackIcon className="size-4" />}>
            {information.length ? (
              <dl className="grid gap-x-8 gap-y-5 sm:grid-cols-2">
                {information.map(([key, value]) => (
                  <DetailField key={key} label={key}>
                    {value}
                  </DetailField>
                ))}
              </dl>
            ) : (
              <p className="text-sm leading-6 text-muted-foreground">
                No additional product information has been added.
              </p>
            )}
          </Panel>
          <Panel title="Product description" icon={<PackageIcon className="size-4" />}>
            <p className="max-w-3xl text-sm leading-6 text-muted-foreground">
              {product.description || "No product description has been added yet."}
            </p>
          </Panel>
        </div>

        <div className="grid gap-5">
          <Panel title="Category information" icon={<TagIcon className="size-4" />}>
            <dl className="grid gap-4">
              <DetailField label="Category">{titleCase(product.category.name)}</DetailField>
              <DetailField label="Category slug">{product.category.slug}</DetailField>
              <DetailField label="Media">{number(images.length)} images</DetailField>
            </dl>
          </Panel>
          <Panel title="Product options" icon={<StackIcon className="size-4" />}>
            <OptionSummary product={product} />
          </Panel>
        </div>
      </div>
    </div>
  );
}
