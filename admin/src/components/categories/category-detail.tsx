"use client";

import type React from "react";
import {
  ArrowLeftIcon,
  CheckCircleIcon,
  CubeIcon,
  ImagesIcon,
  PackageIcon,
  PlusIcon,
  StackIcon,
  TagIcon,
} from "@phosphor-icons/react";
import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";

import { getCategoryDetail } from "#/api/categories.ts";
import { titleCase } from "#/components/catalogue/string-utils";
import { EmptyState } from "#/components/empty-state";
import { ErrorState } from "#/components/error-state";
import { PageHeader } from "#/components/page-header";
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

function Panel({
  title,
  icon,
  action,
  children,
  className,
}: {
  title: string;
  icon: React.ReactNode;
  action?: React.ReactNode;
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <section className={cn("border bg-card shadow-sm", className)}>
      <div className="flex items-center justify-between gap-3 border-b px-4 py-3.5">
        <div className="flex items-center gap-2">
          <span className="text-muted-foreground">{icon}</span>
          <h2 className="text-sm font-semibold">{title}</h2>
        </div>
        {action}
      </div>
      <div className="p-4">{children}</div>
    </section>
  );
}

function Metric({
  icon,
  label,
  value,
  tone = "blue",
}: {
  icon: React.ReactNode;
  label: string;
  value: string;
  tone?: "blue" | "green" | "amber";
}) {
  return (
    <div className="flex items-center gap-3 border bg-card p-4 shadow-sm">
      <div
        className={cn(
          "flex size-10 items-center justify-center",
          tone === "green" && "bg-emerald-50 text-emerald-600 dark:bg-emerald-950/40",
          tone === "amber" && "bg-amber-50 text-amber-600 dark:bg-amber-950/40",
          tone === "blue" && "bg-blue-50 text-blue-600 dark:bg-blue-950/40",
        )}
      >
        {icon}
      </div>
      <div>
        <p className="text-xs text-muted-foreground">{label}</p>
        <p className="mt-0.5 text-xl font-bold tracking-tight">{value}</p>
      </div>
    </div>
  );
}

function CategoryDetailSkeleton() {
  return (
    <div className="grid gap-6 pb-10">
      <div className="flex items-center justify-between">
        <Skeleton className="h-9 w-56" />
        <Skeleton className="h-9 w-36" />
      </div>
      <Skeleton className="h-64" />
      <div className="grid gap-4 lg:grid-cols-4">
        <Skeleton className="h-24" />
        <Skeleton className="h-24" />
        <Skeleton className="h-24" />
        <Skeleton className="h-24" />
      </div>
      <div className="grid gap-5 lg:grid-cols-[1.25fr,.75fr]">
        <Skeleton className="h-80" />
        <Skeleton className="h-80" />
      </div>
    </div>
  );
}

