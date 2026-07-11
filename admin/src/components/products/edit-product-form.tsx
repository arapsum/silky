"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import {
  ArrowLeftIcon,
  CheckCircleIcon,
  PackageIcon,
  PlusIcon,
  TrashIcon,
} from "@phosphor-icons/react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { useEffect, useMemo, useState } from "react";
import { useForm } from "react-hook-form";
import { toast } from "sonner";
import { z } from "zod";

import { categoriesQueryKey, listCategories } from "#/api/categories.ts";
import {
  addProductPicture,
  addVariantPicture,
  createProductVariant,
  deleteProductPicture,
  deleteProductVariant,
  deleteVariantPicture,
  getProduct,
  listProductAttributes,
  listProductTags,
  productsQueryKey,
  productAttributesQueryKey,
  productTagsQueryKey,
  setDefaultProductVariant,
  updateProduct,
  updateProductPicture,
  updateProductVariant,
  updateVariantPicture,
  type ProductAttributeWithValues,
  type ProductPicture,
  type ProductVariantDetail,
} from "#/api/products.ts";
import {
  CatalogueFormActions,
  CatalogueFormHeader,
  CatalogueFormSection,
} from "#/components/catalogue/form-layout";
import { ImageDropzone } from "#/components/catalogue/image-upload";
import { titleCase } from "#/components/catalogue/string-utils";
import { EmptyState } from "#/components/empty-state";
import { ErrorState } from "#/components/error-state";
import FormField from "#/components/form-field";
import {
  ProductInformationFields,
  informationEntries,
  productInformation,
  type ProductInformationEntry,
} from "#/components/products/product-information-fields";
import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";
import { Label } from "#/components/ui/label";
import { Skeleton } from "#/components/ui/skeleton";
import { Select, SelectContent, SelectItem, SelectTrigger } from "#/components/ui/select";
import { uploadProductImage } from "#/api/uploads.ts";

const baseSchema = z.object({
  name: z.string().trim().min(2, "Product name requires 2 letters"),
  categoryId: z.string().min(1, "Choose a category"),
  description: z.string().trim().max(2000, "Description must be under 2000 characters").optional(),
  tagPids: z.array(z.string()),
});

type BaseValues = z.infer<typeof baseSchema>;

type VariantDraft = {
  sku: string;
  price: string;
  stockQuantity: string;
};

type NewVariantDraft = VariantDraft & {
  optionValues: Record<string, string>;
};

function optionValues(attributes: ProductAttributeWithValues[], attributeId: number) {
  return attributes.find((entry) => entry.attribute.id === attributeId)?.values ?? [];
}

function selectLabel(value: string, fallback: string) {
  return <span className={value ? undefined : "text-muted-foreground"}>{value || fallback}</span>;
}

function variantOptionsLabel(variant: ProductVariantDetail) {
  if (!variant.options.length) return "No options";

  return variant.options
    .map((option) => `${titleCase(option.attributeName)}: ${option.value}`)
    .join(" / ");
}

function ProductPictureRow({
  productPid,
  picture,
  onChanged,
}: {
  productPid: string;
  picture: ProductPicture;
  onChanged: () => void;
}) {
  const [displayOrder, setDisplayOrder] = useState(String(picture.displayOrder ?? ""));

  useEffect(() => {
    setDisplayOrder(String(picture.displayOrder ?? ""));
  }, [picture.displayOrder]);

  const updateMutation = useMutation({
    mutationFn: () =>
      updateProductPicture(productPid, picture.pid, {
        displayOrder: displayOrder.trim() ? Number(displayOrder) : null,
      }),
    onSuccess: () => {
      toast.success("Picture order updated");
      onChanged();
    },
    onError: (error) => toast.error(error.message),
  });

  const deleteMutation = useMutation({
    mutationFn: () => deleteProductPicture(productPid, picture.pid),
    onSuccess: () => {
      toast.success("Picture removed");
      onChanged();
    },
    onError: (error) => toast.error(error.message),
  });

  return (
    <div className="grid gap-3 border bg-background p-3 md:grid-cols-[5rem_minmax(0,1fr)_7rem_2.5rem] md:items-center">
      <div className="size-16 overflow-hidden bg-muted">
        <img src={picture.imageLink} alt="" className="h-full w-full object-cover" />
      </div>
      <p className="break-all text-sm text-muted-foreground">{picture.imageLink}</p>
      <Input
        value={displayOrder}
        type="number"
        min={0}
        placeholder="Order"
        onChange={(event) => setDisplayOrder(event.target.value)}
        onBlur={() => updateMutation.mutate()}
      />
      <Button
        type="button"
        variant="outline"
        size="icon"
        aria-label="Remove picture"
        disabled={deleteMutation.isPending}
        onClick={() => deleteMutation.mutate()}
      >
        <TrashIcon className="size-4" />
      </Button>
    </div>
  );
}

