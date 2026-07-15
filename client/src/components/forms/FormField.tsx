import type { InputHTMLAttributes, ReactNode } from "react";

interface FormFieldProps extends InputHTMLAttributes<HTMLInputElement> {
  label: string;
  error?: string;
  hint?: ReactNode;
}

export function FormField({ label, error, hint, id, ...input }: FormFieldProps) {
  const fieldId = id ?? input.name;
  const messageId = `${fieldId}-message`;
  return (
    <label className="form-field" htmlFor={fieldId}>
      <span>{label}</span>
      <input
        {...input}
        aria-describedby={error || hint ? messageId : undefined}
        aria-invalid={Boolean(error)}
        id={fieldId}
      />
      {(error || hint) && <small className={error ? "form-error" : ""} id={messageId}>{error ?? hint}</small>}
    </label>
  );
}
