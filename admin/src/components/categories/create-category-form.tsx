"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { ImageSquareIcon, UploadSimpleIcon, XIcon } from "@phosphor-icons/react";
import { useMutation } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useEffect, useMemo, useRef, useState, type ChangeEvent, type DragEvent } from "react";
import { useForm } from "react-hook-form";
import { toast } from "sonner";
import { z } from "zod";

import { createCategory } from "#/api/categories.ts";
import { uploadCategoryImage } from "#/api/uploads.ts";
import FormField from "#/components/form-field";
import { Button } from "#/components/ui/button";
import { cn } from "#/lib/utils";

const IMAGE_MAX_BYTES = 1024 * 1024 * 5;
const IMAGE_TYPES = new Set(["image/jpeg", "image/png", "image/webp"]);

const categorySchema = z.object({
  name: z
    .string()
    .trim()
    .min(2, "Category name requires 2 letters")
    .max(80, "Category name must be under 80 characters"),
  slug: z
    .string()
    .trim()
    .min(2, "Slug requires 2 letters")
    .max(96, "Slug must be under 96 characters")
    .regex(/^[a-z0-9]+(?:-[a-z0-9]+)*$/, "Use lowercase words separated by hyphens"),
  parentCategory: z.string().optional(),
  displayType: z.string().min(1, "Choose a display type"),
  description: z.string().trim().max(500, "Description must be under 500 characters").optional(),
});

type CategoryFormValues = z.infer<typeof categorySchema>;

const parentCategoryOptions = [
  { label: "No parent category", value: "none" },
  { label: "T-shirts", value: "101" },
  { label: "Trousers", value: "102" },
  { label: "Shoes", value: "103" },
];

const displayTypeOptions = [
  { label: "Products grid", value: "products-grid" },
  { label: "Feature collection", value: "feature-collection" },
  { label: "Editorial landing", value: "editorial-landing" },
];

