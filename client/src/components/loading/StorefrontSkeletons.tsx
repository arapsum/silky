import type { ComponentPropsWithoutRef, ReactNode } from "react";

interface SkeletonProps extends ComponentPropsWithoutRef<"span"> {
  shape?: "line" | "block" | "circle";
}

export function Skeleton({ className = "", shape = "line", ...props }: SkeletonProps) {
  return (
    <span
      {...props}
      aria-hidden="true"
      className={`skeleton skeleton--${shape} ${className}`.trim()}
    />
  );
}

interface LoadingRegionProps {
  children: ReactNode;
  className?: string;
  label: string;
}

function LoadingRegion({ children, className = "", label }: LoadingRegionProps) {
  return (
    <div
      aria-busy="true"
      aria-label={label}
      className={`skeleton-region ${className}`.trim()}
      role="status"
    >
      {children}
    </div>
  );
}

function PageHeadingSkeleton() {
  return (
    <header className="account-header skeleton-page-heading">
      <Skeleton className="skeleton--eyebrow" />
      <Skeleton className="skeleton--display" />
      <Skeleton className="skeleton--copy" />
    </header>
  );
}

function PanelHeadingSkeleton() {
  return (
    <div className="account-panel__heading skeleton-panel-heading">
      <div>
        <Skeleton className="skeleton--eyebrow" />
        <Skeleton className="skeleton--subheading" />
      </div>
      <Skeleton className="skeleton--badge" />
    </div>
  );
}

function FormSkeleton() {
  return (
    <div className="profile-form skeleton-form">
      <Skeleton className="skeleton--field" shape="block" />
      <Skeleton className="skeleton--field" shape="block" />
      <Skeleton className="skeleton--field skeleton--field-wide" shape="block" />
      <Skeleton className="skeleton--button" />
    </div>
  );
}

function OrderRows({ compact = false, count = 3 }: { compact?: boolean; count?: number }) {
  return (
    <div className={`order-list skeleton-order-list ${compact ? "compact" : ""}`.trim()}>
      {Array.from({ length: count }, (_, index) => (
        <div className="skeleton-order-row" key={index}>
          <span>
            <Skeleton className="skeleton--order-number" />
            <Skeleton className="skeleton--meta" />
          </span>
          {!compact && <Skeleton className="skeleton--badge" />}
          {!compact && <Skeleton className="skeleton--badge" />}
          <Skeleton className="skeleton--amount" />
        </div>
      ))}
    </div>
  );
}

function SummarySkeleton() {
  return (
    <aside className="order-summary skeleton-summary">
      <Skeleton className="skeleton--eyebrow" />
      <Skeleton className="skeleton--summary-heading" />
      <div className="skeleton-summary__totals">
        {Array.from({ length: 4 }, (_, index) => (
          <div key={index}>
            <Skeleton className="skeleton--label" />
            <Skeleton className="skeleton--amount" />
          </div>
        ))}
      </div>
      <Skeleton className="skeleton--summary-button" />
      <Skeleton className="skeleton--summary-note" />
    </aside>
  );
}

export function AccountOverviewSkeleton() {
  return (
    <LoadingRegion className="account-overview" label="Loading your account">
      <PageHeadingSkeleton />
      <section className="account-panel">
        <PanelHeadingSkeleton />
        <FormSkeleton />
      </section>
      <section className="account-panel">
        <PanelHeadingSkeleton />
        <OrderRows compact />
      </section>
    </LoadingRegion>
  );
}

export function AddressBookSkeleton() {
  return (
    <LoadingRegion className="address-grid skeleton-address-grid" label="Loading saved addresses">
      {Array.from({ length: 2 }, (_, index) => (
        <article key={index}>
          <Skeleton className="skeleton--badge" />
          <Skeleton className="skeleton--subheading" />
          <Skeleton className="skeleton--copy" />
          <Skeleton className="skeleton--copy-short" />
          <div className="skeleton-address-actions">
            <Skeleton className="skeleton--action" />
            <Skeleton className="skeleton--action" />
          </div>
        </article>
      ))}
    </LoadingRegion>
  );
}

export function OrderHistorySkeleton() {
  return (
    <LoadingRegion label="Loading order history">
      <OrderRows count={4} />
    </LoadingRegion>
  );
}

export function OrderDetailSkeleton() {
  return (
    <LoadingRegion label="Loading order details">
      <Skeleton className="skeleton--back" />
      <PageHeadingSkeleton />
      <section className="account-panel order-items">
        <PanelHeadingSkeleton />
        {Array.from({ length: 2 }, (_, index) => (
          <article className="skeleton-order-item" key={index}>
            <Skeleton className="skeleton--product-image" shape="block" />
            <div>
              <Skeleton className="skeleton--order-number" />
              <Skeleton className="skeleton--copy" />
              <Skeleton className="skeleton--meta" />
            </div>
            <Skeleton className="skeleton--amount" />
          </article>
        ))}
      </section>
      <div className="order-detail-grid">
        {Array.from({ length: 2 }, (_, index) => (
          <section className="account-panel" key={index}>
            <PanelHeadingSkeleton />
            <Skeleton className="skeleton--copy" />
            <Skeleton className="skeleton--copy-short" />
          </section>
        ))}
      </div>
    </LoadingRegion>
  );
}

export function CartSkeleton() {
  return (
    <LoadingRegion className="cart-layout" label="Loading your bag">
      <section>
        <div className="cart-heading skeleton-cart-heading">
          <div>
            <Skeleton className="skeleton--eyebrow" />
            <Skeleton className="skeleton--display" />
          </div>
          <Skeleton className="skeleton--meta" />
        </div>
        <div className="cart-lines">
          <div className="cart-line skeleton-cart-line">
            <Skeleton className="skeleton--cart-image" shape="block" />
            <div className="cart-line__body">
              <Skeleton className="skeleton--order-number" />
              <Skeleton className="skeleton--copy" />
              <Skeleton className="skeleton--amount" />
              <Skeleton className="skeleton--quantity" />
            </div>
          </div>
        </div>
      </section>
      <SummarySkeleton />
    </LoadingRegion>
  );
}

export function CheckoutSkeleton() {
  return (
    <LoadingRegion className="checkout-layout" label="Preparing secure checkout">
      <section className="checkout-main">
        <Skeleton className="skeleton--back" />
        <Skeleton className="skeleton--eyebrow" />
        <Skeleton className="skeleton--display" />
        <Skeleton className="skeleton--copy" />
        <div className="skeleton-address-options">
          {Array.from({ length: 2 }, (_, index) => (
            <Skeleton className="skeleton--address-option" key={index} shape="block" />
          ))}
        </div>
        <Skeleton className="skeleton--action" />
      </section>
      <SummarySkeleton />
    </LoadingRegion>
  );
}

export function InlineAmountSkeleton() {
  return <Skeleton className="skeleton--amount skeleton--amount-inline" />;
}
