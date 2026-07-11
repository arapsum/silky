import { Field, FieldDescription, FieldError, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Checkbox } from "@/components/ui/checkbox";
import { Textarea } from "@/components/ui/textarea";
import { Select, SelectContent, SelectItem, SelectTrigger } from "@/components/ui/select";
import {
  Combobox,
  ComboboxChips,
  ComboboxChip,
  ComboboxChipsInput,
  ComboboxContent,
  ComboboxEmpty,
  ComboboxItem,
  ComboboxList,
  ComboboxValue,
  useComboboxAnchor,
} from "@/components/ui/combobox";
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

type ComboboxMultipleInputProps = {
  type: "combobox-multiple";
  options: SelectOption[];
  onCreateOption?: (label: string) => Promise<SelectOption>;
  placeholder?: string;
  className?: string;
  disabled?: boolean;
  required?: boolean;
};

type FieldInputProps = (BaseInputProps | TextareaInputProps | ComboboxMultipleInputProps) & {
  type?: HTMLInputElement["type"] | "select" | "textarea" | "combobox-multiple";
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
  const [comboboxInput, setComboboxInput] = useState("");
  const [isCreatingOption, setIsCreatingOption] = useState(false);
  const comboboxAnchor = useComboboxAnchor();
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
      const selectedOption = options?.find((option) => option.value === field.value);
      const displayValue = selectedOption?.label ?? "";

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
            <span className={cn("flex flex-1 text-left", !displayValue && "text-muted-foreground")}>
              {displayValue || placeholder}
            </span>
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

    case "combobox-multiple": {
      const { options, onCreateOption, placeholder, className, disabled, required } =
        input as ComboboxMultipleInputProps;
      const values = Array.isArray(field.value) ? field.value : [];
      const newOptionLabel = comboboxInput.trim();
      const canCreateOption =
        !!onCreateOption &&
        !!newOptionLabel &&
        !options.some((option) => option.label.toLowerCase() === newOptionLabel.toLowerCase());

      async function createOption() {
        if (!onCreateOption || !canCreateOption || isCreatingOption) return;

        setIsCreatingOption(true);
        try {
          const option = await onCreateOption(newOptionLabel);
          field.onChange([...values, option.value]);
          field.onBlur();
          setComboboxInput("");
        } catch {
          // The caller owns error reporting so failed creation does not submit a stale value.
        } finally {
          setIsCreatingOption(false);
        }
      }

      return (
        <Combobox
          items={options}
          multiple
          value={values}
          onValueChange={(value) => {
            field.onChange(value);
            field.onBlur();
          }}
          onInputValueChange={setComboboxInput}
          disabled={disabled || isCreatingOption}
        >
          <ComboboxChips ref={comboboxAnchor} className={cn("w-full", className)}>
            <ComboboxValue>
              {(selectedValues: string[]) => (
                <>
                  {selectedValues.map((value) => {
                    const option = options.find((entry) => entry.value === value);

                    return <ComboboxChip key={value}>{option?.label ?? value}</ComboboxChip>;
                  })}
                  <ComboboxChipsInput
                    id={field.name}
                    ref={field.ref}
                    placeholder={selectedValues.length ? "" : placeholder}
                    onBlur={field.onBlur}
                    disabled={disabled || isCreatingOption}
                    required={required}
                    aria-invalid={fieldState.invalid}
                    onKeyDown={(event) => {
                      if (event.key === "Enter" && canCreateOption) {
                        event.preventDefault();
                        void createOption();
                      }
                    }}
                  />
                </>
              )}
            </ComboboxValue>
          </ComboboxChips>
          <ComboboxContent anchor={comboboxAnchor}>
            <ComboboxEmpty>
              {canCreateOption ? (
                <button
                  type="button"
                  className="w-full px-2 py-1.5 text-left text-sm hover:bg-accent"
                  disabled={isCreatingOption}
                  onClick={() => void createOption()}
                >
                  {isCreatingOption ? "Creating tag..." : `Create “${newOptionLabel}”`}
                </button>
              ) : (
                "No matching options."
              )}
            </ComboboxEmpty>
            <ComboboxList>
              {(option: SelectOption) => (
                <ComboboxItem key={option.value} value={option.value}>
                  {option.label}
                </ComboboxItem>
              )}
            </ComboboxList>
          </ComboboxContent>
        </Combobox>
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