function slugify(value: string) {
  return value
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

function validateImageFile(file: File) {
  if (!IMAGE_TYPES.has(file.type)) {
    return "Upload a PNG, JPG or WebP image";
  }

  if (file.size > IMAGE_MAX_BYTES) {
    return "Thumbnail must be 5MB or smaller";
  }

  return null;
}

function SectionHeader({ title, description }: { title: string; description: string }) {
  return (
    <div className="lg:col-span-1">
      <h2 className="text-base font-semibold">{title}</h2>
      <p className="mt-1 max-w-xs text-sm text-muted-foreground">{description}</p>
    </div>
  );
}

export default function CreateCategoryForm() {
  const navigate = useNavigate();
  const [thumbnail, setThumbnail] = useState<File>();
  const [thumbnailPreview, setThumbnailPreview] = useState<string>();
  const [isDragging, setIsDragging] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const form = useForm<CategoryFormValues>({
    resolver: zodResolver(categorySchema),
    defaultValues: {
      name: "",
      slug: "",
      parentCategory: "none",
      displayType: "",
      description: "",
    },
  });

  const watchedName = form.watch("name");

  useEffect(() => {
    form.setValue("slug", slugify(watchedName), { shouldValidate: watchedName.length > 0 });
  }, [form, watchedName]);

  useEffect(() => {
    if (!thumbnail) {
      setThumbnailPreview(undefined);
      return;
    }

    const preview = URL.createObjectURL(thumbnail);
    setThumbnailPreview(preview);

    return () => URL.revokeObjectURL(preview);
  }, [thumbnail]);

  const thumbnailLabel = useMemo(() => {
    if (!thumbnail) return "Drop or select thumbnail";

    return thumbnail.name;
  }, [thumbnail]);

  function setThumbnailFile(file: File) {
    const error = validateImageFile(file);

    if (error) {
      toast.error(error);
      return;
    }

    setThumbnail(file);
  }

  function onThumbnailChange(event: ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    if (file) setThumbnailFile(file);
  }

  function onThumbnailDrop(event: DragEvent<HTMLButtonElement>) {
    event.preventDefault();
    setIsDragging(false);

    const file = event.dataTransfer.files?.[0];
    if (file) setThumbnailFile(file);
  }

  const createMutation = useMutation({
    mutationFn: async (values: CategoryFormValues) => {
      if (!thumbnail) {
        throw new Error("Select a thumbnail image before creating the category");
      }

      const imageLink = await uploadCategoryImage(thumbnail);
      const parentId =
        values.parentCategory && values.parentCategory !== "none"
          ? Number(values.parentCategory)
          : undefined;

      return createCategory({
        name: values.name.trim(),
        slug: values.slug.trim(),
        imageLink,
        parentId,
        ...(values.description?.trim() ? { description: values.description.trim() } : {}),
      });
    },
    onSuccess: (category) => {
      toast.success(`Created ${category.name}`, {
        id: "create-category-success",
      });
      form.reset();
      setThumbnail(undefined);
      void navigate({ to: "/categories/create" });
    },
    onError: (error) => {
      toast.error(error.message, {
        id: "create-category-error",
      });
    },
  });

  function saveDraft(values: CategoryFormValues) {
    console.info("Category draft", { ...values, thumbnail: thumbnail?.name });
    toast.success("Category saved as draft");
  }

  return (
    <form
      className="flex min-h-[calc(100dvh-8rem)] flex-col"
      onSubmit={form.handleSubmit((values) => createMutation.mutate(values))}
    >
      <div className="mb-8 flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">Create A Category</h1>
          <p className="mt-1 text-sm text-muted-foreground">
            Add a category for storefront navigation and merchandising.
          </p>
        </div>

        <Button type="button" variant="outline" onClick={() => window.history.back()}>
          Cancel
        </Button>
      </div>

      <div className="grid gap-8 border-b pb-10 lg:grid-cols-[minmax(14rem,0.45fr)_minmax(0,1fr)]">
        <SectionHeader
          title="Add new category"
          description="Set the public category details and storefront presentation."
        />

        <div className="grid gap-5 lg:grid-cols-2">
          <FormField
            control={form.control}
            name="name"
            label="Category Name"
            placeholder="Category name"
            required
          />
          <FormField control={form.control} name="slug" label="Slug" placeholder="slug" required />
          <FormField
            control={form.control}
            name="parentCategory"
            type="select"
            label="Parent Category"
            placeholder="Select..."
            options={parentCategoryOptions}
          />
          <FormField
            control={form.control}
            name="displayType"
            type="select"
            label="Display Type"
            placeholder="Select..."
            options={displayTypeOptions}
            required
          />
          <div className="lg:col-span-2">
            <FormField
              control={form.control}
              name="description"
              type="textarea"
              label="Description"
              placeholder="Describe how this category should appear to shoppers."
              className="min-h-32 resize-y"
            />
          </div>
        </div>
      </div>

      <div className="grid gap-8 border-b py-10 lg:grid-cols-[minmax(14rem,0.45fr)_minmax(0,1fr)]">
        <SectionHeader
          title="Upload thumbnail image"
          description="Use a square PNG, JPG or WebP image up to 5MB."
        />

        <div className="grid gap-4 md:grid-cols-[minmax(0,1fr)_11rem] md:items-stretch">
          <input
            ref={inputRef}
            type="file"
            accept="image/png,image/jpeg,image/webp"
            className="sr-only"
            onChange={onThumbnailChange}
          />
          <button
            type="button"
            onClick={() => inputRef.current?.click()}
            onDragOver={(event) => {
              event.preventDefault();
              setIsDragging(true);
            }}
            onDragLeave={() => setIsDragging(false)}
            onDrop={onThumbnailDrop}
            className={cn(
              "flex min-h-28 items-center justify-center gap-3 rounded-xl border border-dashed bg-background px-4 text-sm font-medium outline-none transition-colors hover:bg-muted focus-visible:ring-3 focus-visible:ring-ring/30",
              isDragging && "border-primary bg-primary/5",
            )}
          >
            <span className="flex size-10 items-center justify-center rounded-full bg-muted">
              <UploadSimpleIcon className="size-5 text-muted-foreground" aria-hidden />
            </span>
            <span className="truncate">{thumbnailLabel}</span>
          </button>

          <div className="flex min-h-28 items-center justify-center overflow-hidden rounded-xl border bg-muted/40">
            {thumbnailPreview ? (
              <img src={thumbnailPreview} alt="" className="h-full w-full object-cover" />
            ) : (
              <ImageSquareIcon className="size-9 text-muted-foreground" aria-hidden />
            )}
          </div>

          {thumbnail && (
            <div className="md:col-span-2">
              <Button
                type="button"
                variant="ghost"
                size="sm"
                onClick={() => setThumbnail(undefined)}
              >
                <XIcon className="size-4" />
                Remove thumbnail
              </Button>
            </div>
          )}
        </div>
      </div>

      <div className="mt-auto flex flex-col-reverse gap-3 pt-8 sm:flex-row sm:justify-end">
        <Button type="button" variant="outline" onClick={form.handleSubmit(saveDraft)}>
          Save as Draft
        </Button>
        <Button
          type="submit"
          className="bg-blue-600 hover:bg-blue-700"
          disabled={createMutation.isPending}
        >
          {createMutation.isPending ? "Creating..." : "Create Category"}
        </Button>
      </div>
    </form>
  );
}
