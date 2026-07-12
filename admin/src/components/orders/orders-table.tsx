"use client";

import type { ColumnDef } from "@tanstack/react-table";
import {
  ArrowClockwiseIcon,
  ArrowRightIcon,
  CalendarBlankIcon,
  CaretLeftIcon,
  CaretRightIcon,
  MagnifyingGlassIcon,
  XIcon,
} from "@phosphor-icons/react";
import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { format } from "date-fns";
import { useMemo, useState, type FormEvent } from "react";
import type { DateRange } from "react-day-picker";

import { listOrders, ordersQueryKey, type Order } from "#/api/orders.ts";
import { DataTable } from "#/components/data-table";
import { OrderStatusBadge } from "#/components/orders/order-status";
import { PageHeader } from "#/components/page-header";
import { Button } from "#/components/ui/button";
import { Calendar } from "#/components/ui/calendar";
import { Input } from "#/components/ui/input";
import { Popover, PopoverContent, PopoverTrigger } from "#/components/ui/popover";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "#/components/ui/select";
import { cn } from "#/lib/utils";

const ALL_STATUSES = "all";
const PAGE_SIZE = 20;

const dateFormatter = new Intl.DateTimeFormat(undefined, {
  day: "2-digit",
  month: "short",
  year: "numeric",
});

function formatMoney(value: string, currency: string) {
  return new Intl.NumberFormat(undefined, {
    style: "currency",
    currency,
  }).format(Number(value));
}

function orderColumns(): ColumnDef<Order>[] {
  return [
    {
      id: "order",
      header: "Order",
      accessorFn: (order) => order.orderNumber,
      cell: ({ row }) => (
        <div>
          <p className="font-mono text-xs font-semibold">#{row.original.orderNumber}</p>
          <p className="mt-1 text-xs text-muted-foreground">
            {dateFormatter.format(new Date(row.original.placedAt ?? row.original.createdAt))}
          </p>
        </div>
      ),
      size: 150,
    },
    {
      id: "customer",
      header: "Customer",
      accessorFn: (order) => `${order.customerName} ${order.customerEmail}`,
      cell: ({ row }) => (
        <div className="min-w-0">
          <p className="truncate font-medium">{row.original.customerName}</p>
          <p className="truncate text-xs text-muted-foreground">{row.original.customerEmail}</p>
        </div>
      ),
      size: 260,
    },
    {
      id: "total",
      header: "Total",
      accessorFn: (order) => Number(order.grandTotal),
      cell: ({ row }) => (
        <span className="font-mono text-xs font-semibold">
          {formatMoney(row.original.grandTotal, row.original.currency)}
        </span>
      ),
      size: 140,
    },
    {
      id: "payment",
      header: "Payment",
      accessorFn: (order) => order.paymentStatus,
      cell: ({ row }) => <OrderStatusBadge value={row.original.paymentStatus} />,
      size: 140,
    },
    {
      id: "fulfillment",
      header: "Fulfillment",
      accessorFn: (order) => order.fulfillmentStatus,
      cell: ({ row }) => <OrderStatusBadge value={row.original.fulfillmentStatus} />,
      size: 150,
    },
    {
      id: "status",
      header: "Order status",
      accessorFn: (order) => order.status,
      cell: ({ row }) => <OrderStatusBadge value={row.original.status} />,
      size: 150,
    },
    {
      id: "actions",
      header: "",
      enableSorting: false,
      cell: ({ row }) => (
        <Button
          variant="outline"
          size="sm"
          className="rounded-lg"
          render={<Link to="/orders/$pid" params={{ pid: row.original.pid }} />}
        >
          View <ArrowRightIcon />
        </Button>
      ),
      size: 112,
    },
  ];
}

function DateRangePicker({
  value,
  onChange,
}: {
  value?: DateRange;
  onChange: (range?: DateRange) => void;
}) {
  return (
    <Popover>
      <PopoverTrigger
        render={
          <Button
            variant="outline"
            className={cn(
              "w-full justify-start rounded-lg text-left font-normal sm:w-64",
              !value?.from && "text-muted-foreground",
            )}
          />
        }
      >
        <CalendarBlankIcon />
        {value?.from ? (
          value.to ? (
            <>
              {format(value.from, "dd MMM yyyy")} to {format(value.to, "dd MMM yyyy")}
            </>
          ) : (
            format(value.from, "dd MMM yyyy")
          )
        ) : (
          <span>Order date</span>
        )}
      </PopoverTrigger>
      <PopoverContent className="w-auto rounded-lg p-0" align="end">
        <Calendar
          mode="range"
          defaultMonth={value?.from}
          selected={value}
          onSelect={onChange}
          numberOfMonths={2}
        />
      </PopoverContent>
    </Popover>
  );
}

