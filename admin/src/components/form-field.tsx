import { Field, FieldDescription, FieldError, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Checkbox } from "@/components/ui/checkbox";
import { Textarea } from "@/components/ui/textarea";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { cn } from "@/lib/utils";
import type React from "react";
import { useState } from "react";
import {
  type FieldValues,
  type Path,
  type Control,
  Controller,
  type ControllerRenderProps,
  type ControllerFieldState,
} from "react-hook-form";
import { EyeIcon, EyeSlashIcon, EnvelopeSimpleIcon } from "@phosphor-icons/react";

type SelectOption = { label: string; value: string };

type BaseInputProps = Omit<
  React.ComponentProps<"input">,
  "defaultValue" | "name" | "onBlur" | "onChange" | "ref" | "type" | "value"
>;

type TextareaInputProps = Omit<
  React.ComponentProps<"textarea">,
  "defaultValue" | "name" | "onBlur" | "onChange" | "ref" | "value"
> & {
  type: "textarea";
};

type FieldInputProps = (BaseInputProps | TextareaInputProps) & {
  type?: HTMLInputElement["type"] | "select" | "textarea";
  /** Required when type="select" */
  options?: SelectOption[];
};

type Props<TField extends FieldValues> = FieldInputProps & {
  control: Control<TField>;
  name: Path<TField>;
  label?: string;
  description?: string;
};

export default function FormField<TField extends FieldValues>({
  control,
  name,
  label,
  description,
  required,
  ...rest
}: Props<TField>) {
  const isCheckbox = rest.type === "checkbox";

  return (
    <Controller
      control={control}
      name={name}
      render={({ field, fieldState }) => (
        <Field data-invalid={fieldState.invalid}>
          {label && !isCheckbox && (
            <FieldLabel htmlFor={field.name}>
              {label}
              {required && (
                <span className="ml-0.5 text-destructive" aria-hidden="true">
                  *
                </span>
              )}
            </FieldLabel>
          )}

          {isCheckbox ? (
            <div className="flex items-center gap-2">
              <RenderInput field={field} fieldState={fieldState} input={{ ...rest, required }} />
              {label && (
                <FieldLabel htmlFor={field.name} className="font-normal">
                  {label}
                  {required && (
                    <span className="ml-0.5 text-destructive" aria-hidden="true">
                      *
                    </span>
                  )}
                </FieldLabel>
              )}
            </div>
          ) : (
            <RenderInput field={field} fieldState={fieldState} input={{ ...rest, required }} />
          )}

          {description && <FieldDescription>{description}</FieldDescription>}

          {fieldState.invalid && (
            <FieldError className="mt-1 text-xs" errors={[fieldState.error]} />
          )}
        </Field>
      )}
    />
  );
}

type RenderInputProps<TField extends FieldValues> = {
  field: ControllerRenderProps<TField, Path<TField>>;
  fieldState: ControllerFieldState;
  input: FieldInputProps;
};

function RenderInput<TField extends FieldValues>({
  field,
  fieldState,
  input,
}: RenderInputProps<TField>) {
  const [visible, setVisible] = useState(false);
  const type = input.type ?? "text";

  switch (type) {
    case "textarea": {
      const {
        type: _type,
        options: _options,
        placeholder,
        className,
        disabled,
        required,
        ...rest
      } = input as TextareaInputProps & { options?: SelectOption[] };

      return (
        <Textarea
          {...field}
          {...rest}
          id={field.name}
          placeholder={placeholder}
          className={className}
          disabled={disabled}
          required={required}
          aria-invalid={fieldState.invalid}
        />
      );
    }

    case "password": {
      const { placeholder, className, disabled, required, ...rest } = input as BaseInputProps;

      return (
        <div className="relative">
          <Input
            {...field}
            {...rest}
            id={field.name}
            type={visible ? "text" : "password"}
            placeholder={placeholder}
            className={cn("pr-10", className)}
            disabled={disabled}
            required={required}
            aria-invalid={fieldState.invalid}
          />
          <button
            type="button"
            onClick={() => setVisible((v) => !v)}
            className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
            aria-label={visible ? "Hide password" : "Show password"}
          >
            {visible ? (
              <EyeSlashIcon className="size-4" aria-hidden />
            ) : (
              <EyeIcon className="size-4" aria-hidden />
            )}
          </button>
        </div>
      );
    }

    case "email": {
      const { placeholder, className, disabled, required, ...rest } = input as BaseInputProps;

      return (
        <div className="relative">
          <Input
            {...field}
            {...rest}
            id={field.name}
            type="email"
            placeholder={placeholder}
            className={cn("pr-10", className)}
            disabled={disabled}
            required={required}
            aria-invalid={fieldState.invalid}
          />
          <EnvelopeSimpleIcon
            className="pointer-events-none absolute right-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground"
            aria-hidden
          />
        </div>
      );
    }

    case "select": {
      const { placeholder, options, className, disabled, required } = input as BaseInputProps & {
        options?: SelectOption[];
      };

      if (process.env.NODE_ENV !== "production" && !options?.length) {
        console.warn(`FormField: "options" is required for select field "${field.name}"`);
      }

      return (
        <Select
          value={field.value ?? ""}
          onValueChange={field.onChange}
          onOpenChange={(open) => {
            if (!open) field.onBlur();
          }}
          disabled={disabled}
          required={required}
          name={field.name}
        >
          <SelectTrigger
            id={field.name}
            aria-invalid={fieldState.invalid}
            className={cn("w-full", className)}
          >
            <SelectValue placeholder={placeholder} />
          </SelectTrigger>
          <SelectContent>
            {options?.map((option) => (
              <SelectItem key={option.value} value={option.value}>
                {option.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      );
    }

    case "checkbox": {
      const { className, disabled, required } = input as BaseInputProps;

      return (
        <Checkbox
          id={field.name}
          name={field.name}
          ref={field.ref}
          checked={!!field.value}
          onCheckedChange={field.onChange}
          onBlur={field.onBlur}
          disabled={disabled}
          required={required}
          className={className}
          aria-invalid={fieldState.invalid}
        />
      );
    }

    default: {
      const {
        type = "text",
        placeholder,
        className,
        disabled,
        required,
        ...rest
      } = input as BaseInputProps & { type?: HTMLInputElement["type"] };

      return (
        <Input
          {...field}
          {...rest}
          id={field.name}
          type={type}
          className={className}
          disabled={disabled}
          required={required}
          aria-invalid={fieldState.invalid}
          placeholder={placeholder}
        />
      );
    }
  }
}
