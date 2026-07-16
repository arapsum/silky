import { zodResolver } from "@hookform/resolvers/zod";
import { ArrowRightIcon } from "@phosphor-icons/react";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { z } from "zod";
import { FormField } from "@/components/forms/FormField";
import { ApiError, sessionApi } from "@/lib/api/browser";

type AuthMode = "login" | "register" | "forgot" | "reset";

interface AuthFormProps {
  mode: AuthMode;
  next?: string;
  token?: string;
}

const password = z
  .string()
  .min(8, "Use at least 8 characters.")
  .regex(/^\S+$/, "Password cannot contain spaces.");
const schemas: Record<AuthMode, z.ZodType<Record<string, string>, Record<string, string>>> = {
  login: z.object({ email: z.email("Enter a valid email address."), password }),
  register: z
    .object({
      name: z
        .string()
        .min(6, "Enter at least 6 characters.")
        .max(32, "Keep the name under 32 characters."),
      email: z.email("Enter a valid email address."),
      password,
      confirmPassword: z.string(),
    })
    .refine((value) => value.password === value.confirmPassword, {
      path: ["confirmPassword"],
      message: "Passwords do not match.",
    }),
  forgot: z.object({ email: z.email("Enter a valid email address.") }),
  reset: z
    .object({ password, confirmPassword: z.string() })
    .refine((value) => value.password === value.confirmPassword, {
      path: ["confirmPassword"],
      message: "Passwords do not match.",
    }),
};

const copy = {
  login: {
    eyebrow: "Welcome back",
    title: "Sign in to continue.",
    submit: "Sign in",
    alternate: "New to Silk?",
    alternateLink: "/auth/register",
    alternateAction: "Create an account",
  },
  register: {
    eyebrow: "Your Silk account",
    title: "Make checkout easier.",
    submit: "Create account",
    alternate: "Already have an account?",
    alternateLink: "/auth/login",
    alternateAction: "Sign in",
  },
  forgot: {
    eyebrow: "Password help",
    title: "Find your way back in.",
    submit: "Send reset link",
    alternate: "Remembered it?",
    alternateLink: "/auth/login",
    alternateAction: "Sign in",
  },
  reset: {
    eyebrow: "Choose a password",
    title: "Set a new password.",
    submit: "Save password",
    alternate: "Return to",
    alternateLink: "/auth/login",
    alternateAction: "sign in",
  },
} satisfies Record<AuthMode, Record<string, string>>;

const formActions: Record<AuthMode, string> = {
  login: "/auth/login",
  register: "/auth/register",
  forgot: "/auth/forgot-password",
  reset: "/auth/reset-password",
};

export function AuthForm({ mode, next = "/account", token = "" }: AuthFormProps) {
  const [status, setStatus] = useState<string | null>(null);
  const [requestError, setRequestError] = useState<string | null>(null);
  const {
    register,
    handleSubmit,
    formState: { errors, isSubmitting },
  } = useForm<Record<string, string>>({
    resolver: zodResolver(schemas[mode]),
  });
  const content = copy[mode];

  async function submit(values: Record<string, string>) {
    setRequestError(null);
    setStatus(null);
    try {
      if (mode === "login") {
        await sessionApi.login({ email: values.email, password: values.password });
        window.location.assign(next.startsWith("/") ? next : "/account");
      } else if (mode === "register") {
        await sessionApi.register({
          name: values.name,
          email: values.email,
          password: values.password,
          confirmPassword: values.confirmPassword,
        });
        window.location.assign("/auth/login?registered=true");
      } else if (mode === "forgot") {
        await sessionApi.forgotPassword(values.email);
        setStatus("If that email belongs to an account, a reset link is on its way.");
      } else {
        await sessionApi.resetPassword({
          token,
          password: values.password,
          confirmPassword: values.confirmPassword,
        });
        window.location.assign("/auth/login?reset=true");
      }
    } catch (error) {
      setRequestError(
        error instanceof ApiError ? error.message : "Silk could not complete that request.",
      );
    }
  }

  return (
    <section className="auth-card">
      <p className="eyebrow">{content.eyebrow}</p>
      <h1>{content.title}</h1>
      <form action={formActions[mode]} method="post" noValidate onSubmit={handleSubmit(submit)}>
        {mode === "register" && (
          <FormField
            autoComplete="name"
            error={errors.name?.message}
            label="Full name"
            {...register("name")}
          />
        )}
        {mode !== "reset" && (
          <FormField
            autoComplete="email"
            error={errors.email?.message}
            label="Email address"
            type="email"
            {...register("email")}
          />
        )}
        {mode !== "forgot" && (
          <FormField
            autoComplete={mode === "login" ? "current-password" : "new-password"}
            error={errors.password?.message}
            label="Password"
            type="password"
            {...register("password")}
          />
        )}
        {(mode === "register" || mode === "reset") && (
          <FormField
            autoComplete="new-password"
            error={errors.confirmPassword?.message}
            label="Confirm password"
            type="password"
            {...register("confirmPassword")}
          />
        )}
        {mode === "login" && (
          <a className="auth-card__forgot" href="/auth/forgot-password">
            Forgot your password?
          </a>
        )}
        {requestError && (
          <p className="form-error" role="alert">
            {requestError}
          </p>
        )}
        {status && (
          <p className="form-success" role="status">
            {status}
          </p>
        )}
        <button
          className="button-primary auth-card__submit"
          disabled={isSubmitting || (mode === "reset" && !token)}
          type="submit"
        >
          {isSubmitting ? "Please wait" : content.submit}
          {!isSubmitting && <ArrowRightIcon aria-hidden size={17} />}
        </button>
      </form>
      <p className="auth-card__alternate">
        {content.alternate} <a href={content.alternateLink}>{content.alternateAction}</a>
      </p>
    </section>
  );
}
