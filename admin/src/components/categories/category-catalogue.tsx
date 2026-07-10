"use client";

import type { ColumnDef } from "@tanstack/react-table";
import {
  CaretDoubleLeftIcon,
  CaretDoubleRightIcon,
  CaretLeftIcon,
  CaretRightIcon,
  GridFourIcon,
  MagnifyingGlassIcon,
  PencilSimpleIcon,
  PlusIcon,
  TrashIcon,
} from "@phosphor-icons/react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { useMemo, useState } from "react";
import { toast } from "sonner";

import {
  categoriesQueryKey,
  deleteCategory,
  listCategories,
  type Category,
  type Pagination,
} from "#/api/categories.ts";
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
import { Checkbox } from "#/components/ui/checkbox";
import { Input } from "#/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "#/components/ui/select";
import { titleCase } from "#/components/catalogue/string-utils";
import { cn } from "#/lib/utils";

const rowOptions = [10, 20, 40] as const;

function productLabel(category: Category) {
  if (category.productCount === undefined) return "N/A";

  return Intl.NumberFormat().format(category.productCount);
}

function rangeLabel(pagination?: Pagination) {
  if (!pagination || pagination.totalItems === 0) return "No categories";

  const start = (pagination.page - 1) * pagination.limit + 1;
  const end = Math.min(pagination.page * pagination.limit, pagination.totalItems);

  return `${start}-${end} of ${pagination.totalItems} categories`;
}

function categoryColumns({
  selectedIds,
  onToggleRow,
  onToggleAll,
  onDelete,
  isDeleting,
}: {
  selectedIds: Set<string>;
  onToggleRow: (pid: string, checked: boolean) => void;
  onToggleAll: (checked: boolean) => void;
  onDelete: (category: Category) => void;
  isDeleting: boolean;
}): ColumnDef<Category>[] {
  return [
    {
      id: "select",
      header: ({ table }) => {
        const rows = table.getRowModel().rows;
        const checked = rows.length > 0 && rows.every((row) => selectedIds.has(row.original.pid));

        return (
          <Checkbox
            checked={checked}
            onCheckedChange={(value) => onToggleAll(value === true)}
            aria-label="Select all categories"
          />
        );
      },
      cell: ({ row }) => (
        <Checkbox
          checked={selectedIds.has(row.original.pid)}
          onCheckedChange={(value) => onToggleRow(row.original.pid, value === true)}
          aria-label={`Select ${row.original.name}`}
        />
      ),
      enableSorting: false,
      size: 52,
    },
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
          <div className="flex justify-end gap-2">
            <Button
              type="button"
              variant="outline"
              size="icon-sm"
              aria-label={`Edit ${category.name}`}
            >
              <PencilSimpleIcon className="size-4" />
            </Button>
            <AlertDialog>
              <AlertDialogTrigger
                render={
                  <Button
                    type="button"
                    variant="outline"
                    size="icon-sm"
                    aria-label={`Delete ${category.name}`}
                  />
                }
              >
                <TrashIcon className="size-4" />
              </AlertDialogTrigger>
              <AlertDialogContent>
                <AlertDialogHeader>
                  <AlertDialogTitle>Delete category?</AlertDialogTitle>
                  <AlertDialogDescription>
                    {titleCase(category.name)} will be removed from the active catalogue.
                  </AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                  <AlertDialogCancel disabled={isDeleting}>Cancel</AlertDialogCancel>
                  <AlertDialogAction
                    variant="destructive"
                    disabled={isDeleting}
                    onClick={() => onDelete(category)}
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

export default function CategoryCatalogue() {
  const queryClient = useQueryClient();
  const [search, setSearch] = useState("");
  const [page, setPage] = useState(1);
  const [limit, setLimit] = useState<(typeof rowOptions)[number]>(10);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(() => new Set());

  const categoriesQuery = useQuery({
    queryKey: [...categoriesQueryKey, { page, limit, search }],
    queryFn: () =>
      listCategories({
        page,
        limit,
        ...(search.trim() ? { search: search.trim() } : {}),
      }),
  });

  const deleteMutation = useMutation({
    mutationFn: deleteCategory,
    onSuccess: async () => {
      setSelectedIds(new Set());
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

  function onSearch(value: string) {
    setSearch(value);
    setPage(1);
    setSelectedIds(new Set());
  }

  function onLimitChange(value: string | null) {
    if (!value) return;

    setLimit(Number(value) as (typeof rowOptions)[number]);
    setPage(1);
    setSelectedIds(new Set());
  }

  function onToggleRow(pid: string, checked: boolean) {
    setSelectedIds((current) => {
      const next = new Set(current);

      if (checked) {
        next.add(pid);
      } else {
        next.delete(pid);
      }

      return next;
    });
  }

  function onToggleAll(checked: boolean) {
    if (!checked) {
      setSelectedIds(new Set());
      return;
    }

    setSelectedIds(new Set(categories.map((category) => category.pid)));
  }

  const columns = useMemo(
    () =>
      categoryColumns({
        selectedIds,
        onToggleRow,
        onToggleAll,
        onDelete: (category) => deleteMutation.mutate(category.pid),
        isDeleting: deleteMutation.isPending,
      }),
    [categories, deleteMutation, selectedIds],
  );

  const canGoPrevious = Boolean(pagination?.hasPrev);
  const canGoNext = Boolean(pagination?.hasNext);

  return (
    <div className="w-full pb-10">
      <PageHeader
        title="Categories"
        subtitle="Manage storefront category navigation and product grouping."
        actions={
          <Button
            className="bg-blue-600 hover:bg-blue-700"
            render={<Link to="/categories/create" />}
          >
            <PlusIcon className="size-4" />
            Add Category
          </Button>
        }
      />

      <div className="mb-4 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div className="relative w-full sm:max-w-80">
          <MagnifyingGlassIcon
            className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground"
            aria-hidden
          />
          <Input
            value={search}
            onChange={(event) => onSearch(event.target.value)}
            placeholder="Search by category name..."
            className="rounded-lg pl-9"
          />
        </div>

        <Button type="button" variant="outline" size="icon" aria-label="Grid view">
          <GridFourIcon className="size-5" />
        </Button>
      </div>

      <DataTable
        columns={columns}
        data={categories}
        getRowId={(category) => category.pid}
        isLoading={categoriesQuery.isLoading}
        isError={categoriesQuery.isError}
        errorTitle="Categories could not be loaded"
        onRetry={() => categoriesQuery.refetch()}
        emptyTitle={search ? "No matching categories" : "No categories found"}
        emptyDescription={
          search
            ? "Try a different category name or slug."
            : "Create a category to start organizing the catalogue."
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
