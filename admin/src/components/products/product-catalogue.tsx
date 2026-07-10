"use client";

import type { ColumnDef } from "@tanstack/react-table";
import {
  CaretDoubleLeftIcon,
  CaretDoubleRightIcon,
  CaretLeftIcon,
  CaretRightIcon,
  EyeIcon,
  GridFourIcon,
  MagnifyingGlassIcon,
  PackageIcon,
  PlusIcon,
  TrashIcon,
} from "@phosphor-icons/react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { useMemo, useState } from "react";
import { toast } from "sonner";

import {
  deleteProduct,
  listProducts,
  productsQueryKey,
  type Pagination,
  type ProductListItem,
  type ProductListParams,
} from "#/api/products.ts";
import { titleCase } from "#/components/catalogue/string-utils";
import { DataTable } from "#/components/data-table";
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
import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "#/components/ui/select";
import { cn } from "#/lib/utils";

const rowOptions = [10, 20, 40] as const;
const stockOptions = [
  { label: "All stock", value: "all" },
  { label: "In stock", value: "inStock" },
  { label: "Out of stock", value: "outOfStock" },
] as const;

function money(value?: string) {
  if (!value) return "N/A";

  return Intl.NumberFormat(undefined, {
    style: "currency",
    currency: "USD",
  }).format(Number(value));
}

function rangeLabel(pagination?: Pagination) {
  if (!pagination || pagination.totalItems === 0) return "No products";

  const start = (pagination.page - 1) * pagination.limit + 1;
  const end = Math.min(pagination.page * pagination.limit, pagination.totalItems);

  return `${start}-${end} of ${pagination.totalItems} products`;
}

function productColumns({
  onDelete,
  isDeleting,
}: {
  onDelete: (product: ProductListItem) => void;
  isDeleting: boolean;
}): ColumnDef<ProductListItem>[] {
  return [
    {
      id: "image",
      header: "Image",
      accessorKey: "primaryImage",
      cell: ({ row }) => (
        <div className="flex size-12 items-center justify-center overflow-hidden rounded-lg bg-muted">
          {row.original.primaryImage ? (
            <img
              src={row.original.primaryImage}
              alt=""
              className="h-full w-full object-cover"
              loading="lazy"
            />
          ) : (
            <PackageIcon className="size-5 text-muted-foreground" />
          )}
        </div>
      ),
      enableSorting: false,
      size: 96,
    },
    {
      id: "name",
      header: "Product Name",
      accessorKey: "name",
      cell: ({ row }) => (
        <div className="min-w-0">
          <p className="font-semibold">{row.original.name}</p>
          <p className="mt-1 line-clamp-1 max-w-80 text-xs text-muted-foreground">
            {row.original.description || "No description"}
          </p>
        </div>
      ),
      size: 300,
    },
    {
      id: "category",
      header: "Category",
      accessorFn: (product) => product.category.name,
      cell: ({ row }) => (
        <span className="font-medium text-muted-foreground">
          {titleCase(row.original.category.name)}
        </span>
      ),
      size: 180,
    },
    {
      id: "sku",
      header: "Default SKU",
      accessorFn: (product) => product.defaultVariant?.sku ?? "",
      cell: ({ row }) => (
        <span
          className={cn(
            "font-mono text-xs",
            !row.original.defaultVariant && "text-muted-foreground",
          )}
        >
          {row.original.defaultVariant?.sku ?? "N/A"}
        </span>
      ),
      size: 180,
    },
    {
      id: "price",
      header: "Price",
      accessorFn: (product) => Number(product.defaultVariant?.price ?? 0),
      cell: ({ row }) => <span>{money(row.original.defaultVariant?.price)}</span>,
      size: 120,
    },
    {
      id: "stock",
      header: "Stock",
      accessorKey: "totalStock",
      cell: ({ row }) => (
        <span className={cn(row.original.totalStock === 0 && "text-destructive")}>
          {Intl.NumberFormat().format(row.original.totalStock)}
        </span>
      ),
      size: 100,
    },
    {
      id: "variants",
      header: "Variants",
      accessorKey: "variantCount",
      cell: ({ row }) => <span>{Intl.NumberFormat().format(row.original.variantCount)}</span>,
      size: 120,
    },
    {
      id: "actions",
      header: "",
      cell: ({ row }) => {
        const product = row.original;

        return (
          <div className="flex justify-end gap-2">
            <Button
              type="button"
              variant="outline"
              size="icon-sm"
              aria-label={`View ${product.name}`}
              render={<Link to="/products/$pid" params={{ pid: product.pid }} />}
            >
              <EyeIcon className="size-4" />
            </Button>
            <AlertDialog>
              <AlertDialogTrigger
                render={
                  <Button
                    type="button"
                    variant="outline"
                    size="icon-sm"
                    aria-label={`Delete ${product.name}`}
                  />
                }
              >
                <TrashIcon className="size-4" />
              </AlertDialogTrigger>
              <AlertDialogContent>
                <AlertDialogHeader>
                  <AlertDialogTitle>Delete product?</AlertDialogTitle>
                  <AlertDialogDescription>
                    {product.name} will be removed from the active catalogue.
                  </AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                  <AlertDialogCancel disabled={isDeleting}>Cancel</AlertDialogCancel>
                  <AlertDialogAction
                    variant="destructive"
                    disabled={isDeleting}
                    onClick={() => onDelete(product)}
                  >
                    Delete
                  </AlertDialogAction>
                </AlertDialogFooter>
              </AlertDialogContent>
            </AlertDialog>
          </div>
        );
      },
      enableSorting: false,
      size: 112,
    },
  ];
}