export function OrdersTable() {
  const [page, setPage] = useState(1);
  const [searchInput, setSearchInput] = useState("");
  const [search, setSearch] = useState("");
  const [status, setStatus] = useState(ALL_STATUSES);
  const [dateRange, setDateRange] = useState<DateRange>();

  const params = {
    page,
    limit: PAGE_SIZE,
    search: search || undefined,
    status: status === ALL_STATUSES ? undefined : status,
    createdFrom: dateRange?.from ? format(dateRange.from, "yyyy-MM-dd") : undefined,
    createdTo: dateRange?.to ? format(dateRange.to, "yyyy-MM-dd") : undefined,
  };
  const ordersQuery = useQuery({
    queryKey: [...ordersQueryKey, params],
    queryFn: () => listOrders(params),
    placeholderData: keepPreviousData,
  });
  const columns = useMemo(() => orderColumns(), []);
  const orders = ordersQuery.data?.data ?? [];
  const pagination = ordersQuery.data?.pagination;
  const activeFilters = Boolean(search || status !== ALL_STATUSES || dateRange?.from);

  function applySearch(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setPage(1);
    setSearch(searchInput.trim());
  }

  function resetFilters() {
    setSearchInput("");
    setSearch("");
    setStatus(ALL_STATUSES);
    setDateRange(undefined);
    setPage(1);
  }

  return (
    <div className="w-full pb-10">
      <PageHeader
        title="Orders"
        subtitle="Track customer orders, payments, and fulfillment from one operational view."
        actions={
          <Button
            type="button"
            variant="outline"
            className="rounded-lg"
            onClick={() => ordersQuery.refetch()}
            disabled={ordersQuery.isFetching}
          >
            <ArrowClockwiseIcon className={cn(ordersQuery.isFetching && "animate-spin")} />
            Refresh
          </Button>
        }
      />

      <section
        className="mb-5 grid rounded-lg border sm:grid-cols-2 xl:grid-cols-4"
        aria-label="Order summary"
      >
        <div className="border-b p-4 sm:border-r xl:border-b-0">
          <p className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
            {activeFilters ? "Matching orders" : "Total orders"}
          </p>
          <p className="mt-2 text-2xl font-semibold tabular-nums">{pagination?.totalItems ?? 0}</p>
        </div>
        <div className="border-b p-4 xl:border-r xl:border-b-0">
          <p className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
            On this page
          </p>
          <p className="mt-2 text-2xl font-semibold tabular-nums">{orders.length}</p>
        </div>
        <div className="border-b p-4 sm:border-r sm:border-b-0 xl:border-r">
          <p className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
            Awaiting payment
          </p>
          <p className="mt-2 text-2xl font-semibold tabular-nums">
            {orders.filter((order) => order.paymentStatus === "pending").length}
          </p>
        </div>
        <div className="p-4">
          <p className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
            To fulfill
          </p>
          <p className="mt-2 text-2xl font-semibold tabular-nums">
            {orders.filter((order) => order.fulfillmentStatus === "unfulfilled").length}
          </p>
        </div>
      </section>

      <div className="rounded-lg border bg-card">
        <div className="grid gap-3 border-b p-3 xl:grid-cols-[minmax(18rem,1fr)_auto_auto_auto]">
          <form onSubmit={applySearch} className="flex min-w-0">
            <div className="relative min-w-0 flex-1">
              <MagnifyingGlassIcon className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                value={searchInput}
                onChange={(event) => setSearchInput(event.target.value)}
                placeholder="Search order number or customer"
                className="rounded-lg pr-11 pl-9"
              />
              <Button
                type="submit"
                variant="ghost"
                size="icon-sm"
                className="absolute right-1 top-1/2 -translate-y-1/2 rounded-md"
                aria-label="Search orders"
              >
                <MagnifyingGlassIcon />
              </Button>
            </div>
          </form>

          <Select
            value={status}
            onValueChange={(value) => {
              setStatus(value ?? ALL_STATUSES);
              setPage(1);
            }}
          >
            <SelectTrigger className="w-full rounded-lg bg-background xl:w-44">
              <SelectValue />
            </SelectTrigger>
            <SelectContent className="rounded-lg">
              <SelectItem value={ALL_STATUSES}>All statuses</SelectItem>
              <SelectItem value="pending">Pending</SelectItem>
              <SelectItem value="completed">Completed</SelectItem>
              <SelectItem value="cancelled">Cancelled</SelectItem>
            </SelectContent>
          </Select>

          <DateRangePicker
            value={dateRange}
            onChange={(range) => {
              setDateRange(range);
              setPage(1);
            }}
          />

          {activeFilters && (
            <Button type="button" variant="ghost" className="rounded-lg" onClick={resetFilters}>
              <XIcon /> Clear
            </Button>
          )}
        </div>

        <DataTable
          columns={columns}
          data={orders}
          getRowId={(order) => order.pid}
          isLoading={ordersQuery.isLoading}
          isError={ordersQuery.isError}
          onRetry={() => ordersQuery.refetch()}
          errorTitle="Orders could not be loaded"
          errorDescription="Check the order service and retry the request."
          emptyTitle={activeFilters ? "No matching orders" : "No orders yet"}
          emptyDescription={
            activeFilters
              ? "Adjust or clear the current filters."
              : "Customer orders will appear here after checkout."
          }
        />

        {!ordersQuery.isError && !ordersQuery.isLoading && pagination && (
          <div className="flex flex-col gap-3 border-t p-3 text-sm text-muted-foreground sm:flex-row sm:items-center sm:justify-between">
            <p>
              Page {pagination.page} of {Math.max(pagination.totalPages, 1)} ·{" "}
              {pagination.totalItems} orders
            </p>
            <div className="flex items-center gap-2">
              <Button
                type="button"
                variant="outline"
                size="sm"
                className="rounded-lg"
                disabled={!pagination.hasPrev || ordersQuery.isFetching}
                onClick={() => setPage((value) => value - 1)}
              >
                <CaretLeftIcon /> Previous
              </Button>
              <Button
                type="button"
                variant="outline"
                size="sm"
                className="rounded-lg"
                disabled={!pagination.hasNext || ordersQuery.isFetching}
                onClick={() => setPage((value) => value + 1)}
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