export default function CategoryDetailPage({ pid }: { pid: string }) {
  const categoryQuery = useQuery({
    queryKey: ["category-detail", pid],
    queryFn: () => getCategoryDetail(pid),
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
        action={<Button render={<Link to="/categories" />}>Back to Categories</Button>}
      />
    );
  }

  const category = detail.category;

  return (
    <div className="w-full pb-10">
      <PageHeader
        title={
          <>
            <span>{titleCase(category.name)}</span>
            <Badge className="h-6 bg-emerald-50 px-2 text-xs text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-300">
              <CheckCircleIcon weight="fill" /> Active
            </Badge>
          </>
        }
        actions={
          <>
            <Button variant="outline" render={<Link to="/categories" />}>
              <ArrowLeftIcon className="size-4" />
              Back
            </Button>
            <Button render={<Link to="/categories/create" />}>
              <PlusIcon className="size-4" />
              Add category
            </Button>
          </>
        }
      />

      <section className="border bg-card p-4 shadow-sm sm:p-5">
        <div className="grid gap-6 lg:grid-cols-[minmax(14rem,.62fr)_minmax(0,1.38fr)]">
          <div className="flex aspect-[4/3] items-center justify-center overflow-hidden bg-muted/50">
            {category.imageLink ? (
              <img
                src={category.imageLink}
                alt={category.name}
                className="size-full object-cover"
              />
            ) : (
              <PackageIcon className="size-12 text-muted-foreground" />
            )}
          </div>
          <dl className="grid content-center gap-x-10 gap-y-5 sm:grid-cols-2">
            <div>
              <dt className="flex items-center gap-2 text-xs text-muted-foreground">
                <TagIcon className="size-4" /> Name
              </dt>
              <dd className="mt-1 text-sm font-medium">{titleCase(category.name)}</dd>
            </div>
            <div>
              <dt className="flex items-center gap-2 text-xs text-muted-foreground">
                <CheckCircleIcon className="size-4" /> Status
              </dt>
              <dd className="mt-1">
                <Badge className="h-5 bg-emerald-50 px-1.5 text-[11px] text-emerald-700">
                  Active
                </Badge>
              </dd>
            </div>
            <div>
              <dt className="flex items-center gap-2 text-xs text-muted-foreground">
                <TagIcon className="size-4" /> Slug
              </dt>
              <dd className="mt-1 font-mono text-xs">{category.slug}</dd>
            </div>
            <div>
              <dt className="flex items-center gap-2 text-xs text-muted-foreground">
                <StackIcon className="size-4" /> Child categories
              </dt>
              <dd className="mt-1 text-sm font-medium">{number(detail.children.length)}</dd>
            </div>
            <div>
              <dt className="flex items-center gap-2 text-xs text-muted-foreground">
                <CubeIcon className="size-4" /> Parent category
              </dt>
              <dd className="mt-1 text-sm font-medium">
                {category.parentName ? titleCase(category.parentName) : "Top level"}
              </dd>
            </div>
            <div>
              <dt className="flex items-center gap-2 text-xs text-muted-foreground">
                <ImagesIcon className="size-4" /> Description
              </dt>
              <dd className="mt-1 text-xs leading-5 text-muted-foreground">
                {category.description || "No description"}
              </dd>
            </div>
            <div className="text-xs text-muted-foreground">
              Created <span className="ml-1 text-foreground">{dateTime(category.createdAt)}</span>
            </div>
            <div className="text-xs text-muted-foreground">
              Updated <span className="ml-1 text-foreground">{dateTime(category.updatedAt)}</span>
            </div>
          </dl>
        </div>
      </section>

      <div className="mt-5 grid gap-4 lg:grid-cols-4">
        <Metric
          icon={<PackageIcon className="size-5" />}
          label="Total products"
          value={number(category.productCount ?? 0)}
        />
        <Metric
          icon={<CheckCircleIcon className="size-5" />}
          label="Active products"
          value={number(category.productCount ?? 0)}
          tone="green"
        />
        <Metric
          icon={<StackIcon className="size-5" />}
          label="Total variants"
          value={number(detail.totalVariants)}
          tone="amber"
        />
        <Metric
          icon={<CubeIcon className="size-5" />}
          label="Total stock"
          value={number(detail.totalStock)}
        />
      </div>

      <div className="mt-5 grid gap-5 lg:grid-cols-[1.25fr,.75fr]">
        <Panel
          title={`Child categories (${detail.children.length})`}
          icon={<StackIcon className="size-4" />}
          action={
            <Button size="xs" variant="outline">
              <PlusIcon /> Add subcategory
            </Button>
          }
        >
          {detail.children.length ? (
            <div className="divide-y">
              {detail.children.map((child) => (
                <Link
                  key={child.pid}
                  to="/categories/$pid"
                  params={{ pid: child.pid }}
                  className="flex items-center gap-3 py-3 first:pt-0 last:pb-0 hover:bg-muted/20"
                >
                  <div className="size-10 shrink-0 overflow-hidden bg-muted">
                    {child.imageLink ? (
                      <img src={child.imageLink} alt="" className="size-full object-cover" />
                    ) : (
                      <PackageIcon className="m-2.5 size-5 text-muted-foreground" />
                    )}
                  </div>
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-sm font-medium">{titleCase(child.name)}</p>
                    <p className="truncate font-mono text-[11px] text-muted-foreground">
                      /{child.slug}
                    </p>
                  </div>
                  <span className="text-xs text-muted-foreground">
                    {number(child.productCount)} products
                  </span>
                  <Badge className="h-5 bg-emerald-50 px-1.5 text-[11px] text-emerald-700">
                    Active
                  </Badge>
                </Link>
              ))}
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">No child categories.</p>
          )}
        </Panel>

        <Panel title="Top products in this category" icon={<PackageIcon className="size-4" />}>
          {detail.topProducts.length ? (
            <div className="divide-y">
              {detail.topProducts.map((product) => (
                <Link
                  key={product.pid}
                  to="/products/$pid"
                  params={{ pid: product.pid }}
                  className="flex items-center gap-3 py-3 first:pt-0 last:pb-0"
                >
                  <div className="size-10 shrink-0 overflow-hidden bg-muted">
                    {product.imageLink ? (
                      <img src={product.imageLink} alt="" className="size-full object-cover" />
                    ) : (
                      <PackageIcon className="m-2.5 size-5 text-muted-foreground" />
                    )}
                  </div>
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-sm font-medium">{product.name}</p>
                    <p className="font-mono text-[11px] text-muted-foreground">
                      SKU: {product.sku || "Not set"}
                    </p>
                  </div>
                  <span className="text-right text-xs text-muted-foreground">
                    {number(product.stockQuantity)}
                    <br />
                    units
                  </span>
                </Link>
              ))}
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">No products in this category.</p>
          )}
        </Panel>
      </div>

      <div className="mt-5 grid gap-5 lg:grid-cols-2">
        <Panel title="Category attributes" icon={<TagIcon className="size-4" />}>
          {detail.attributes.length ? (
            <div className="divide-y">
              {detail.attributes.map((attribute) => (
                <div
                  key={attribute.pid}
                  className="grid grid-cols-[minmax(0,.8fr)_minmax(0,1.2fr)] gap-4 py-3 first:pt-0 last:pb-0"
                >
                  <span className="text-sm font-medium">{titleCase(attribute.name)}</span>
                  <span className="text-sm text-muted-foreground">
                    {attribute.description || "No description"}
                  </span>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">No attributes linked to this category.</p>
          )}
        </Panel>
        <Panel title="Catalogue metadata" icon={<ImagesIcon className="size-4" />}>
          <dl className="grid gap-4 sm:grid-cols-2">
            <div>
              <dt className="text-xs text-muted-foreground">Category path</dt>
              <dd className="mt-1 text-sm font-medium">
                {category.parentName ? `${titleCase(category.parentName)} > ` : ""}
                {titleCase(category.name)}
              </dd>
            </div>
            <div>
              <dt className="text-xs text-muted-foreground">Media</dt>
              <dd className="mt-1 text-sm font-medium">
                {category.imageLink ? "1 image" : "No image"}
              </dd>
            </div>
          </dl>
        </Panel>
      </div>
    </div>
  );
}