function VariantPictureRow({
  productPid,
  variantPid,
  picture,
  onChanged,
}: {
  productPid: string;
  variantPid: string;
  picture: ProductPicture;
  onChanged: () => void;
}) {
  const [displayOrder, setDisplayOrder] = useState(String(picture.displayOrder ?? ""));

  useEffect(() => {
    setDisplayOrder(String(picture.displayOrder ?? ""));
  }, [picture.displayOrder]);

  const updateMutation = useMutation({
    mutationFn: () =>
      updateVariantPicture(productPid, variantPid, picture.pid, {
        displayOrder: displayOrder.trim() ? Number(displayOrder) : null,
      }),
    onSuccess: () => {
      toast.success("Variant picture order updated");
      onChanged();
    },
    onError: (error) => toast.error(error.message),
  });

  const deleteMutation = useMutation({
    mutationFn: () => deleteVariantPicture(productPid, variantPid, picture.pid),
    onSuccess: () => {
      toast.success("Variant picture removed");
      onChanged();
    },
    onError: (error) => toast.error(error.message),
  });

  return (
    <div className="grid gap-3 border bg-background p-3 md:grid-cols-[5rem_minmax(0,1fr)_7rem_2.5rem] md:items-center">
      <div className="size-16 overflow-hidden bg-muted">
        <img src={picture.imageLink} alt="" className="h-full w-full object-cover" />
      </div>
      <p className="break-all text-sm text-muted-foreground">{picture.imageLink}</p>
      <Input
        value={displayOrder}
        type="number"
        min={0}
        placeholder="Order"
        onChange={(event) => setDisplayOrder(event.target.value)}
        onBlur={() => updateMutation.mutate()}
      />
      <Button
        type="button"
        variant="outline"
        size="icon"
        aria-label="Remove variant picture"
        disabled={deleteMutation.isPending}
        onClick={() => deleteMutation.mutate()}
      >
        <TrashIcon className="size-4" />
      </Button>
    </div>
  );
}

