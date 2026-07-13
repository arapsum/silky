"use client";

import type React from "react";
import { useState } from "react";
import {
  ArrowLeftIcon,
  CheckCircleIcon,
  CubeIcon,
  ImagesIcon,
  PackageIcon,
  PencilSimpleIcon,
  PlusIcon,
  StackIcon,
  TagIcon,
  TrashIcon,
} from "@phosphor-icons/react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link, useNavigate } from "@tanstack/react-router";
import { toast } from "sonner";

import { categoriesQueryKey, deleteCategory, getCategoryDetail } from "#/api/categories.ts";
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

function OverviewField({
  icon,
  label,
  children,
  className,
}: {
  icon: React.ReactNode;
  label: string;
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <div className={cn("min-w-0", className)}>
      <dt className="flex items-center gap-2 text-xs font-medium text-muted-foreground">
        {icon}
        {label}
      </dt>
      <dd className="mt-1.5 break-words text-sm font-medium">{children}</dd>
    </div>
  );
}

function Metric({ icon, label, value }: { icon: React.ReactNode; label: string; value: string }) {
  return (
    <div className="flex min-h-24 items-center gap-3 rounded-lg border bg-card p-4">
      <span className="flex size-10 shrink-0 items-center justify-center bg-primary/10 text-primary">
        {icon}
      </span>
      <div className="min-w-0">
        <p className="text-xs font-medium text-muted-foreground">{label}</p>
        <p className="mt-1 truncate text-2xl font-semibold tracking-tight">{value}</p>
      </div>
    </div>
  );
}

function SectionHeader({
  icon,
  title,
  action,
}: {
  icon: React.ReactNode;
  title: string;
  action?: React.ReactNode;
}) {
  return (
    <div className="flex min-h-14 items-center justify-between gap-4 border-b px-4 py-3">
      <div className="flex min-w-0 items-center gap-2.5">
        <span className="text-primary">{icon}</span>
        <h2 className="truncate text-sm font-semibold">{title}</h2>
      </div>
      {action}
    </div>
  );
}

function CategoryDetailSkeleton() {
  return (
    <div className="grid gap-5 pb-10">
      <div className="flex items-center justify-between gap-4">
        <Skeleton className="h-9 w-56 rounded-lg" />
        <Skeleton className="h-9 w-56 rounded-lg" />
      </div>
      <div className="grid rounded-lg border lg:grid-cols-[minmax(18rem,.72fr)_minmax(0,1.28fr)]">
        <Skeleton className="h-80 w-full rounded-lg lg:h-96" />
        <div className="grid gap-5 p-5 sm:grid-cols-2">
          {Array.from({ length: 6 }).map((_, index) => (
            <Skeleton key={index} className="h-12 rounded-lg" />
          ))}
        </div>
      </div>
      <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        {Array.from({ length: 4 }).map((_, index) => (
          <Skeleton key={index} className="h-24 rounded-lg" />
        ))}
      </div>
      <div className="grid gap-5 xl:grid-cols-[minmax(0,1.35fr)_minmax(20rem,.65fr)]">
        <Skeleton className="h-80 rounded-lg" />
        <Skeleton className="h-80 rounded-lg" />
      </div>
    </div>
  );
}

