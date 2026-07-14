"use client";

import type React from "react";
import { useState } from "react";
import {
  ArrowLeftIcon,
  CalendarBlankIcon,
  ChatTextIcon,
  CreditCardIcon,
  MapPinIcon,
  PackageIcon,
  PencilSimpleIcon,
  ReceiptIcon,
  UserIcon,
} from "@phosphor-icons/react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { toast } from "sonner";

import {
  getOrder,
  ordersQueryKey,
  updateOrder,
  type OrderAddressSnapshot,
  type OrderDetail,
  type OrderItem,
  type UpdateOrderInput,
} from "#/api/orders.ts";
import { formatCurrency, formatDateTime, initials, titleCase } from "#/utils/formatters";
import {
  FULFILLMENT_STATUSES,
  ORDER_STATUSES,
  OrderStatusBadge,
  PAYMENT_STATUSES,
} from "#/components/orders/order-status";
import { EmptyState } from "#/components/empty-state";
import { ErrorState } from "#/components/error-state";
import { PageHeader } from "#/components/page-header";
import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
import { Button } from "#/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "#/components/ui/dialog";
import { Label } from "#/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "#/components/ui/select";
import { Skeleton } from "#/components/ui/skeleton";
import { Textarea } from "#/components/ui/textarea";
import { cn } from "#/lib/utils";
import { useAccess } from "#/hooks/use-access";
import { PERMISSIONS } from "#/lib/access";

function optionSummary(options: Record<string, unknown>) {
  const values = Object.entries(options).filter(([, value]) => value !== null && value !== "");

  if (!values.length) return "No options recorded";

  return values.map(([name, value]) => `${titleCase(name)}: ${String(value)}`).join(" · ");
}

function addressLines(address: OrderAddressSnapshot) {
  const locality = [address.city, address.region, address.postalCode].filter(Boolean).join(", ");

  return [
    address.recipientName,
    address.company ?? undefined,
    address.lineOne,
    address.lineTwo ?? undefined,
    locality || undefined,
    address.countryCode,
  ].filter((line): line is string => Boolean(line));
}

function OrderFact({
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
    <div className={cn("flex min-w-0 gap-3 p-4 sm:p-5", className)}>
      <span className="mt-0.5 text-primary">{icon}</span>
      <div className="min-w-0">
        <p className="text-xs font-medium text-muted-foreground">{label}</p>
        <div className="mt-1 truncate text-sm font-semibold">{children}</div>
      </div>
    </div>
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
    <section className={cn("flex h-full flex-col rounded-lg border bg-card", className)}>
      <div className="flex items-center gap-2 border-b px-4 py-3.5">
        <span className="text-primary">{icon}</span>
        <h2 className="text-sm font-semibold">{title}</h2>
      </div>
      <div className="flex-1">{children}</div>
    </section>
  );
}

function AddressBlock({ title, address }: { title: string; address: OrderAddressSnapshot }) {
  const lines = addressLines(address);

  return (
    <div className="border-b px-4 py-4 last:border-b-0">
      <div className="flex items-center gap-2 text-sm font-medium">
        <MapPinIcon className="size-4 text-muted-foreground" />
        {title}
      </div>
      {lines.length ? (
        <address className="mt-2 pl-6 text-sm leading-6 not-italic text-muted-foreground">
          {lines.map((line) => (
            <div key={line}>{line}</div>
          ))}
        </address>
      ) : (
        <p className="mt-2 pl-6 text-sm text-muted-foreground">No address recorded.</p>
      )}
    </div>
  );
}

