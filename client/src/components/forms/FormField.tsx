import type { InputHTMLAttributes, ReactNode } from "react";

interface FormFieldProps extends InputHTMLAttributes<HTMLInputElement> {
  label: string;
  error?: string;
  hint?: ReactNode;
}

export function FormField({ label, error, hint, id, ...input }: FormFieldProps) {
  const fieldId = id ?? input.name;
  const messageId = `${fieldId}-message`;
  const message = error ?? hint;
  return (
    <label className={`form-field ${error ? "is-invalid" : ""}`} htmlFor={fieldId}>
      <span>{label}</span>
      <input
        {...input}
        aria-describedby={message ? messageId : undefined}
        aria-invalid={Boolean(error)}
        id={fieldId}
      />
      <small aria-hidden={!message} className={error ? "form-error" : ""} id={messageId}>
        {message ?? "\u00a0"}
      </small>
    </label>
  );
}
