"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { PackageIcon, PlusIcon, TrashIcon } from "@phosphor-icons/react";
import { useMutation, useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useEffect, useMemo, useRef, useState, type ComponentProps } from "react";
import { useForm } from "react-hook-form";
import { toast } from "sonner";
import { z } from "zod";

import { categoriesQueryKey, listCategories } from "#/api/categories.ts";
import {
  createProduct,
  listProductAttributes,
  productAttributesQueryKey,
  type ProductAttributeWithValues,
  type ProductInput,
  type ProductPictureInput,
  type ProductVariantInput,
} from "#/api/products.ts";
import { uploadProductImage } from "#/api/uploads.ts";
import {
  CatalogueFormActions,
  CatalogueFormHeader,
  CatalogueFormSection,
} from "#/components/catalogue/form-layout";
import {
  createImageDraft,
  ImageDropzone,
  ImageGrid,
  ImagePreviewSlot,
  type ImageDraft,
} from "#/components/catalogue/image-upload";
import { titleCase } from "#/components/catalogue/string-utils";
import { EmptyState } from "#/components/empty-state";
import { ErrorState } from "#/components/error-state";
import FormField from "#/components/form-field";
import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";
import { Label } from "#/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger } from "#/components/ui/select";
import { cn } from "#/lib/utils";

const productSchema = z.object({
  name: z
    .string()
    .trim()
    .min(2, "Product name requires 2 letters")
    .max(255, "Product name must be under 255 characters"),
  categoryId: z.string().min(1, "Choose a category"),
  description: z.string().trim().max(2000, "Description must be under 2000 characters").optional(),
  defaultSku: z.string().trim().min(1, "Default SKU is required").max(64, "SKU is too long"),
  defaultPrice: z
    .string()
    .trim()
    .regex(/^\d+(\.\d{1,2})?$/, "Use a valid price"),
  defaultStockQuantity: z.string().trim().regex(/^\d+$/, "Use a whole stock quantity"),
});

type ProductFormValues = z.infer<typeof productSchema>;

type DefaultOptionDraft = {
  id: string;
  attributeId: string;
  attributeValueId: string;
};

type VariantDraft = {
  id: string;
  sku: string;
  priceOverride: string;
  stockOverride: string;
  optionOverrides: Record<string, string>;
  image?: ImageDraft;
};

type VariantInputFieldProps = Omit<ComponentProps<typeof Input>, "id" | "onChange" | "value"> & {
  id: string;
  label: string;
  value: string;
  onValueChange: (value: string) => void;
};

function draftId() {
  return crypto.randomUUID();
}

function attributeValues(attributes: ProductAttributeWithValues[], attributeId: string) {
  const attribute = attributes.find((entry) => String(entry.attribute.id) === attributeId);

  return attribute?.values ?? [];
}

function optionLabel(attributes: ProductAttributeWithValues[], attributeId: string) {
  const attribute = attributes.find((entry) => String(entry.attribute.id) === attributeId);

  return attribute ? titleCase(attribute.attribute.name) : "Option";
}

function selectedValueLabel(
  attributes: ProductAttributeWithValues[],
  attributeId: string,
  attributeValueId: string,
) {
  const value = attributeValues(attributes, attributeId).find(
    (entry) => String(entry.id) === attributeValueId,
  );

  return value?.value ?? "Inherited";
}

function selectLabel(value: string, fallback: string) {
  return (
    <span className={cn("flex flex-1 text-left", !value && "text-muted-foreground")}>
      {value || fallback}
    </span>
  );
}

function VariantInputField({ id, label, value, onValueChange, ...props }: VariantInputFieldProps) {
  return (
    <div className="space-y-2">
      <Label htmlFor={id}>{label}</Label>
      <Input
        id={id}
        value={value}
        onChange={(event) => onValueChange(event.target.value)}
        {...props}
      />
    </div>
  );
}

async function uploadPictures(images: ImageDraft[]): Promise<ProductPictureInput[]> {
  const urls = await Promise.all(images.map((image) => uploadProductImage(image.file)));

  return urls.map((imageLink, index) => ({
    imageLink,
    displayOrder: index + 1,
  }));
}