export default function ProductCatalogue() {
  const queryClient = useQueryClient();
  const [search, setSearch] = useState("");
  const [stockStatus, setStockStatus] = useState<(typeof stockOptions)[number]["value"]>("all");
  const [page, setPage] = useState(1);
  const [limit, setLimit] = useState<(typeof rowOptions)[number]>(10);

  const queryParams: ProductListParams = {
    page,
    limit,
    ...(search.trim() ? { search: search.trim() } : {}),
    ...(stockStatus !== "all" ? { stockStatus } : {}),
  };

  const productsQuery = useQuery({
    queryKey: [...productsQueryKey, queryParams],
    queryFn: () => listProducts(queryParams),
  });

  const deleteMutation = useMutation({
    mutationFn: deleteProduct,
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: productsQueryKey });
      toast.success("Product deleted", {
        id: "delete-product-success",
      });
    },
    onError: (error) => {
      toast.error(error.message, {
        id: "delete-product-error",
      });
    },
  });

  const products = productsQuery.data?.data ?? [];
  const pagination = productsQuery.data?.pagination;
  const columns = useMemo(
    () =>
      productColumns({
        onDelete: (product) => deleteMutation.mutate(product.pid),
        isDeleting: deleteMutation.isPending,
      }),
    [deleteMutation],
  );
  const canGoPrevious = Boolean(pagination?.hasPrev);
  const canGoNext = Boolean(pagination?.hasNext);

  function onSearch(value: string) {
    setSearch(value);
    setPage(1);
  }

  function onLimitChange(value: string | null) {
    if (!value) return;

    setLimit(Number(value) as (typeof rowOptions)[number]);
    setPage(1);
  }

  function onStockStatusChange(value: string | null) {
    if (!value) return;

    setStockStatus(value as (typeof stockOptions)[number]["value"]);
    setPage(1);
  }

  return (
    <div className="w-full pb-10">
      <PageHeader
        title="Products"
        subtitle="Manage product catalogue rows, stock, pricing, and variants."
        actions={
          <Button render={<Link to="/products/create" />}>
            <PlusIcon className="size-4" />
            Add Product
          </Button>
        }
      />

      <div className="mb-4 flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
        <div className="relative w-full lg:max-w-96">
          <MagnifyingGlassIcon
            className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground"
            aria-hidden
          />
          <Input
            value={search}
            onChange={(event) => onSearch(event.target.value)}
            placeholder="Search products, categories, or SKU..."
            className="rounded-lg pl-9"
          />
        </div>

        <div className="flex items-center gap-3">
          <Select value={stockStatus} onValueChange={onStockStatusChange}>
            <SelectTrigger size="sm" className="w-36 rounded-lg">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {stockOptions.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          <Button type="button" variant="outline" size="icon" aria-label="Grid view">
            <GridFourIcon className="size-5" />
          </Button>
        </div>
      </div>

      <DataTable
        columns={columns}
        data={products}
        getRowId={(product) => product.pid}
        isLoading={productsQuery.isLoading}
        isError={productsQuery.isError}
        errorTitle="Products could not be loaded"
        onRetry={() => productsQuery.refetch()}
        emptyTitle={search ? "No matching products" : "No products found"}
        emptyDescription={
          search
            ? "Try a different product name, category, or SKU."
            : "Create a product to start building the catalogue."
        }
      />

      <div className="mt-4 flex flex-col gap-3 text-sm text-muted-foreground lg:flex-row lg:items-center lg:justify-between">
        <div className="flex items-center gap-3">
          <span>Rows per page</span>
          <Select value={String(limit)} onValueChange={onLimitChange}>
            <SelectTrigger size="sm" className="w-20 rounded-lg">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {rowOptions.map((option) => (
                <SelectItem key={option} value={String(option)}>
                  {option}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <span className="hidden sm:inline">{rangeLabel(pagination)}</span>
        </div>

        <div className="flex items-center gap-3 lg:justify-end">
          <span>
            Page {pagination?.page ?? page} of {pagination?.totalPages || 1}
          </span>
          <div className="flex gap-2">
            <Button
              type="button"
              variant="outline"
              size="icon-sm"
              disabled={!canGoPrevious}
              onClick={() => setPage(1)}
              aria-label="First page"
            >
              <CaretDoubleLeftIcon className="size-4" />
            </Button>
            <Button
              type="button"
              variant="outline"
              size="icon-sm"
              disabled={!canGoPrevious}
              onClick={() => setPage((current) => Math.max(1, current - 1))}
              aria-label="Previous page"
            >
              <CaretLeftIcon className="size-4" />
            </Button>
            <Button
              type="button"
              variant="outline"
              size="icon-sm"
              disabled={!canGoNext}
              onClick={() => setPage((current) => current + 1)}
              aria-label="Next page"
            >
              <CaretRightIcon className="size-4" />
            </Button>
            <Button
              type="button"
              variant="outline"
              size="icon-sm"
              disabled={!canGoNext || !pagination}
              onClick={() => setPage(pagination?.totalPages ?? page)}
              aria-label="Last page"
            >
              <CaretDoubleRightIcon className="size-4" />
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