export default function CategoryDetailPage({ pid }: { pid: string }) {
  const [failedImage, setFailedImage] = useState<string>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const categoryQuery = useQuery({
    queryKey: ["category-detail", pid],
    queryFn: () => getCategoryDetail(pid),
  });
  const deleteMutation = useMutation({
    mutationFn: () => deleteCategory(pid),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: categoriesQueryKey });
      toast.success("Category deleted", { id: "delete-category-success" });
      await navigate({ to: "/categories" });
    },
    onError: (error) => {
      toast.error(error.message, { id: "delete-category-error" });
    },
  });

  if (categoryQuery.isLoading) return <CategoryDetailSkeleton />;

  if (categoryQuery.isError) {
    return (
      <ErrorState
        title="Category could not be loaded"
        description={categoryQuery.error.message}
        onRetry={() => categoryQuery.refetch()}
      />
    );
  }

  const detail = categoryQuery.data;
  if (!detail) {
    return (
      <EmptyState
        title="Category not found"
        description="The requested category no longer exists or is unavailable."
        action={
          <Button className="rounded-lg" render={<Link to="/categories" />}>
            Back to Categories
          </Button>
        }
      />
    );
  }

  const category = detail.category;
  const hasImage = Boolean(category.imageLink && failedImage !== category.imageLink);
  const hasProducts = (category.productCount ?? 0) > 0;

  return (
    <div className="w-full pb-10">
      <PageHeader
        title={
          <>
            <span>{titleCase(category.name)}</span>
            <Badge
              variant="outline"
              className="h-6 border-primary/25 bg-primary/10 px-2 text-xs text-primary"
            >
              <CheckCircleIcon weight="fill" /> Active
            </Badge>
          </>
        }
        actions={
          <>
            <Button className="rounded-lg" variant="outline" render={<Link to="/categories" />}>
              <ArrowLeftIcon className="size-4" />
              Back
            </Button>
            <AlertDialog>
              <AlertDialogTrigger
                render={
                  <Button
                    className="rounded-lg"
                    variant="outline"
                    disabled={deleteMutation.isPending || hasProducts}
                    title={hasProducts ? "Categories with products cannot be deleted." : undefined}
                  />
                }
              >
                <TrashIcon className="size-4" />
                Delete
              </AlertDialogTrigger>
              <AlertDialogContent className="rounded-lg">
                <AlertDialogHeader>
                  <AlertDialogTitle>Delete category?</AlertDialogTitle>
                  <AlertDialogDescription>
                    {titleCase(category.name)} will be removed from the active catalogue.
                  </AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                  <AlertDialogCancel className="rounded-lg" disabled={deleteMutation.isPending}>
                    Cancel
                  </AlertDialogCancel>
                  <AlertDialogAction
                    variant="destructive"
                    className="rounded-lg"
                    disabled={deleteMutation.isPending}
                    onClick={() => deleteMutation.mutate()}
                  >
                    Delete category
                  </AlertDialogAction>
                </AlertDialogFooter>
              </AlertDialogContent>
            </AlertDialog>
            <Button
              className="rounded-lg"
              render={<Link to="/categories/$pid/edit" params={{ pid }} />}
            >
              <PencilSimpleIcon className="size-4" />
              Edit category
            </Button>
          </>
        }
      />

      <section className="rounded-lg border bg-card">
        <div
          className={cn(
            "grid",
            hasImage
              ? "lg:grid-cols-[minmax(18rem,.72fr)_minmax(0,1.28fr)]"
              : "lg:grid-cols-[9rem_minmax(0,1fr)]",
          )}
        >
          <div
            className={cn(
              "flex items-center justify-center overflow-hidden rounded-t-lg border-b bg-muted/40 lg:rounded-tl-lg lg:rounded-tr-none lg:rounded-bl-lg lg:border-r lg:border-b-0",
              hasImage ? "h-80 lg:h-96" : "min-h-28 lg:min-h-full",
            )}
          >
            {hasImage ? (
              <img
                src={category.imageLink}
                alt={category.name}
                className="size-full object-cover"
                onError={() => setFailedImage(category.imageLink)}
              />
            ) : (
              <div className="flex flex-col items-center gap-2 text-muted-foreground">
                <PackageIcon className="size-7" />
                <span className="text-xs">No image</span>
              </div>
            )}
          </div>

          <div className="flex min-w-0 flex-col p-5 sm:p-6">
            <dl className="grid gap-x-10 gap-y-6 sm:grid-cols-2 xl:grid-cols-3">
              <OverviewField icon={<TagIcon className="size-4" />} label="Name">
                {titleCase(category.name)}
              </OverviewField>
              <OverviewField icon={<CheckCircleIcon className="size-4" />} label="Status">
                <Badge
                  variant="outline"
                  className="h-5 border-primary/25 bg-primary/10 px-1.5 text-[11px] text-primary"
                >
                  Active
                </Badge>
              </OverviewField>
              <OverviewField icon={<TagIcon className="size-4" />} label="Slug">
                <span className="font-mono text-xs">{category.slug}</span>
              </OverviewField>
              <OverviewField icon={<StackIcon className="size-4" />} label="Child categories">
                {number(detail.children.length)}
              </OverviewField>
              <OverviewField icon={<CubeIcon className="size-4" />} label="Parent category">
                {category.parentName ? titleCase(category.parentName) : "Top level"}
              </OverviewField>
              <OverviewField
                icon={<ImagesIcon className="size-4" />}
                label="Description"
                className="sm:col-span-2 xl:col-span-1"
              >
                <span className="font-normal leading-6 text-muted-foreground">
                  {category.description || "No description"}
                </span>
              </OverviewField>
            </dl>

            <div className="mt-6 flex flex-wrap gap-x-6 gap-y-2 border-t pt-4 text-xs text-muted-foreground">
              <span>Created {dateTime(category.createdAt)}</span>
              <span>Updated {dateTime(category.updatedAt)}</span>
            </div>
          </div>
        </div>
      </section>

      <div className="mt-3 grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        <Metric
          icon={<PackageIcon className="size-5" />}
          label="Total products"
          value={number(category.productCount ?? 0)}
        />
        <Metric
          icon={<CheckCircleIcon className="size-5" />}
          label="Active products"
          value={number(category.productCount ?? 0)}
        />
        <Metric
          icon={<StackIcon className="size-5" />}
          label="Total variants"
          value={number(detail.totalVariants)}
        />
        <Metric
          icon={<CubeIcon className="size-5" />}
          label="Total stock"
          value={number(detail.totalStock)}
        />
      </div>

      <div className="mt-5 grid items-stretch gap-5 xl:grid-cols-[minmax(0,1.35fr)_minmax(20rem,.65fr)]">
        <section className="rounded-lg border bg-card">
          <SectionHeader
            title="Top products in this category"
            icon={<PackageIcon className="size-4" />}
          />
          {detail.topProducts.length ? (
            <div className="divide-y px-4">
              {detail.topProducts.map((product) => (
                <Link
                  key={product.pid}
                  to="/products/$pid"
                  params={{ pid: product.pid }}
                  className="grid grid-cols-[2.75rem_minmax(0,1fr)_auto] items-center gap-3 py-3.5 transition-colors hover:bg-muted/30"
                >
                  <div className="flex size-11 items-center justify-center overflow-hidden bg-muted">
                    {product.imageLink ? (
                      <img src={product.imageLink} alt="" className="size-full object-cover" />
                    ) : (
                      <PackageIcon className="size-4 text-muted-foreground" />
                    )}
                  </div>
                  <div className="min-w-0">
                    <p className="truncate text-sm font-medium">{product.name}</p>
                    <p className="mt-1 truncate font-mono text-[11px] text-muted-foreground">
                      SKU: {product.sku || "Not set"}
                    </p>
                  </div>
                  <div className="min-w-16 text-right">
                    <p className="text-sm font-semibold">{number(product.stockQuantity)}</p>
                    <p className="text-[11px] text-muted-foreground">units</p>
                  </div>
                </Link>
              ))}
            </div>
          ) : (
            <div className="flex min-h-56 flex-col items-center justify-center px-6 text-center">
              <span className="flex size-10 items-center justify-center bg-primary/10 text-primary">
                <PackageIcon className="size-5" />
              </span>
              <p className="mt-3 text-sm text-muted-foreground">No products in this category.</p>
            </div>
          )}
        </section>

        <section className="rounded-lg border bg-card">
          <SectionHeader
            title={`Child categories (${detail.children.length})`}
            icon={<StackIcon className="size-4" />}
            action={
              <Button className="rounded-lg" size="xs" variant="outline" disabled>
                <PlusIcon /> Add subcategory
              </Button>
            }
          />
          {detail.children.length ? (
            <div className="divide-y px-4">
              {detail.children.map((child) => (
                <Link
                  key={child.pid}
                  to="/categories/$pid"
                  params={{ pid: child.pid }}
                  className="grid grid-cols-[2.5rem_minmax(0,1fr)_auto] items-center gap-3 py-3.5 transition-colors hover:bg-muted/30"
                >
                  <div className="flex size-10 items-center justify-center overflow-hidden bg-muted">
                    {child.imageLink ? (
                      <img src={child.imageLink} alt="" className="size-full object-cover" />
                    ) : (
                      <PackageIcon className="size-4 text-muted-foreground" />
                    )}
                  </div>
                  <div className="min-w-0">
                    <p className="truncate text-sm font-medium">{titleCase(child.name)}</p>
                    <p className="mt-1 truncate font-mono text-[11px] text-muted-foreground">
                      /{child.slug}
                    </p>
                  </div>
                  <div className="text-right">
                    <p className="text-xs text-muted-foreground">
                      {number(child.productCount)} products
                    </p>
                    <Badge
                      variant="outline"
                      className="mt-1 h-5 border-primary/25 bg-primary/10 px-1.5 text-[11px] text-primary"
                    >
                      Active
                    </Badge>
                  </div>
                </Link>
              ))}
            </div>
          ) : (
            <div className="flex min-h-56 flex-col items-center justify-center px-6 text-center">
              <span className="flex size-10 items-center justify-center bg-primary/10 text-primary">
                <StackIcon className="size-5" />
              </span>
              <p className="mt-3 text-sm text-muted-foreground">No child categories.</p>
            </div>
          )}
        </section>
      </div>

      <section className="mt-5 rounded-lg border bg-card">
        <div className="grid lg:grid-cols-[minmax(0,1.35fr)_minmax(17rem,.65fr)]">
          <div>
            <SectionHeader title="Category attributes" icon={<TagIcon className="size-4" />} />
            {detail.attributes.length ? (
              <div className="grid gap-x-10 px-5 py-2 sm:grid-cols-2">
                {detail.attributes.map((attribute) => (
                  <div key={attribute.pid} className="border-b py-4 last:border-b-0">
                    <p className="text-sm font-medium">{titleCase(attribute.name)}</p>
                    <p className="mt-1.5 text-sm leading-5 text-muted-foreground">
                      {attribute.description || "No description"}
                    </p>
                  </div>
                ))}
              </div>
            ) : (
              <div className="flex min-h-40 items-center gap-3 px-5 py-6">
                <span className="flex size-10 shrink-0 items-center justify-center bg-primary/10 text-primary">
                  <TagIcon className="size-5" />
                </span>
                <p className="text-sm text-muted-foreground">
                  No attributes linked to this category.
                </p>
              </div>
            )}
          </div>

          <aside className="border-t bg-muted/20 lg:border-t-0 lg:border-l">
            <SectionHeader title="Catalogue metadata" icon={<ImagesIcon className="size-4" />} />
            <dl className="grid gap-6 p-5">
              <div>
                <dt className="text-xs font-medium text-muted-foreground">Category path</dt>
                <dd className="mt-1.5 text-sm font-medium">
                  {category.parentName ? `${titleCase(category.parentName)} > ` : ""}
                  {titleCase(category.name)}
                </dd>
              </div>
              <div>
                <dt className="text-xs font-medium text-muted-foreground">Media</dt>
                <dd className="mt-1.5 text-sm font-medium">{hasImage ? "1 image" : "No image"}</dd>
              </div>
            </dl>
          </aside>
        </div>
      </section>
    </div>
  );
}
