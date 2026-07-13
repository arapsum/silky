"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useEffect, useMemo, useState } from "react";
import { useForm } from "react-hook-form";
import { toast } from "sonner";
import { z } from "zod";

import {
  categoriesQueryKey,
  getCategory,
  listCategories,
  updateCategory,
} from "#/api/categories.ts";
import { uploadCategoryImage } from "#/api/uploads.ts";
import {
  CatalogueFormActions,
  CatalogueFormHeader,
  CatalogueFormSection,
} from "#/components/catalogue/form-layout";
import {
  createImageDraft,
  ImageDropzone,
  ImagePreviewSlot,
  type ImageDraft,
} from "#/components/catalogue/image-upload";
import { slugify, titleCase } from "#/components/catalogue/string-utils";
import { EmptyState } from "#/components/empty-state";
import { ErrorState } from "#/components/error-state";
import FormField from "#/components/form-field";
import { Button } from "#/components/ui/button";
import { Skeleton } from "#/components/ui/skeleton";

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
  parentCategory: z.string(),
  description: z.string().trim().max(500, "Description must be under 500 characters"),
});

type CategoryFormValues = z.infer<typeof categorySchema>;

function isDescendantOf(
  categoryId: number,
  possibleParentId: number,
  categories: { id: number; parentId: number | null }[],
) {
  const categoriesById = new Map(categories.map((category) => [category.id, category]));
  const visited = new Set<number>();
  let current = categoriesById.get(possibleParentId);

  while (current && current.parentId !== null && !visited.has(current.id)) {
    if (current.parentId === categoryId) return true;

    visited.add(current.id);
    current = categoriesById.get(current.parentId);
  }

  return false;
}

function categoryValues(category: {
  name: string;
  slug: string;
  parentId: number | null;
  description: string | null;
}): CategoryFormValues {
  return {
    name: category.name,
    slug: category.slug,
    parentCategory: category.parentId ? String(category.parentId) : "none",
    description: category.description ?? "",
  };
}

