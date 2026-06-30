import { zodResolver } from "@hookform/resolvers/zod";
import { Link } from "@tanstack/react-router";
import { useForm } from "react-hook-form";
import { z } from "zod";

import FormField from "#/components/form-field.tsx";
import { Button } from "#/components/ui/button.tsx";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "#/components/ui/card.tsx";
import { cn } from "#/lib/utils.ts";

const loginSchema = z.object({
  email: z.email("Enter a valid email address"),
  password: z.string().min(8, "Password must be at least 8 characters"),
});

type LoginValues = z.infer<typeof loginSchema>;

export function LoginForm({ className, ...props }: React.ComponentProps<"form">) {
  const form = useForm<LoginValues>({
    resolver: zodResolver(loginSchema),
    defaultValues: {
      email: "",
      password: "",
    },
  });

  async function onSubmit(values: LoginValues) {
    await Promise.resolve(values);
  }

  return (
    <form
      className={cn("w-full", className)}
      onSubmit={form.handleSubmit(onSubmit)}
      noValidate
      {...props}
    >
      <Card>
        <CardHeader className="items-center text-center">
          <CardTitle className="text-2xl font-semibold">Sign in to Silk</CardTitle>
          <CardDescription className="text-balance">
            Enter your email and password to access the admin dashboard.
          </CardDescription>
        </CardHeader>

        <CardContent className="grid gap-5">
          <FormField
            control={form.control}
            name="email"
            label="Email"
            type="email"
            placeholder="name@example.com"
            autoComplete="email"
            required
          />

          <div className="grid gap-2">
            <div className="flex items-center justify-between gap-4">
              <div className="text-sm font-medium">
                Password
                <span className="ml-2.5 text-destructive" aria-hidden="true">
                  *
                </span>
              </div>
              <Link
                to="/sign-in"
                className="text-xs font-medium text-muted-foreground underline-offset-4 hover:text-foreground hover:underline"
              >
                Forgot password?
              </Link>
            </div>
            <FormField
              control={form.control}
              name="password"
              type="password"
              placeholder="Enter your password"
              autoComplete="current-password"
              required
            />
          </div>

          <Button type="submit" className="w-full" disabled={form.formState.isSubmitting}>
            {form.formState.isSubmitting ? "Signing in..." : "Sign in"}
          </Button>
        </CardContent>
      </Card>
    </form>
  );
}
