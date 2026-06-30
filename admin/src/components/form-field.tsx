import { Field, FieldDescription, FieldError, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
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
import { EyeIcon, EyeSlashIcon } from "@phosphor-icons/react";

type Props<TField extends FieldValues> = Omit<
  React.ComponentProps<"input">,
  "defaultValue" | "name" | "onBlur" | "onChange" | "ref" | "type" | "value"
> & {
  control: Control<TField, any>;
  name: Path<TField>;
  label?: string;
  type?: HTMLInputElement["type"];
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
  return (
    <Controller
      control={control}
      name={name}
      render={({ field, fieldState }) => (
        <Field data-invalid={fieldState.invalid}>
          {label && (
            <FieldLabel>
              {label}
              {required && (
                <span className="ml-0.5 text-destructive" aria-hidden="true">
                  *
                </span>
              )}
            </FieldLabel>
          )}

          <RenderInput field={field} fieldState={fieldState} input={{ ...rest, required }} />

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
  input: React.ComponentProps<"input">;
};

function RenderInput<TField extends FieldValues>({
  field,
  fieldState,
  input,
}: RenderInputProps<TField>) {
  const [visible, setVisible] = useState(false);
  const { type = "text", placeholder } = input;

  switch (type) {
    case "password": {
      return (
        <div className="relative">
          <Input
            {...field}
            {...input}
            id={field.name}
            type={visible ? "text" : "password"}
            placeholder={placeholder}
            aria-label={visible ? "Hide password" : "Show password"}
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

    default: {
      return (
        <Input
          {...field}
          {...input}
          id={field.name}
          type={type}
          aria-invalid={fieldState.invalid}
          placeholder={placeholder}
        />
      );
    }
  }
}