function LineItems({ items, currency }: { items: OrderItem[]; currency: string }) {
  return (
    <Panel title={`Order items (${items.length})`} icon={<PackageIcon className="size-4" />}>
      {items.length ? (
        <div className="overflow-x-auto">
          <table className="w-full min-w-[42rem] text-left text-sm">
            <thead className="border-b bg-muted/35 text-xs text-muted-foreground">
              <tr>
                <th className="px-4 py-3 font-medium">SKU</th>
                <th className="px-4 py-3 font-medium">Product</th>
                <th className="px-4 py-3 font-medium">Options</th>
                <th className="px-4 py-3 text-right font-medium">Qty</th>
                <th className="px-4 py-3 text-right font-medium">Line total</th>
              </tr>
            </thead>
            <tbody>
              {items.map((item) => (
                <tr key={item.pid} className="border-b last:border-b-0 hover:bg-muted/20">
                  <td className="px-4 py-3.5 font-mono text-xs font-medium">{item.sku}</td>
                  <td className="px-4 py-3.5 font-medium">{item.productName}</td>
                  <td className="max-w-64 px-4 py-3.5 text-xs leading-5 text-muted-foreground">
                    {optionSummary(item.selectedOptions)}
                  </td>
                  <td className="px-4 py-3.5 text-right tabular-nums">{item.quantity}</td>
                  <td className="px-4 py-3.5 text-right font-medium tabular-nums">
                    {formatCurrency(item.lineTotal, currency)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <div className="flex min-h-48 flex-col items-center justify-center px-6 text-center">
          <PackageIcon className="size-7 text-muted-foreground" />
          <p className="mt-3 text-sm font-medium">No order items recorded</p>
          <p className="mt-1 text-sm text-muted-foreground">
            This order does not contain a saved line item.
          </p>
        </div>
      )}
    </Panel>
  );
}

function CustomerInformation({ order }: { order: OrderDetail }) {
  return (
    <Panel title="Customer & addresses" icon={<UserIcon className="size-4" />}>
      <div className="flex items-center gap-3 border-b px-4 py-4">
        <Avatar>
          <AvatarImage src={order.customerImage ?? undefined} alt={order.customerName} />
          <AvatarFallback>{initials(order.customerName, "C")}</AvatarFallback>
        </Avatar>
        <div className="min-w-0">
          <p className="truncate text-sm font-semibold">{order.customerName}</p>
          <a
            className="mt-1 block truncate text-sm text-primary hover:underline"
            href={`mailto:${order.customerEmail}`}
          >
            {order.customerEmail}
          </a>
        </div>
      </div>
      <AddressBlock title="Shipping address" address={order.shippingAddressSnapshot} />
      <AddressBlock title="Billing address" address={order.billingAddressSnapshot} />
    </Panel>
  );
}

function PaymentSummary({ order }: { order: OrderDetail }) {
  const rows = [
    ["Subtotal", order.subtotal],
    ["Discount", order.discountTotal],
    ["Shipping", order.shippingTotal],
    ["Tax", order.taxTotal],
  ];

  return (
    <Panel title="Payment summary" icon={<CreditCardIcon className="size-4" />}>
      <dl className="space-y-3 p-4 text-sm">
        {rows.map(([label, amount]) => (
          <div
            key={label}
            className="flex items-center justify-between gap-4 text-muted-foreground"
          >
            <dt>{label}</dt>
            <dd className="font-medium tabular-nums text-foreground">
              {formatCurrency(amount, order.currency)}
            </dd>
          </div>
        ))}
        <div className="flex items-center justify-between gap-4 border-t pt-3 text-base font-semibold">
          <dt>Order total</dt>
          <dd className="tabular-nums">{formatCurrency(order.grandTotal, order.currency)}</dd>
        </div>
      </dl>
    </Panel>
  );
}

function Notes({ order }: { order: OrderDetail }) {
  const notes = [
    ["Customer note", order.customerNote],
    ["Staff note", order.staffNote],
  ];

  return (
    <Panel title="Notes" icon={<ChatTextIcon className="size-4" />}>
      <div className="divide-y">
        {notes.map(([label, note]) => (
          <div key={label} className="px-4 py-4">
            <p className="text-sm font-medium">{label}</p>
            <p className="mt-1.5 text-sm leading-6 text-muted-foreground">
              {note || "No note recorded."}
            </p>
          </div>
        ))}
      </div>
    </Panel>
  );
}

function UpdateOrderDialog({ order, pid }: { order: OrderDetail; pid: string }) {
  const queryClient = useQueryClient();
  const [isOpen, setIsOpen] = useState(false);
  const [status, setStatus] = useState(order.status);
  const [paymentStatus, setPaymentStatus] = useState(order.paymentStatus);
  const [fulfillmentStatus, setFulfillmentStatus] = useState(order.fulfillmentStatus);
  const [staffNote, setStaffNote] = useState(order.staffNote ?? "");
  const mutation = useMutation({
    mutationFn: (input: UpdateOrderInput) => updateOrder(pid, input),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: [...ordersQueryKey, pid] }),
        queryClient.invalidateQueries({ queryKey: ordersQueryKey }),
      ]);
      toast.success("Order updated", { id: "order-update-success" });
      setIsOpen(false);
    },
    onError: (error) => {
      toast.error(error.message, { id: "order-update-error" });
    },
  });

  function resetForm() {
    setStatus(order.status);
    setPaymentStatus(order.paymentStatus);
    setFulfillmentStatus(order.fulfillmentStatus);
    setStaffNote(order.staffNote ?? "");
  }

  function handleOpenChange(open: boolean) {
    setIsOpen(open);
    if (open) resetForm();
  }

  function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    mutation.mutate({
      status,
      paymentStatus,
      fulfillmentStatus,
      staffNote,
    });
  }

  return (
    <>
      <Button type="button" className="rounded-lg" onClick={() => handleOpenChange(true)}>
        <PencilSimpleIcon /> Update order
      </Button>
      <Dialog open={isOpen} onOpenChange={handleOpenChange}>
        <DialogContent className="rounded-lg">
          <DialogHeader>
            <DialogTitle>Update order #{order.orderNumber}</DialogTitle>
            <DialogDescription>
              Update order state and the internal note. Customer notes remain read-only.
            </DialogDescription>
          </DialogHeader>
          <form className="grid gap-4" onSubmit={handleSubmit}>
            <div className="grid gap-4 sm:grid-cols-2">
              <div className="grid gap-2">
                <Label htmlFor="order-status">Order status</Label>
                <Select value={status} onValueChange={(value) => value && setStatus(value)}>
                  <SelectTrigger id="order-status" className="w-full rounded-lg bg-background">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent className="rounded-lg">
                    {ORDER_STATUSES.map((value) => (
                      <SelectItem key={value} value={value}>
                        {titleCase(value)}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="grid gap-2">
                <Label htmlFor="payment-status">Payment status</Label>
                <Select
                  value={paymentStatus}
                  onValueChange={(value) => value && setPaymentStatus(value)}
                >
                  <SelectTrigger id="payment-status" className="w-full rounded-lg bg-background">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent className="rounded-lg">
                    {PAYMENT_STATUSES.map((value) => (
                      <SelectItem key={value} value={value}>
                        {titleCase(value)}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>
            <div className="grid gap-2">
              <Label htmlFor="fulfillment-status">Fulfillment status</Label>
              <Select
                value={fulfillmentStatus}
                onValueChange={(value) => value && setFulfillmentStatus(value)}
              >
                <SelectTrigger id="fulfillment-status" className="w-full rounded-lg bg-background">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent className="rounded-lg">
                  {FULFILLMENT_STATUSES.map((value) => (
                    <SelectItem key={value} value={value}>
                      {titleCase(value)}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="grid gap-2">
              <Label htmlFor="staff-note">Staff note</Label>
              <Textarea
                id="staff-note"
                value={staffNote}
                onChange={(event) => setStaffNote(event.target.value)}
                className="rounded-lg"
                placeholder="Add a note visible to staff only"
              />
            </div>
            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                className="rounded-lg"
                disabled={mutation.isPending}
                onClick={() => handleOpenChange(false)}
              >
                Cancel
              </Button>
              <Button type="submit" className="rounded-lg" disabled={mutation.isPending}>
                {mutation.isPending ? "Saving..." : "Save changes"}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </>
  );
}

function OrderDetailSkeleton() {
  return (
    <div className="grid gap-5 pb-10">
      <div className="flex items-center justify-between">
        <div className="grid gap-2">
          <Skeleton className="h-8 w-52 rounded-lg" />
          <Skeleton className="h-4 w-72 rounded-lg" />
        </div>
        <Skeleton className="h-9 w-32 rounded-lg" />
      </div>
      <Skeleton className="h-28 rounded-lg" />
      <div className="grid gap-5 xl:grid-cols-[minmax(0,1.45fr)_minmax(20rem,.8fr)]">
        <Skeleton className="h-96 rounded-lg" />
        <Skeleton className="h-96 rounded-lg" />
      </div>
      <div className="grid gap-5 lg:grid-cols-2">
        <Skeleton className="h-60 rounded-lg" />
        <Skeleton className="h-60 rounded-lg" />
      </div>
    </div>
  );
}

export default function OrderDetailPage({ pid }: { pid: string }) {
  const { can } = useAccess();
  const canUpdate = can(PERMISSIONS.orders.update);
  const orderQuery = useQuery({
    queryKey: [...ordersQueryKey, pid],
    queryFn: () => getOrder(pid),
  });

  if (orderQuery.isLoading) return <OrderDetailSkeleton />;

  if (orderQuery.isError) {
    return (
      <ErrorState
        title="Order could not be loaded"
        description={orderQuery.error.message}
        onRetry={() => orderQuery.refetch()}
        className="rounded-lg bg-card"
      />
    );
  }

  const detail = orderQuery.data;
  if (!detail) {
    return (
      <EmptyState
        title="Order not found"
        description="The requested order no longer exists or is unavailable."
        className="rounded-lg bg-card"
        action={
          <Button className="rounded-lg" variant="outline" render={<Link to="/orders" />}>
            Back to orders
          </Button>
        }
      />
    );
  }

  const { order, items } = detail;

  return (
    <div className="w-full pb-10">
      <PageHeader
        title={
          <>
            <span>Order #{order.orderNumber}</span>
            <OrderStatusBadge value={order.status} />
          </>
        }
        subtitle={`${order.customerName} · Placed ${formatDateTime(order.placedAt ?? order.createdAt, "Not recorded")}`}
        actions={
          <>
            <Button className="rounded-lg" variant="outline" render={<Link to="/orders" />}>
              <ArrowLeftIcon /> Back to orders
            </Button>
            {canUpdate && <UpdateOrderDialog key={order.updatedAt} order={order} pid={pid} />}
          </>
        }
      />

      <section className="grid overflow-hidden rounded-lg border bg-card sm:grid-cols-2 xl:grid-cols-4">
        <OrderFact
          icon={<ReceiptIcon className="size-5" />}
          label="Order number"
          className="border-b sm:border-r xl:border-b-0"
        >
          #{order.orderNumber}
        </OrderFact>
        <OrderFact
          icon={<CalendarBlankIcon className="size-5" />}
          label="Placed on"
          className="border-b xl:border-r xl:border-b-0"
        >
          {formatDateTime(order.placedAt ?? order.createdAt, "Not recorded")}
        </OrderFact>
        <OrderFact
          icon={<CreditCardIcon className="size-5" />}
          label="Payment"
          className="border-b sm:border-r sm:border-b-0 xl:border-r"
        >
          <OrderStatusBadge value={order.paymentStatus} />
        </OrderFact>
        <OrderFact icon={<PackageIcon className="size-5" />} label="Fulfillment">
          <OrderStatusBadge value={order.fulfillmentStatus} />
        </OrderFact>
      </section>

      <div className="mt-5 grid items-start gap-5 xl:grid-cols-[minmax(0,1.45fr)_minmax(20rem,.8fr)]">
        <LineItems items={items} currency={order.currency} />
        <CustomerInformation order={order} />
      </div>

      <div className="mt-5 grid gap-5 lg:grid-cols-2">
        <PaymentSummary order={order} />
        <Notes order={order} />
      </div>

      <p className="mt-4 text-xs text-muted-foreground">
        Created {formatDateTime(order.createdAt, "Not recorded")}
        <span className="mx-2 text-border">|</span>
        Updated {formatDateTime(order.updatedAt, "Not recorded")}
      </p>
    </div>
  );
}
