"use client";

import type { ColumnDef } from "@tanstack/react-table";
import {
  CaretLeftIcon,
  CaretRightIcon,
  MagnifyingGlassIcon,
  PlusIcon,
  XIcon,
} from "@phosphor-icons/react";
import { keepPreviousData, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { useMemo, useState, type FormEvent } from "react";
import { toast } from "sonner";

import {
  categoriesQueryKey,
  deleteCategory,
  listCategories,
  type Category,
} from "#/api/categories.ts";
import { CatalogueRowActions } from "#/components/catalogue/catalogue-row-actions";
import { SummaryGrid } from "#/components/catalogue/summary-grid";
import { DataTable } from "#/components/data-table";
import { formatNumber } from "#/utils/formatters";
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
import { titleCase } from "#/utils/formatters";
import { cn } from "#/lib/utils";

const PAGE_SIZE = 20;
const hierarchyOptions = [
  { label: "All categories", value: "all" },
  { label: "Top level", value: "topLevel" },
  { label: "Subcategories", value: "subcategories" },
] as const;

function productLabel(category: Category) {
  if (category.productCount === undefined) return "N/A";

  return formatNumber(category.productCount);
}

function categoryColumns({
  onDelete,
  isDeleting,
}: {
  onDelete: (category: Category) => void;
  isDeleting: boolean;
}): ColumnDef<Category>[] {
  return [
    {
      id: "image",
      header: "Image",
      accessorKey: "imageLink",
      cell: ({ row }) => (
        <div className="flex size-12 items-center justify-center overflow-hidden rounded-lg bg-muted">
          <img
            src={row.original.imageLink}
            alt=""
            className="h-full w-full object-cover"
            loading="lazy"
          />
        </div>
      ),
      enableSorting: false,
      size: 116,
    },
    {
      id: "name",
      header: "Category Name",
      accessorKey: "name",
      cell: ({ row }) => <span className="font-semibold">{titleCase(row.original.name)}</span>,
      size: 260,
    },
    {
      id: "description",
      header: "Description",
      accessorFn: (category) => category.description ?? "",
      cell: ({ row }) => (
        <span className="line-clamp-2 text-muted-foreground">
          {row.original.description || "No description"}
        </span>
      ),
      size: 340,
    },
    {
      id: "slug",
      header: "Slug",
      accessorKey: "slug",
      cell: ({ row }) => (
        <span className="font-mono text-xs text-muted-foreground">{row.original.slug}</span>
      ),
      size: 200,
    },
    {
      id: "products",
      header: "Products",
      accessorFn: (category) => category.productCount ?? -1,
      cell: ({ row }) => (
        <span className={cn(row.original.productCount === undefined && "text-muted-foreground")}>
          {productLabel(row.original)}
        </span>
      ),
      size: 140,
    },
    {
      id: "actions",
      header: "",
      cell: ({ row }) => {
        const category = row.original;

        return (
          <div className="flex justify-end">
            <CatalogueRowActions
              itemName={titleCase(category.name)}
              itemType="Category"
              view={<Link to="/categories/$pid" params={{ pid: category.pid }} />}
              edit={<Link to="/categories/$pid/edit" params={{ pid: category.pid }} />}
              isDeleting={isDeleting}
              deleteDisabled={(category.productCount ?? 0) > 0}
              deleteDisabledReason="Categories with products cannot be deleted."
              onDelete={() => onDelete(category)}
            />
          </div>
        );
      },
      enableSorting: false,
      size: 72,
    },
  ];
}

export default function CategoryCatalogue() {
  const queryClient = useQueryClient();
  const [searchInput, setSearchInput] = useState("");
  const [search, setSearch] = useState("");
  const [page, setPage] = useState(1);
  const [hierarchy, setHierarchy] = useState<(typeof hierarchyOptions)[number]["value"]>("all");

  const categoriesQuery = useQuery({
    queryKey: [...categoriesQueryKey, { page, search, hierarchy }],
    queryFn: () =>
      listCategories({
        page,
        limit: PAGE_SIZE,
        ...(search ? { search } : {}),
        ...(hierarchy === "topLevel" ? { hasParent: false } : {}),
        ...(hierarchy === "subcategories" ? { hasParent: true } : {}),
      }),
    placeholderData: keepPreviousData,
  });

  const deleteMutation = useMutation({
    mutationFn: deleteCategory,
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: categoriesQueryKey });
      toast.success("Category deleted", {
        id: "delete-category-success",
      });
    },
    onError: (error) => {
      toast.error(error.message, {
        id: "delete-category-error",
      });
    },
  });

  const categories = categoriesQuery.data?.data ?? [];
  const pagination = categoriesQuery.data?.pagination;
  const activeFilters = Boolean(search || hierarchy !== "all");

  function applySearch(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSearch(searchInput.trim());
    setPage(1);
  }

  function onHierarchyChange(value: string | null) {
    if (!value) return;

    setHierarchy(value as (typeof hierarchyOptions)[number]["value"]);
    setPage(1);
  }

  function resetFilters() {
    setSearchInput("");
    setSearch("");
    setHierarchy("all");
    setPage(1);
  }

  const columns = useMemo(
    () =>
      categoryColumns({
        onDelete: (category) => deleteMutation.mutate(category.pid),
        isDeleting: deleteMutation.isPending,
      }),
    [deleteMutation],
  );

  const canGoPrevious = Boolean(pagination?.hasPrev);
  const canGoNext = Boolean(pagination?.hasNext);

  return (
    <div className="w-full pb-10">
      <PageHeader
        title="Categories"
        subtitle="Manage storefront category navigation and product grouping."
        actions={
          <Button className="rounded-lg" render={<Link to="/categories/create" />}>
            <PlusIcon className="size-4" />
            Add Category
          </Button>
        }
      />

      <SummaryGrid
        ariaLabel="Category summary"
        items={[
          {
            label: activeFilters ? "Matching categories" : "Total categories",
            value: pagination?.totalItems ?? 0,
          },
          { label: "On this page", value: categories.length },
          {
            label: "Top level",
            value: categories.filter((category) => category.parentId === null).length,
          },
          {
            label: "With products",
            value: categories.filter((category) => (category.productCount ?? 0) > 0).length,
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
                placeholder="Search by category name or slug..."
                className="rounded-lg pl-9"
              />
            </div>
          </form>
          <div className="flex flex-wrap items-center gap-3">
            <Select value={hierarchy} onValueChange={onHierarchyChange}>
              <SelectTrigger className="w-full rounded-lg bg-background sm:w-44">
                <SelectValue />
              </SelectTrigger>
              <SelectContent className="rounded-lg">
                {hierarchyOptions.map((option) => (
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
          data={categories}
          getRowId={(category) => category.pid}
          isLoading={categoriesQuery.isLoading}
          isError={categoriesQuery.isError}
          errorTitle="Categories could not be loaded"
          onRetry={() => categoriesQuery.refetch()}
          emptyTitle={activeFilters ? "No matching categories" : "No categories found"}
          emptyDescription={
            activeFilters
              ? "Adjust or clear the current filters."
              : "Create a category to start organizing the catalogue."
          }
        />

        {!categoriesQuery.isError && !categoriesQuery.isLoading && pagination && (
          <div className="flex flex-col gap-3 border-t p-3 text-sm text-muted-foreground sm:flex-row sm:items-center sm:justify-between">
            <p>
              Page {pagination.page} of {Math.max(pagination.totalPages, 1)} ·{" "}
              {pagination.totalItems}
              {" categories"}
            </p>
            <div className="flex items-center gap-2">
              <Button
                type="button"
                variant="outline"
                size="sm"
                className="rounded-lg"
                disabled={!canGoPrevious || categoriesQuery.isFetching}
                onClick={() => setPage((current) => current - 1)}
              >
                <CaretLeftIcon /> Previous
              </Button>
              <Button
                type="button"
                variant="outline"
                size="sm"
                className="rounded-lg"
                disabled={!canGoNext || categoriesQuery.isFetching}
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
