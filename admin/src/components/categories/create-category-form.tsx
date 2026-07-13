"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useEffect, useMemo, useState } from "react";
import { useForm } from "react-hook-form";
import { toast } from "sonner";
import { z } from "zod";

import { categoriesQueryKey, createCategory, listCategories } from "#/api/categories.ts";
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
import { slugify } from "#/utils/string-utils";
import { titleCase } from "#/utils/formatters";
import FormField from "#/components/form-field";
import { Button } from "#/components/ui/button";

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

const displayTypeOptions = [
  { label: "Products grid", value: "products-grid" },
  { label: "Feature collection", value: "feature-collection" },
  { label: "Editorial landing", value: "editorial-landing" },
];

export default function CreateCategoryForm() {
  const navigate = useNavigate();
  const [thumbnail, setThumbnail] = useState<ImageDraft>();
  const categoriesQuery = useQuery({
    queryKey: [...categoriesQueryKey, { limit: 100 }],
    queryFn: () => listCategories({ limit: 100 }),
  });
  const parentCategoryOptions = useMemo(
    () => [
      { label: "No parent category", value: "none" },
      ...(categoriesQuery.data?.data ?? []).map((category) => ({
        label: titleCase(category.name),
        value: String(category.id),
      })),
    ],
    [categoriesQuery.data?.data],
  );

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

  useEffect(
    () => () => {
      if (thumbnail) URL.revokeObjectURL(thumbnail.preview);
    },
    [thumbnail],
  );

  function setThumbnailFile(file: File) {
    setThumbnail(createImageDraft(file));
  }

  const createMutation = useMutation({
    mutationFn: async (values: CategoryFormValues) => {
      if (!thumbnail) {
        throw new Error("Select a thumbnail image before creating the category");
      }

      const uploadedImage = await uploadCategoryImage(thumbnail.file);
      const parentId =
        values.parentCategory && values.parentCategory !== "none"
          ? Number(values.parentCategory)
          : undefined;

      return createCategory({
        name: values.name.trim(),
        slug: values.slug.trim(),
        imageLink: uploadedImage.imageLink,
        mediaAssetPid: uploadedImage.assetPid,
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
    console.info("Category draft", { ...values, thumbnail: thumbnail?.file.name });
    toast.success("Category saved as draft");
  }

  return (
    <form
      className="flex min-h-[calc(100dvh-8rem)] flex-col"
      onSubmit={form.handleSubmit((values) => createMutation.mutate(values))}
    >
      <CatalogueFormHeader
        title="Create A Category"
        description="Add a category for storefront navigation and merchandising."
      />

      <CatalogueFormSection
        title="Add new category"
        description="Set the public category details and storefront presentation."
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
      </CatalogueFormSection>

      <CatalogueFormSection
        title="Upload thumbnail image"
        description="Use a square PNG, JPG or WebP image up to 5MB."
      >
        <div className="grid gap-4 md:grid-cols-[minmax(0,1fr)_11rem] md:items-stretch">
          <ImageDropzone
            label={thumbnail?.file.name ?? "Drop or select thumbnail"}
            onFiles={(files) => {
              const file = files[0];
              if (file) setThumbnailFile(file);
            }}
          />

          <ImagePreviewSlot preview={thumbnail?.preview} />

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
                Remove thumbnail
              </Button>
            </div>
          )}
        </div>
      </CatalogueFormSection>

      <CatalogueFormActions
        submitLabel="Create Category"
        pendingSubmitLabel="Creating..."
        isPending={createMutation.isPending}
        onDraft={form.handleSubmit(saveDraft)}
      />
    </form>
  );
}
