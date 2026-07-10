import type React from "react";

import { PageHeader } from "#/components/page-header";
import { Button } from "#/components/ui/button";

type CatalogueFormHeaderProps = {
  title: string;
  description: string;
  onCancel?: () => void;
  cancelLabel?: string;
};

type CatalogueFormSectionProps = {
  title: string;
  description: string;
  children: React.ReactNode;
  className?: string;
  contentClassName?: string;
  paddedTop?: boolean;
};

type CatalogueFormActionsProps = {
  draftLabel?: string;
  submitLabel: string;
  pendingSubmitLabel: string;
  isPending?: boolean;
  onDraft: () => void;
};

export function CatalogueFormHeader({
  title,
  description,
  onCancel = () => window.history.back(),
  cancelLabel = "Cancel",
}: CatalogueFormHeaderProps) {
  return (
    <PageHeader
      title={title}
      subtitle={description}
      className="mb-8"
      actions={
        <Button type="button" variant="outline" onClick={onCancel}>
          {cancelLabel}
        </Button>
      }
    />
  );
}

export function CatalogueFormSection({
  title,
  description,
  children,
  className,
  contentClassName,
  paddedTop = true,
}: CatalogueFormSectionProps) {
  return (
    <div
      className={[
        "grid gap-8 border-b lg:grid-cols-[minmax(14rem,0.45fr)_minmax(0,1fr)]",
        paddedTop ? "py-10" : "pb-10",
        className,
      ]
        .filter(Boolean)
        .join(" ")}
    >
      <div className="lg:col-span-1">
        <h2 className="text-base font-semibold">{title}</h2>
        <p className="mt-1 max-w-xs text-sm text-muted-foreground">{description}</p>
      </div>

      <div className={contentClassName}>{children}</div>
    </div>
  );
}

export function CatalogueFormActions({
  draftLabel = "Save as Draft",
  submitLabel,
  pendingSubmitLabel,
  isPending,
  onDraft,
}: CatalogueFormActionsProps) {
  return (
    <div className="mt-auto flex flex-col-reverse gap-3 pt-8 sm:flex-row sm:justify-end">
      <Button type="button" variant="outline" onClick={onDraft}>
        {draftLabel}
      </Button>
      <Button type="submit" disabled={isPending}>
        {isPending ? pendingSubmitLabel : submitLabel}
      </Button>
    </div>
  );
}
