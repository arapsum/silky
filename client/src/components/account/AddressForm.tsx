import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { z } from "zod";
import { FormField } from "@/components/forms/FormField";
import { addressApi, ApiError } from "@/lib/api/browser";
import type { Address, AddressInput } from "@/lib/api/types";
import { toast } from "@/lib/toast";

const optionalText = z.string().max(255).optional();
const schema = z.object({
  addressType: z.enum(["shipping", "billing", "other"]),
  label: z.string().max(120).optional(),
  recipientName: z.string().min(1, "Enter the recipient name.").max(255),
  company: optionalText,
  lineOne: z.string().min(1, "Enter the street address.").max(255),
  lineTwo: optionalText,
  city: z.string().min(1, "Enter the city.").max(120),
  region: z.string().max(120).optional(),
  postalCode: z.string().max(32).optional(),
  countryCode: z.string().length(2, "Use a two-letter country code."),
  email: z.union([z.literal(""), z.email("Enter a valid email address.")]).optional(),
  phone: z.string().max(32).optional(),
  isDefault: z.boolean(),
});

type Values = z.infer<typeof schema>;

interface AddressFormProps {
  address?: Address;
  onSaved: (address: Address) => void;
  onCancel?: () => void;
  compact?: boolean;
}

function valuesFor(address?: Address): Values {
  return {
    addressType: address?.addressType ?? "shipping",
    label: address?.label ?? "",
    recipientName: address?.recipientName ?? "",
    company: address?.company ?? "",
    lineOne: address?.lineOne ?? "",
    lineTwo: address?.lineTwo ?? "",
    city: address?.city ?? "",
    region: address?.region ?? "",
    postalCode: address?.postalCode ?? "",
    countryCode: address?.countryCode ?? "KE",
    email: address?.email ?? "",
    phone: address?.phone ?? "",
    isDefault: address?.isDefault ?? false,
  };
}

function toInput(values: Values): AddressInput {
  const optional = (value?: string) => value?.trim() || null;
  return {
    addressType: values.addressType,
    label: optional(values.label),
    recipientName: values.recipientName.trim(),
    company: optional(values.company),
    lineOne: values.lineOne.trim(),
    lineTwo: optional(values.lineTwo),
    city: values.city.trim(),
    region: optional(values.region),
    postalCode: optional(values.postalCode),
    countryCode: values.countryCode.toUpperCase(),
    email: optional(values.email),
    phone: optional(values.phone),
    isDefault: values.isDefault,
  };
}

export function AddressForm({ address, onSaved, onCancel, compact = false }: AddressFormProps) {
  const { register, handleSubmit, formState: { errors, isSubmitting } } = useForm<Values>({
    resolver: zodResolver(schema),
    defaultValues: valuesFor(address),
    mode: "onChange",
    reValidateMode: "onChange",
  });

  async function submit(values: Values) {
    try {
      const saved = address
        ? await addressApi.update(address.pid, toInput(values))
        : await addressApi.create(toInput(values));
      onSaved(saved);
      toast.success(address ? "Address updated" : "Address saved", "Your delivery details are ready to use.");
    } catch (error) {
      toast.error("Address not saved", error instanceof ApiError ? error.message : "The address could not be saved.");
    }
  }

  return (
    <form className={`address-form ${compact ? "is-compact" : ""}`} noValidate onSubmit={handleSubmit(submit)}>
      <label className="form-field">
        <span>Address type</span>
        <select {...register("addressType")}>
          <option value="shipping">Shipping</option>
          <option value="billing">Billing</option>
          <option value="other">Other</option>
        </select>
      </label>
      <FormField error={errors.label?.message} label="Label" placeholder="Home, office" {...register("label")} />
      <FormField error={errors.recipientName?.message} label="Recipient name" {...register("recipientName")} />
      <FormField error={errors.company?.message} label="Company (optional)" {...register("company")} />
      <FormField error={errors.lineOne?.message} label="Street address" {...register("lineOne")} />
      <FormField error={errors.lineTwo?.message} label="Apartment or suite (optional)" {...register("lineTwo")} />
      <FormField error={errors.city?.message} label="City" {...register("city")} />
      <FormField error={errors.region?.message} label="Region (optional)" {...register("region")} />
      <FormField error={errors.postalCode?.message} label="Postal code (optional)" {...register("postalCode")} />
      <FormField error={errors.countryCode?.message} label="Country code" maxLength={2} {...register("countryCode")} />
      <FormField error={errors.email?.message} label="Delivery email (optional)" type="email" {...register("email")} />
      <FormField error={errors.phone?.message} label="Delivery phone (optional)" type="tel" {...register("phone")} />
      <label className="check-field">
        <input type="checkbox" {...register("isDefault")} />
        <span>Use as my default {address?.addressType ?? "address"}</span>
      </label>
      <div className="address-form__actions">
        {onCancel && <button className="button-secondary" onClick={onCancel} type="button">Cancel</button>}
        <button className="button-primary" disabled={isSubmitting} type="submit">
          {isSubmitting ? "Saving" : address ? "Save changes" : "Save address"}
        </button>
      </div>
    </form>
  );
}