export default function EditCategoryForm({ pid }: { pid: string }) {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [thumbnail, setThumbnail] = useState<ImageDraft>();
  const form = useForm<CategoryFormValues>({
    resolver: zodResolver(categorySchema),
    defaultValues: {
      name: "",
      slug: "",
      parentCategory: "none",
      description: "",
    },
  });
  const watchedName = form.watch("name");
  const nameChanged = form.formState.dirtyFields.name;

  const categoryQuery = useQuery({
    queryKey: ["category", pid],
    queryFn: () => getCategory(pid),
  });
  const categoriesQuery = useQuery({
    queryKey: [...categoriesQueryKey, { limit: 100 }],
    queryFn: () => listCategories({ limit: 100 }),
  });

  const category = categoryQuery.data;
  const categories = categoriesQuery.data?.data ?? [];
  const parentCategoryOptions = useMemo(
    () => [
      { label: "No parent category", value: "none" },
      ...categories
        .filter(
          (candidate) =>
            candidate.id !== category?.id &&
            !isDescendantOf(category?.id ?? 0, candidate.id, categories),
        )
        .map((candidate) => ({
          label: titleCase(candidate.name),
          value: String(candidate.id),
        })),
    ],
    [categories, category?.id],
  );

  useEffect(() => {
    if (!category) return;

    form.reset(categoryValues(category));
  }, [category, form]);

  useEffect(() => {
    if (!nameChanged) return;

    form.setValue("slug", slugify(watchedName), { shouldValidate: watchedName.length > 0 });
  }, [form, nameChanged, watchedName]);

  useEffect(
    () => () => {
      if (thumbnail) URL.revokeObjectURL(thumbnail.preview);
    },
    [thumbnail],
  );

  function setThumbnailFile(file: File) {
    const nextThumbnail = createImageDraft(file);

    setThumbnail((current) => {
      if (current) URL.revokeObjectURL(current.preview);
      return nextThumbnail;
    });
  }

  const updateMutation = useMutation({
    mutationFn: async (values: CategoryFormValues) => {
      if (!category) throw new Error("Category is unavailable");

      const uploadedImage = thumbnail ? await uploadCategoryImage(thumbnail.file) : undefined;
      const parentId = values.parentCategory === "none" ? undefined : Number(values.parentCategory);

      return updateCategory(pid, {
        name: values.name.trim(),
        slug: values.slug.trim(),
        imageLink: uploadedImage?.imageLink ?? category.imageLink,
        ...(uploadedImage
          ? { mediaAssetPid: uploadedImage.assetPid }
          : category.mediaAssetPid
            ? { mediaAssetPid: category.mediaAssetPid }
            : {}),
        ...(parentId ? { parentId } : { clearParent: true }),
        description: values.description.trim(),
      });
    },
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: categoriesQueryKey }),
        queryClient.invalidateQueries({ queryKey: ["category", pid] }),
        queryClient.invalidateQueries({ queryKey: ["category-detail", pid] }),
      ]);
      toast.success("Category updated", { id: "update-category-success" });
      void navigate({ to: "/categories/$pid", params: { pid } });
    },
    onError: (error) => toast.error(error.message, { id: "update-category-error" }),
  });

  function resetChanges() {
    if (!category) return;

    form.reset(categoryValues(category));
    setThumbnail((current) => {
      if (current) URL.revokeObjectURL(current.preview);
      return undefined;
    });
  }

  if (categoryQuery.isLoading) {
    return (
      <div className="space-y-8 pb-10">
        <Skeleton className="h-16 w-full rounded-lg" />
        <Skeleton className="h-72 w-full rounded-lg" />
        <Skeleton className="h-64 w-full rounded-lg" />
      </div>
    );
  }

  if (categoryQuery.isError) {
    return (
      <ErrorState
        title="Category could not be loaded"
        description={categoryQuery.error.message}
        onRetry={() => categoryQuery.refetch()}
      />
    );
  }

  if (!category) {
    return (
      <EmptyState
        title="Category not found"
        description="The requested category no longer exists or is unavailable."
        action={
          <Button
            type="button"
            className="rounded-lg"
            onClick={() => void navigate({ to: "/categories" })}
          >
            Back to Categories
          </Button>
        }
      />
    );
  }

  const preview = thumbnail?.preview ?? category.imageLink;

  return (
    <form
      className="flex min-h-[calc(100dvh-8rem)] flex-col"
      onSubmit={form.handleSubmit((values) => updateMutation.mutate(values))}
    >
      <CatalogueFormHeader
        title={`Edit ${titleCase(category.name)}`}
        description="Update the category details and storefront presentation."
        onCancel={() => void navigate({ to: "/categories/$pid", params: { pid } })}
      />

      <CatalogueFormSection
        title="Category details"
        description="Keep the catalogue name, placement, and description accurate."
        contentClassName="grid gap-5 lg:grid-cols-2"
        paddedTop={false}
      >
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
          placeholder={categoriesQuery.isLoading ? "Loading..." : "Select..."}
          options={parentCategoryOptions}
          disabled={categoriesQuery.isLoading}
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
      </CatalogueFormSection>

      <CatalogueFormSection
        title="Thumbnail image"
        description="Replace the thumbnail only when the existing image is no longer suitable."
      >
        <div className="grid gap-4 md:grid-cols-[minmax(0,1fr)_11rem] md:items-stretch">
          <ImageDropzone
            label={thumbnail?.file.name ?? "Replace thumbnail"}
            onFiles={(files) => {
              const file = files[0];
              if (file) setThumbnailFile(file);
            }}
          />

          <ImagePreviewSlot preview={preview} />

          {thumbnail && (
            <div className="md:col-span-2">
              <Button
                type="button"
                variant="ghost"
                size="sm"
                onClick={() => {
                  URL.revokeObjectURL(thumbnail.preview);
                  setThumbnail(undefined);
                }}
              >
                Keep existing thumbnail
              </Button>
            </div>
          )}
        </div>
      </CatalogueFormSection>

      <CatalogueFormActions
        draftLabel="Reset changes"
        submitLabel="Save changes"
        pendingSubmitLabel="Saving..."
        isPending={updateMutation.isPending}
        onDraft={resetChanges}
      />
    </form>
  );
}
