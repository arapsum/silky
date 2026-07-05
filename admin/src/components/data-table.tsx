"use client";

import {
  flexRender,
  getCoreRowModel,
  getSortedRowModel,
  useReactTable,
  type ColumnDef,
  type SortingState,
} from "@tanstack/react-table";
import { CaretDownIcon, CaretUpDownIcon, CaretUpIcon } from "@phosphor-icons/react";
import { useState } from "react";

import { EmptyState } from "#/components/empty-state";
import { ErrorState } from "#/components/error-state";
import { Button } from "#/components/ui/button";
import { Skeleton } from "#/components/ui/skeleton";
import { cn } from "#/lib/utils";

type DataTableProps<TData, TValue> = {
  columns: ColumnDef<TData, TValue>[];
  data: TData[];
  emptyTitle: string;
  emptyDescription: string;
  errorTitle?: string;
  errorDescription?: string;
  isLoading?: boolean;
  isError?: boolean;
  onRetry?: () => void;
  getRowId?: (row: TData, index: number) => string;
};

export function DataTable<TData, TValue>({
  columns,
  data,
  emptyTitle,
  emptyDescription,
  errorTitle = "Data could not be loaded",
  errorDescription = "Check your session and retry the request.",
  isLoading = false,
  isError = false,
  onRetry,
  getRowId,
}: DataTableProps<TData, TValue>) {
  const [sorting, setSorting] = useState<SortingState>([]);
  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getRowId,
    onSortingChange: setSorting,
    state: {
      sorting,
    },
  });

  if (isLoading) {
    return (
      <div className="rounded-lg border bg-background">
        <div className="grid gap-0">
          {Array.from({ length: 6 }).map((_, index) => (
            <div
              key={index}
              className="grid grid-cols-[minmax(14rem,1.3fr)_minmax(10rem,1fr)_8rem_10rem] gap-4 border-b p-4 last:border-b-0"
            >
              <Skeleton className="h-10 w-full" />
              <Skeleton className="h-10 w-full" />
              <Skeleton className="h-10 w-full" />
              <Skeleton className="h-10 w-full" />
            </div>
          ))}
        </div>
      </div>
    );
  }

  if (isError) {
    return <ErrorState title={errorTitle} description={errorDescription} onRetry={onRetry} />;
  }

  if (!data.length) {
    return <EmptyState title={emptyTitle} description={emptyDescription} />;
  }

  return (
    <div className="overflow-hidden rounded-lg border bg-background">
      <div className="w-full overflow-x-auto">
        <table className="w-full min-w-240 border-collapse text-sm">
          <thead className="bg-muted/50 text-left text-xs font-medium uppercase text-muted-foreground">
            {table.getHeaderGroups().map((headerGroup) => (
              <tr key={headerGroup.id}>
                {headerGroup.headers.map((header) => {
                  const sorted = header.column.getIsSorted();

                  return (
                    <th
                      key={header.id}
                      className="border-b px-4 py-3 align-middle"
                      style={{ width: header.getSize() }}
                    >
                      {header.isPlaceholder ? null : (
                        <Button
                          type="button"
                          variant="ghost"
                          size="sm"
                          className={cn(
                            "-ml-2 h-8 px-2 text-xs font-semibold uppercase text-muted-foreground",
                            !header.column.getCanSort() && "pointer-events-none",
                          )}
                          onClick={header.column.getToggleSortingHandler()}
                        >
                          {flexRender(header.column.columnDef.header, header.getContext())}
                          {header.column.getCanSort() &&
                            (sorted === "asc" ? (
                              <CaretUpIcon className="size-3.5" />
                            ) : sorted === "desc" ? (
                              <CaretDownIcon className="size-3.5" />
                            ) : (
                              <CaretUpDownIcon className="size-3.5" />
                            ))}
                        </Button>
                      )}
                    </th>
                  );
                })}
              </tr>
            ))}
          </thead>
          <tbody>
            {table.getRowModel().rows.map((row) => (
              <tr key={row.id} className="border-b last:border-b-0 hover:bg-muted/35">
                {row.getVisibleCells().map((cell) => (
                  <td key={cell.id} className="px-4 py-3 align-middle">
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