export default function EditProductForm({ pid }: { pid: string }) {
  const queryClient = useQueryClient();
  const [variantDrafts, setVariantDrafts] = useState<Record<string, VariantDraft>>({});
  const [information, setInformation] = useState<ProductInformationEntry[]>([]);
  const [newVariant, setNewVariant] = useState<NewVariantDraft>({
    sku: "",
    price: "",
    stockQuantity: "0",
    optionValues: {},
  });

  const form = useForm<BaseValues>({
    resolver: zodResolver(baseSchema),
    defaultValues: {
      name: "",
      categoryId: "",
      description: "",
      tagPids: [],
    },
  });

  const productQuery = useQuery({
    queryKey: [...productsQueryKey, pid],
    queryFn: () => getProduct(pid),
  });

  const categoriesQuery = useQuery({
    queryKey: [...categoriesQueryKey, { limit: 100 }],
    queryFn: () => listCategories({ limit: 100 }),
  });

  const attributesQuery = useQuery({
    queryKey: productAttributesQueryKey,
    queryFn: listProductAttributes,
  });

  const tagsQuery = useQuery({
    queryKey: productTagsQueryKey,
    queryFn: listProductTags,
  });

  const product = productQuery.data;
  const attributes = attributesQuery.data ?? [];
  const tags = tagsQuery.data ?? [];
  const tagOptions = useMemo(
    () => tags.map((tag) => ({ label: tag.name, value: tag.pid })),
    [tags],
  );

  const categoryOptions = useMemo(
    () =>
      (categoriesQuery.data?.data ?? []).map((category) => ({
        label: titleCase(category.name),
        value: String(category.id),
      })),
    [categoriesQuery.data?.data],
  );

  function refreshProduct() {
    void queryClient.invalidateQueries({ queryKey: productsQueryKey });
  }

  useEffect(() => {
    if (!product) return;

    form.reset({
      name: product.name,
      categoryId: String(product.category.id),
      description: product.description ?? "",
      tagPids: product.tags.map((tag) => tag.pid),
    });
    setInformation(informationEntries(product.information));

    setVariantDrafts(
      Object.fromEntries(
        product.variants.map((variant) => [
          variant.pid,
          {
            sku: variant.sku,
            price: variant.price,
            stockQuantity: String(variant.stockQuantity),
          },
        ]),
      ),
    );
  }, [form, product]);

  const updateBaseMutation = useMutation({
    mutationFn: (values: BaseValues) =>
      updateProduct(pid, {
        name: values.name.trim(),
        categoryId: Number(values.categoryId),
        description: values.description?.trim() ? values.description.trim() : null,
        information: productInformation(information),
        tagPids: values.tagPids,
      }),
    onSuccess: () => {
      toast.success("Product updated");
      refreshProduct();
    },
    onError: (error) => toast.error(error.message),
  });

  const addProductImageMutation = useMutation({
    mutationFn: async (file: File) => {
      const upload = await uploadProductImage(file);
      return addProductPicture(pid, {
        imageLink: upload.imageLink,
        mediaAssetPid: upload.assetPid,
        displayOrder: (product?.pictures.length ?? 0) + 1,
      });
    },
    onSuccess: () => {
      toast.success("Product image added");
      refreshProduct();
    },
    onError: (error) => toast.error(error.message),
  });

  const createVariantMutation = useMutation({
    mutationFn: () =>
      createProductVariant(pid, {
        sku: newVariant.sku.trim(),
        price: newVariant.price.trim(),
        stockQuantity: Number(newVariant.stockQuantity),
        isDefault: false,
        options:
          product?.options.map((option) => ({
            attributeId: option.attributeId,
            attributeValueId: Number(newVariant.optionValues[String(option.attributeId)]),
            displayOrder: option.displayOrder ?? undefined,
          })) ?? [],
      }),
    onSuccess: () => {
      toast.success("Variant created");
      setNewVariant({ sku: "", price: "", stockQuantity: "0", optionValues: {} });
      refreshProduct();
    },
    onError: (error) => toast.error(error.message),
  });

  function updateVariantDraft(pid: string, patch: Partial<VariantDraft>) {
    setVariantDrafts((current) => ({
      ...current,
      [pid]: {
        ...current[pid],
        ...patch,
      },
    }));
  }

  function saveVariant(variant: ProductVariantDetail) {
    const draft = variantDrafts[variant.pid];
    if (!draft) return;

    updateProductVariant(pid, variant.pid, {
      sku: draft.sku.trim(),
      price: draft.price.trim(),
      stockQuantity: Number(draft.stockQuantity),
    })
      .then(() => {
        toast.success("Variant updated");
        refreshProduct();
      })
      .catch((error: Error) => toast.error(error.message));
  }

  function deleteVariant(variant: ProductVariantDetail) {
    deleteProductVariant(pid, variant.pid)
      .then(() => {
        toast.success("Variant deleted");
        refreshProduct();
      })
      .catch((error: Error) => toast.error(error.message));
  }

  function setDefault(variant: ProductVariantDetail) {
    setDefaultProductVariant(pid, variant.pid)
      .then(() => {
        toast.success("Default variant updated");
        refreshProduct();
      })
      .catch((error: Error) => toast.error(error.message));
  }

  async function addVariantImage(variant: ProductVariantDetail, file: File) {
    const upload = await uploadProductImage(file);
    addVariantPicture(pid, variant.pid, {
      imageLink: upload.imageLink,
      mediaAssetPid: upload.assetPid,
      displayOrder: variant.pictures.length + 1,
    })
      .then(() => {
        toast.success("Variant image added");
        refreshProduct();
      })
      .catch((error: Error) => toast.error(error.message));
  }

  if (productQuery.isLoading) {
    return <Skeleton className="h-[40rem] w-full" />;
  }

  if (productQuery.isError) {
    return (
      <ErrorState
        title="Product could not be loaded"
        description={productQuery.error.message}
        onRetry={() => void productQuery.refetch()}
      />
    );
  }

  if (!product) {
    return (
      <EmptyState
        icon={<PackageIcon className="size-8" />}
        title="Product not found"
        description="The requested product no longer exists or is unavailable."
      />
    );
  }

  if (tagsQuery.isError) {
    return (
      <ErrorState
        title="Product tags could not be loaded"
        description={tagsQuery.error.message}
        onRetry={() => void tagsQuery.refetch()}
      />
    );
  }

  return (
    <form
      className="flex min-h-[calc(100dvh-8rem)] flex-col"
      onSubmit={form.handleSubmit((values) => updateBaseMutation.mutate(values))}
    >
      <CatalogueFormHeader
        title={`Edit ${product.name}`}
        description="Update base product details, manage media, and maintain variants."
      />

      <div className="mb-4">
        <Button
          variant="ghost"
          className="w-fit px-0"
          render={<Link to="/products/$pid" params={{ pid }} />}
        >
          <ArrowLeftIcon className="size-4" />
          Product details
        </Button>
      </div>

      <CatalogueFormSection
        title="Product details"
        description="These fields update the base catalogue record only."
        contentClassName="grid gap-5 lg:grid-cols-2"
        paddedTop={false}
      >
        <FormField control={form.control} name="name" label="Product Name" required />
        <FormField
          control={form.control}
          name="categoryId"
          type="select"
          label="Category"
          options={categoryOptions}
          placeholder={categoriesQuery.isLoading ? "Loading..." : "Select category"}
          disabled={categoriesQuery.isLoading}
          required
        />
        <div className="lg:col-span-2">
          <FormField
            control={form.control}
            name="description"
            type="textarea"
            label="Description"
            className="min-h-32 resize-y"
          />
        </div>
        <div className="lg:col-span-2">
          <ProductInformationFields entries={information} onChange={setInformation} />
        </div>
        <div className="lg:col-span-2">
          <FormField
            control={form.control}
            name="tagPids"
            type="combobox-multiple"
            label="Tags"
            description="Assign reusable labels for catalogue filtering and merchandising."
            placeholder={tagsQuery.isLoading ? "Loading tags..." : "Search or select tags"}
            options={tagOptions}
            disabled={tagsQuery.isLoading}
          />
        </div>
      </CatalogueFormSection>

      <CatalogueFormSection
        title="Product media"
        description="Add, remove, or reorder product-level images."
      >
        <div className="space-y-4">
          <ImageDropzone
            label="Drop or select product images"
            multiple
            onFiles={(files) => files.forEach((file) => addProductImageMutation.mutate(file))}
          />
          {product.pictures.length === 0 ? (
            <EmptyState title="No product media" description="Add product-level images." />
          ) : (
            <div className="space-y-3">
              {product.pictures.map((picture) => (
                <ProductPictureRow
                  key={picture.pid}
                  productPid={pid}
                  picture={picture}
                  onChanged={refreshProduct}
                />
              ))}
            </div>
          )}
        </div>
      </CatalogueFormSection>

      <CatalogueFormSection
        title="Options"
        description="Product option dimensions are read-only after creation."
      >
        <div className="grid gap-3 md:grid-cols-2">
          {product.options.map((option) => (
            <div key={option.pid} className="border bg-background p-4">
              <p className="font-semibold">{titleCase(option.attributeName)}</p>
              {option.attributeDescription && (
                <p className="mt-1 text-sm text-muted-foreground">{option.attributeDescription}</p>
              )}
              <p className="mt-1 text-sm text-muted-foreground">
                Display order {option.displayOrder ?? "N/A"}
              </p>
            </div>
          ))}
        </div>
      </CatalogueFormSection>

      <CatalogueFormSection
        title="Variants"
        description="SKU, price, and stock are editable. Option values are immutable."
      >
        <div className="space-y-5">
          {product.variants.map((variant) => {
            const draft = variantDrafts[variant.pid] ?? {
              sku: variant.sku,
              price: variant.price,
              stockQuantity: String(variant.stockQuantity),
            };

            return (
              <div key={variant.pid} className="space-y-4 border bg-background p-4">
                <div className="flex flex-col gap-3 md:flex-row md:items-start md:justify-between">
                  <div>
                    <div className="flex items-center gap-2">
                      <h3 className="font-semibold">{variant.sku}</h3>
                      {variant.isDefault && (
                        <span className="inline-flex items-center gap-1 border bg-muted px-2 py-1 text-xs">
                          <CheckCircleIcon className="size-3" />
                          Default
                        </span>
                      )}
                    </div>
                    <p className="mt-1 text-sm text-muted-foreground">
                      {variantOptionsLabel(variant)}
                    </p>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    {!variant.isDefault && (
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={() => setDefault(variant)}
                      >
                        Set Default
                      </Button>
                    )}
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      onClick={() => saveVariant(variant)}
                    >
                      Save Variant
                    </Button>
                    <Button
                      type="button"
                      variant="outline"
                      size="icon-sm"
                      disabled={variant.isDefault}
                      aria-label="Delete variant"
                      onClick={() => deleteVariant(variant)}
                    >
                      <TrashIcon className="size-4" />
                    </Button>
                  </div>
                </div>

                <div className="grid gap-4 lg:grid-cols-3">
                  <div className="space-y-2">
                    <Label>SKU</Label>
                    <Input
                      value={draft.sku}
                      onChange={(event) =>
                        updateVariantDraft(variant.pid, { sku: event.target.value })
                      }
                    />
                  </div>
                  <div className="space-y-2">
                    <Label>Price</Label>
                    <Input
                      value={draft.price}
                      inputMode="decimal"
                      onChange={(event) =>
                        updateVariantDraft(variant.pid, { price: event.target.value })
                      }
                    />
                  </div>
                  <div className="space-y-2">
                    <Label>Stock</Label>
                    <Input
                      value={draft.stockQuantity}
                      type="number"
                      min={0}
                      onChange={(event) =>
                        updateVariantDraft(variant.pid, { stockQuantity: event.target.value })
                      }
                    />
                  </div>
                </div>

                <div className="space-y-3">
                  <ImageDropzone
                    label="Drop or select variant image"
                    onFiles={(files) => {
                      const file = files[0];
                      if (file) addVariantImage(variant, file);
                    }}
                  />
                  {variant.pictures.map((picture) => (
                    <VariantPictureRow
                      key={picture.pid}
                      productPid={pid}
                      variantPid={variant.pid}
                      picture={picture}
                      onChanged={refreshProduct}
                    />
                  ))}
                </div>
              </div>
            );
          })}

          <div className="space-y-4 border bg-muted/20 p-4">
            <div>
              <h3 className="font-semibold">Add Variant</h3>
              <p className="text-sm text-muted-foreground">
                New variants must provide one value for every product option.
              </p>
            </div>

            <div className="grid gap-4 lg:grid-cols-3">
              <div className="space-y-2">
                <Label>SKU</Label>
                <Input
                  value={newVariant.sku}
                  placeholder="SKU-NEW"
                  onChange={(event) =>
                    setNewVariant((current) => ({ ...current, sku: event.target.value }))
                  }
                />
              </div>
              <div className="space-y-2">
                <Label>Price</Label>
                <Input
                  value={newVariant.price}
                  inputMode="decimal"
                  placeholder="29.99"
                  onChange={(event) =>
                    setNewVariant((current) => ({ ...current, price: event.target.value }))
                  }
                />
              </div>
              <div className="space-y-2">
                <Label>Stock</Label>
                <Input
                  value={newVariant.stockQuantity}
                  type="number"
                  min={0}
                  onChange={(event) =>
                    setNewVariant((current) => ({ ...current, stockQuantity: event.target.value }))
                  }
                />
              </div>
            </div>

            <div className="grid gap-4 lg:grid-cols-2">
              {product.options.map((option) => (
                <div key={option.pid} className="space-y-2">
                  <Label>{titleCase(option.attributeName)}</Label>
                  {option.attributeDescription && (
                    <p className="text-xs text-muted-foreground">{option.attributeDescription}</p>
                  )}
                  <Select
                    value={newVariant.optionValues[String(option.attributeId)] ?? ""}
                    onValueChange={(value) =>
                      setNewVariant((current) => ({
                        ...current,
                        optionValues: {
                          ...current.optionValues,
                          [String(option.attributeId)]: value ?? "",
                        },
                      }))
                    }
                  >
                    <SelectTrigger className="w-full">
                      {selectLabel(
                        optionValues(attributes, option.attributeId).find(
                          (value) =>
                            String(value.id) ===
                            newVariant.optionValues[String(option.attributeId)],
                        )?.value ?? "",
                        "Select value",
                      )}
                    </SelectTrigger>
                    <SelectContent>
                      {optionValues(attributes, option.attributeId).map((value) => (
                        <SelectItem key={value.id} value={String(value.id)}>
                          {value.value}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              ))}
            </div>

            <div className="flex justify-end">
              <Button
                type="button"
                variant="outline"
                onClick={() => createVariantMutation.mutate()}
                disabled={createVariantMutation.isPending}
              >
                <PlusIcon className="size-4" />
                Add Variant
              </Button>
            </div>
          </div>
        </div>
      </CatalogueFormSection>

      <CatalogueFormActions
        draftLabel="Reset"
        submitLabel="Save Product"
        pendingSubmitLabel="Saving..."
        isPending={updateBaseMutation.isPending}
        onDraft={() => {
          if (product) {
            form.reset({
              name: product.name,
              categoryId: String(product.category.id),
              description: product.description ?? "",
            });
          }
        }}
      />
    </form>
  );
}
