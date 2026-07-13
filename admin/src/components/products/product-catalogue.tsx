"use client";

import type { ColumnDef } from "@tanstack/react-table";
import {
  CaretLeftIcon,
  CaretRightIcon,
  MagnifyingGlassIcon,
  PackageIcon,
  PlusIcon,
  XIcon,
} from "@phosphor-icons/react";
import { keepPreviousData, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { useMemo, useState, type FormEvent } from "react";
import { toast } from "sonner";

import {
  deleteProduct,
  listProducts,
  productsQueryKey,
  type ProductListItem,
  type ProductListParams,
} from "#/api/products.ts";
import { SummaryGrid } from "#/components/catalogue/summary-grid";
import { titleCase } from "#/components/catalogue/string-utils";
import { CatalogueRowActions } from "#/components/catalogue/catalogue-row-actions";
import { DataTable } from "#/components/data-table";
import { formatCurrency, formatNumber } from "#/utils/formatters";
import { PageHeader } from "#/components/page-header";
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

const PAGE_SIZE = 20;
const stockOptions = [
  { label: "All stock", value: "all" },
  { label: "In stock", value: "inStock" },
  { label: "Out of stock", value: "outOfStock" },
] as const;

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
      cell: ({ row }) => (
        <span>
          {row.original.defaultVariant?.price
            ? formatCurrency(row.original.defaultVariant.price)
            : "N/A"}
        </span>
      ),
      size: 120,
    },
    {
      id: "stock",
      header: "Stock",
      accessorKey: "totalStock",
      cell: ({ row }) => (
        <span className={cn(row.original.totalStock === 0 && "text-destructive")}>
          {formatNumber(row.original.totalStock)}
        </span>
      ),
      size: 100,
    },
    {
      id: "variants",
      header: "Variants",
      accessorKey: "variantCount",
      cell: ({ row }) => <span>{formatNumber(row.original.variantCount)}</span>,
      size: 120,
    },
    {
      id: "actions",
      header: "",
      cell: ({ row }) => {
        const product = row.original;

        return (
          <div className="flex justify-end">
            <CatalogueRowActions
              itemName={product.name}
              itemType="Product"
              view={<Link to="/products/$pid" params={{ pid: product.pid }} />}
              edit={<Link to="/products/$pid/edit" params={{ pid: product.pid }} />}
              isDeleting={isDeleting}
              onDelete={() => onDelete(product)}
            />
          </div>
        );
      },
      enableSorting: false,
      size: 72,
    },
  ];
}

export default function ProductCatalogue() {
  const queryClient = useQueryClient();
  const [searchInput, setSearchInput] = useState("");
  const [search, setSearch] = useState("");
  const [stockStatus, setStockStatus] = useState<(typeof stockOptions)[number]["value"]>("all");
  const [page, setPage] = useState(1);

  const queryParams: ProductListParams = {
    page,
    limit: PAGE_SIZE,
    ...(search ? { search } : {}),
    ...(stockStatus !== "all" ? { stockStatus } : {}),
  };

  const productsQuery = useQuery({
    queryKey: [...productsQueryKey, queryParams],
    queryFn: () => listProducts(queryParams),
    placeholderData: keepPreviousData,
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
  const activeFilters = Boolean(search || stockStatus !== "all");

  function applySearch(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSearch(searchInput.trim());
    setPage(1);
  }

  function onStockStatusChange(value: string | null) {
    if (!value) return;

    setStockStatus(value as (typeof stockOptions)[number]["value"]);
    setPage(1);
  }

  function resetFilters() {
    setSearchInput("");
    setSearch("");
    setStockStatus("all");
    setPage(1);
  }

  return (
    <div className="w-full pb-10">
      <PageHeader
        title="Products"
        subtitle="Manage product catalogue rows, stock, pricing, and variants."
        actions={
          <Button className="rounded-lg" render={<Link to="/products/create" />}>
            <PlusIcon className="size-4" />
            Add Product
          </Button>
        }
      />

      <SummaryGrid
        ariaLabel="Product summary"
        items={[
          {
            label: activeFilters ? "Matching products" : "Total products",
            value: pagination?.totalItems ?? 0,
          },
          { label: "On this page", value: products.length },
          {
            label: "In stock",
            value: products.filter((product) => product.totalStock > 0).length,
          },
          {
            label: "Out of stock",
            value: products.filter((product) => product.totalStock === 0).length,
          },
        ]}
      />

      <div className="rounded-lg border bg-card">
        <div className="flex flex-col gap-3 border-b p-3 sm:flex-row sm:items-center sm:justify-between">
          <form onSubmit={applySearch} className="w-full min-w-0 sm:w-96">
            <div className="relative min-w-0">
              <MagnifyingGlassIcon
                className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground"
                aria-hidden
              />
              <Input
                value={searchInput}
                onChange={(event) => setSearchInput(event.target.value)}
                placeholder="Search products, categories, or SKU..."
                className="rounded-lg pl-9"
              />
            </div>
          </form>
          <div className="flex flex-wrap items-center gap-3">
            <Select value={stockStatus} onValueChange={onStockStatusChange}>
              <SelectTrigger className="w-full rounded-lg bg-background sm:w-40">
                <SelectValue />
              </SelectTrigger>
              <SelectContent className="rounded-lg">
                {stockOptions.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            {activeFilters && (
              <Button type="button" variant="ghost" className="rounded-lg" onClick={resetFilters}>
                <XIcon /> Clear
              </Button>
            )}
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
          emptyTitle={activeFilters ? "No matching products" : "No products found"}
          emptyDescription={
            activeFilters
              ? "Adjust or clear the current filters."
              : "Create a product to start building the catalogue."
          }
        />

        {!productsQuery.isError && !productsQuery.isLoading && pagination && (
          <div className="flex flex-col gap-3 border-t p-3 text-sm text-muted-foreground sm:flex-row sm:items-center sm:justify-between">
            <p>
              Page {pagination.page} of {Math.max(pagination.totalPages, 1)} ·{" "}
              {pagination.totalItems}
              {" products"}
            </p>
            <div className="flex items-center gap-2">
              <Button
                type="button"
                variant="outline"
                size="sm"
                className="rounded-lg"
                disabled={!canGoPrevious || productsQuery.isFetching}
                onClick={() => setPage((current) => current - 1)}
              >
                <CaretLeftIcon /> Previous
              </Button>
              <Button
                type="button"
                variant="outline"
                size="sm"
                className="rounded-lg"
                disabled={!canGoNext || productsQuery.isFetching}
                onClick={() => setPage((current) => current + 1)}
              >
                Next <CaretRightIcon />
              </Button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