export default function CreateProductForm() {
  const navigate = useNavigate();
  const previewUrlsRef = useRef<string[]>([]);
  const [productImages, setProductImages] = useState<ImageDraft[]>([]);
  const [defaultOptions, setDefaultOptions] = useState<DefaultOptionDraft[]>([]);
  const [variants, setVariants] = useState<VariantDraft[]>([]);

  const form = useForm<ProductFormValues>({
    resolver: zodResolver(productSchema),
    defaultValues: {
      name: "",
      categoryId: "",
      description: "",
      defaultSku: "",
      defaultPrice: "",
      defaultStockQuantity: "0",
    },
  });

  useEffect(() => {
    return () => {
      previewUrlsRef.current.forEach((url) => URL.revokeObjectURL(url));
    };
  }, []);

  const categoriesQuery = useQuery({
    queryKey: [...categoriesQueryKey, { limit: 100 }],
    queryFn: () => listCategories({ limit: 100 }),
  });

  const attributesQuery = useQuery({
    queryKey: productAttributesQueryKey,
    queryFn: listProductAttributes,
  });

  const categoryOptions = useMemo(
    () =>
      (categoriesQuery.data?.data ?? []).map((category) => ({
        label: titleCase(category.name),
        value: String(category.id),
      })),
    [categoriesQuery.data?.data],
  );

  const attributes = attributesQuery.data ?? [];

  function addPreview(file: File) {
    const draft = createImageDraft(file);
    previewUrlsRef.current.push(draft.preview);
    setProductImages((current) => [...current, draft]);
  }

  function removeProductImage(image: ImageDraft) {
    URL.revokeObjectURL(image.preview);
    previewUrlsRef.current = previewUrlsRef.current.filter((url) => url !== image.preview);
    setProductImages((current) => current.filter((entry) => entry.id !== image.id));
  }

  function addDefaultOption() {
    setDefaultOptions((current) => [
      ...current,
      {
        id: draftId(),
        attributeId: "",
        attributeValueId: "",
      },
    ]);
  }

  function updateDefaultOption(id: string, patch: Partial<DefaultOptionDraft>) {
    setDefaultOptions((current) =>
      current.map((option) => (option.id === id ? { ...option, ...patch } : option)),
    );
  }

  function removeDefaultOption(id: string) {
    setDefaultOptions((current) => current.filter((option) => option.id !== id));
  }

  function addVariant() {
    setVariants((current) => [
      ...current,
      {
        id: draftId(),
        sku: "",
        priceOverride: "",
        stockOverride: "",
        optionOverrides: {},
      },
    ]);
  }

  function updateVariant(id: string, patch: Partial<VariantDraft>) {
    setVariants((current) =>
      current.map((variant) => (variant.id === id ? { ...variant, ...patch } : variant)),
    );
  }

  function removeVariant(variant: VariantDraft) {
    if (variant.image) {
      URL.revokeObjectURL(variant.image.preview);
      previewUrlsRef.current = previewUrlsRef.current.filter(
        (url) => url !== variant.image?.preview,
      );
    }

    setVariants((current) => current.filter((entry) => entry.id !== variant.id));
  }

  function setVariantImage(variant: VariantDraft, file: File) {
    if (variant.image) {
      URL.revokeObjectURL(variant.image.preview);
      previewUrlsRef.current = previewUrlsRef.current.filter(
        (url) => url !== variant.image?.preview,
      );
    }

    const image = createImageDraft(file);
    previewUrlsRef.current.push(image.preview);
    updateVariant(variant.id, { image });
  }

  function buildOptions() {
    const seenAttributes = new Set<string>();

    return defaultOptions.map((option, index) => {
      if (!option.attributeId || !option.attributeValueId) {
        throw new Error("Complete every default option before creating the product");
      }

      if (seenAttributes.has(option.attributeId)) {
        throw new Error("Each product option can only be selected once");
      }

      seenAttributes.add(option.attributeId);

      return {
        attributeId: Number(option.attributeId),
        attributeValueId: Number(option.attributeValueId),
        displayOrder: index + 1,
      };
    });
  }

  async function buildPayload(values: ProductFormValues): Promise<ProductInput> {
    const defaultSku = values.defaultSku.trim();
    const defaultPrice = values.defaultPrice.trim();
    const defaultStockQuantity = Number(values.defaultStockQuantity);
    const options = buildOptions();
    const skuSet = new Set([defaultSku.toLowerCase()]);

    const productPictures = await uploadPictures(productImages);
    const extraVariants: ProductVariantInput[] = [];

    for (const variant of variants) {
      const sku = variant.sku.trim();

      if (!sku) {
        throw new Error("Every additional variant needs a SKU");
      }

      const normalizedSku = sku.toLowerCase();

      if (skuSet.has(normalizedSku)) {
        throw new Error("Variant SKUs must be unique");
      }

      skuSet.add(normalizedSku);

      const pictures = variant.image ? await uploadPictures([variant.image]) : [];

      extraVariants.push({
        sku,
        price: variant.priceOverride.trim() || defaultPrice,
        stockQuantity: variant.stockOverride.trim()
          ? Number(variant.stockOverride)
          : defaultStockQuantity,
        isDefault: false,
        options: options.map((option) => ({
          ...option,
          attributeValueId:
            Number(variant.optionOverrides[String(option.attributeId)]) || option.attributeValueId,
        })),
        ...(pictures.length ? { pictures } : {}),
      });
    }

    const defaultVariant: ProductVariantInput = {
      sku: defaultSku,
      price: defaultPrice,
      stockQuantity: defaultStockQuantity,
      isDefault: true,
      options,
    };

    return {
      categoryId: Number(values.categoryId),
      name: values.name.trim(),
      ...(values.description?.trim() ? { description: values.description.trim() } : {}),
      ...(productPictures.length ? { pictures: productPictures } : {}),
      variants: [defaultVariant, ...extraVariants],
    };
  }

  const createMutation = useMutation({
    mutationFn: async (values: ProductFormValues) => createProduct(await buildPayload(values)),
    onSuccess: async () => {
      toast.success("Product created", {
        id: "create-product-success",
      });
      form.reset();
      setProductImages([]);
      setDefaultOptions([]);
      setVariants([]);
      await navigate({ to: "/products/create" });
    },
    onError: (error) => {
      toast.error(error.message, {
        id: "create-product-error",
      });
    },
  });

  function saveDraft(values: ProductFormValues) {
    console.info("Product draft", {
      ...values,
      productImages: productImages.map((image) => image.file.name),
      defaultOptions,
      variants,
    });
    toast.success("Product saved as draft");
  }

  if (attributesQuery.isError) {
    return (
      <ErrorState
        icon={<PackageIcon className="size-8" />}
        title="Unable to load product options"
        description={attributesQuery.error.message}
        onRetry={() => void attributesQuery.refetch()}
      />
    );
  }

  return (
    <form
      className="flex min-h-[calc(100dvh-8rem)] flex-col"
      onSubmit={form.handleSubmit((values) => createMutation.mutate(values))}
    >
      <CatalogueFormHeader
        title="Create A Product"
        description="Add catalogue details, default pricing, stock, and variant options."
      />

      <CatalogueFormSection
        title="Product details"
        description="Set the product identity and the category shoppers will find it under."
        contentClassName="grid gap-5 lg:grid-cols-2"
        paddedTop={false}
      >
        <FormField
          control={form.control}
          name="name"
          label="Product Name"
          placeholder="Product name"
          required
        />
        <FormField
          control={form.control}
          name="categoryId"
          type="select"
          label="Category"
          placeholder={categoriesQuery.isLoading ? "Loading..." : "Select category"}
          options={categoryOptions}
          disabled={categoriesQuery.isLoading}
          required
        />
        <div className="lg:col-span-2">
          <FormField
            control={form.control}
            name="description"
            type="textarea"
            label="Description"
            placeholder="Describe this product for catalogue teams and shoppers."
            className="min-h-36 resize-y"
          />
        </div>
      </CatalogueFormSection>

      <CatalogueFormSection
        title="Default variant"
        description="Use the base SKU, price, stock, and options that other variants inherit from."
        contentClassName="grid gap-5 lg:grid-cols-3"
      >
        <FormField
          control={form.control}
          name="defaultSku"
          label="SKU"
          placeholder="SKU-001"
          required
        />
        <FormField
          control={form.control}
          name="defaultPrice"
          type="text"
          inputMode="decimal"
          label="Price"
          placeholder="59.99"
          required
        />
        <FormField
          control={form.control}
          name="defaultStockQuantity"
          type="number"
          min={0}
          label="Stock"
          placeholder="0"
          required
        />

        <div className="space-y-4 lg:col-span-3">
          <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
            <div>
              <Label className="text-sm font-medium">Options</Label>
              <p className="mt-1 text-sm text-muted-foreground">
                Order controls how options are displayed to shoppers.
              </p>
            </div>
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={addDefaultOption}
              disabled={attributesQuery.isLoading || attributes.length === 0}
            >
              <PlusIcon className="size-4" />
              Add Option
            </Button>
          </div>

          {defaultOptions.length === 0 ? (
            <div className="flex min-h-28 items-center justify-center border bg-muted/20 px-6 text-center text-sm text-muted-foreground">
              No default options selected.
            </div>
          ) : (
            <div className="space-y-3">
              {defaultOptions.map((option, index) => {
                const selectedAttributeIds = new Set(
                  defaultOptions
                    .filter((entry) => entry.id !== option.id)
                    .map((entry) => entry.attributeId)
                    .filter(Boolean),
                );
                const values = attributeValues(attributes, option.attributeId);

                return (
                  <div
                    key={option.id}
                    className="grid gap-3 border bg-background p-3 md:grid-cols-[5rem_minmax(0,1fr)_minmax(0,1fr)_2.5rem]"
                  >
                    <div className="space-y-2">
                      <Label>Order</Label>
                      <div className="flex h-9 items-center rounded-3xl bg-input/50 px-3 text-sm">
                        {index + 1}
                      </div>
                    </div>

                    <div className="space-y-2">
                      <Label>Attribute</Label>
                      <Select
                        value={option.attributeId}
                        onValueChange={(attributeId) =>
                          updateDefaultOption(option.id, {
                            attributeId: attributeId ?? "",
                            attributeValueId: "",
                          })
                        }
                      >
                        <SelectTrigger className="w-full">
                          {selectLabel(
                            option.attributeId ? optionLabel(attributes, option.attributeId) : "",
                            "Select attribute",
                          )}
                        </SelectTrigger>
                        <SelectContent>
                          {attributes.map((entry) => (
                            <SelectItem
                              key={entry.attribute.id}
                              value={String(entry.attribute.id)}
                              disabled={selectedAttributeIds.has(String(entry.attribute.id))}
                            >
                              {titleCase(entry.attribute.name)}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    </div>

                    <div className="space-y-2">
                      <Label>Default Value</Label>
                      <Select
                        value={option.attributeValueId}
                        onValueChange={(attributeValueId) =>
                          updateDefaultOption(option.id, {
                            attributeValueId: attributeValueId ?? "",
                          })
                        }
                        disabled={!option.attributeId}
                      >
                        <SelectTrigger className="w-full">
                          {selectLabel(
                            option.attributeValueId
                              ? selectedValueLabel(
                                  attributes,
                                  option.attributeId,
                                  option.attributeValueId,
                                )
                              : "",
                            "Select value",
                          )}
                        </SelectTrigger>
                        <SelectContent>
                          {values.map((value) => (
                            <SelectItem key={value.id} value={String(value.id)}>
                              {value.value}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    </div>

                    <div className="space-y-2">
                      <Label className="invisible" aria-hidden="true">
                        Remove
                      </Label>
                      <Button
                        type="button"
                        variant="outline"
                        size="icon"
                        aria-label="Remove option"
                        onClick={() => removeDefaultOption(option.id)}
                      >
                        <TrashIcon className="size-4" />
                      </Button>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </CatalogueFormSection>

      <CatalogueFormSection
        title="Product media"
        description="Upload product-level images. Variant-specific images can be added on variant rows."
      >
        <div className="space-y-4">
          <ImageDropzone
            label="Drop or select product images"
            multiple
            className="w-full"
            onFiles={(files) => files.forEach(addPreview)}
          />
          <ImageGrid images={productImages} onRemove={removeProductImage} />
        </div>
      </CatalogueFormSection>

      <CatalogueFormSection
        title="Additional variants"
        description="Add SKUs that differ from the default variant by price, stock, option values, or image."
      >
        <div className="space-y-4">
          <div className="flex justify-end">
            <Button type="button" variant="outline" size="sm" onClick={addVariant}>
              <PlusIcon className="size-4" />
              Add Variant
            </Button>
          </div>

          {variants.length === 0 ? (
            <EmptyState
              icon={<PackageIcon className="size-8" />}
              title="No additional variants"
              description="The default variant will be created as the sellable SKU."
              className="min-h-40"
            />
          ) : (
            <div className="space-y-4">
              {variants.map((variant, index) => (
                <div key={variant.id} className="space-y-5 border bg-background p-4">
                  <div className="flex items-center justify-between gap-3">
                    <div>
                      <h3 className="font-semibold">Variant {index + 1}</h3>
                      <p className="text-sm text-muted-foreground">
                        Blank price and stock fields inherit the default variant.
                      </p>
                    </div>
                    <Button
                      type="button"
                      variant="outline"
                      size="icon-sm"
                      aria-label="Remove variant"
                      onClick={() => removeVariant(variant)}
                    >
                      <TrashIcon className="size-4" />
                    </Button>
                  </div>

                  <div className="grid gap-4 lg:grid-cols-3">
                    <VariantInputField
                      id={`variant-${variant.id}-sku`}
                      label="SKU"
                      value={variant.sku}
                      placeholder="SKU-002"
                      onValueChange={(sku) => updateVariant(variant.id, { sku })}
                    />
                    <VariantInputField
                      id={`variant-${variant.id}-price`}
                      label="Price Override"
                      value={variant.priceOverride}
                      inputMode="decimal"
                      placeholder={form.watch("defaultPrice") || "Inherited"}
                      onValueChange={(priceOverride) =>
                        updateVariant(variant.id, { priceOverride })
                      }
                    />
                    <VariantInputField
                      id={`variant-${variant.id}-stock`}
                      label="Stock Override"
                      type="number"
                      min={0}
                      value={variant.stockOverride}
                      placeholder={form.watch("defaultStockQuantity") || "Inherited"}
                      onValueChange={(stockOverride) =>
                        updateVariant(variant.id, { stockOverride })
                      }
                    />
                  </div>

                  {defaultOptions.length > 0 && (
                    <div className="grid gap-4 lg:grid-cols-2">
                      {defaultOptions
                        .filter((option) => option.attributeId && option.attributeValueId)
                        .map((option) => {
                          const values = attributeValues(attributes, option.attributeId);

                          return (
                            <div key={option.id} className="space-y-2">
                              <Label>{optionLabel(attributes, option.attributeId)}</Label>
                              <Select
                                value={variant.optionOverrides[option.attributeId] ?? ""}
                                onValueChange={(attributeValueId) =>
                                  updateVariant(variant.id, {
                                    optionOverrides: {
                                      ...variant.optionOverrides,
                                      [option.attributeId]: attributeValueId ?? "",
                                    },
                                  })
                                }
                              >
                                <SelectTrigger className="w-full">
                                  {selectLabel(
                                    selectedValueLabel(
                                      attributes,
                                      option.attributeId,
                                      variant.optionOverrides[option.attributeId] ??
                                        option.attributeValueId,
                                    ),
                                    selectedValueLabel(
                                      attributes,
                                      option.attributeId,
                                      option.attributeValueId,
                                    ),
                                  )}
                                </SelectTrigger>
                                <SelectContent>
                                  {values.map((value) => (
                                    <SelectItem key={value.id} value={String(value.id)}>
                                      {value.value}
                                    </SelectItem>
                                  ))}
                                </SelectContent>
                              </Select>
                            </div>
                          );
                        })}
                    </div>
                  )}

                  <div className="grid gap-4 md:grid-cols-[minmax(0,1fr)_8rem] md:items-stretch">
                    <ImageDropzone
                      label={variant.image?.file.name ?? "Variant image"}
                      className="min-h-24"
                      onFiles={(files) => {
                        const file = files[0];
                        if (file) setVariantImage(variant, file);
                      }}
                    />
                    <ImagePreviewSlot preview={variant.image?.preview} className="min-h-24" />
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </CatalogueFormSection>

      <CatalogueFormActions
        submitLabel="Create Product"
        pendingSubmitLabel="Creating..."
        isPending={createMutation.isPending}
        onDraft={form.handleSubmit(saveDraft)}
      />
    </form>
  );
}
