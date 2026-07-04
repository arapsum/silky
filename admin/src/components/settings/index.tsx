"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { CheckIcon, XIcon } from "@phosphor-icons/react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";
import { useForm } from "react-hook-form";
import { toast } from "sonner";
import { z } from "zod";

import {
  changePassword,
  getCurrentUser,
  updateCurrentUser,
  type CurrentUser,
} from "#/api/account.ts";
import FormField from "#/components/form-field";
import { Button } from "#/components/ui/button";
import { Separator } from "#/components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "#/components/ui/tabs";
import { cn } from "#/lib/utils";

const currentUserQueryKey = ["current-user"] as const;

const PASSWORD_RULES: { id: string; label: string; test: (v: string) => boolean }[] = [
  { id: "length", label: "At least 12 characters", test: (v) => v.length >= 12 },
  { id: "lowercase", label: "At least 1 lowercase letter", test: (v) => /[a-z]/.test(v) },
  { id: "uppercase", label: "At least 1 uppercase letter", test: (v) => /[A-Z]/.test(v) },
  { id: "number", label: "At least 1 number", test: (v) => /[0-9]/.test(v) },
  { id: "special", label: "At least 1 special character", test: (v) => /[^A-Za-z0-9]/.test(v) },
];

const tabTriggerClass =
  "relative rounded-none border-b-2 border-transparent bg-transparent px-0.5 pb-3 pt-0 font-medium text-muted-foreground shadow-none transition-colors hover:text-foreground data-[state=active]:border-foreground data-[state=active]:bg-transparent data-[state=active]:text-foreground data-[state=active]:shadow-none";

const personalInfoSchema = z.object({
  name: z
    .string()
    .trim()
    .min(6, "Name requires 6 letters")
    .max(32, "Name must be under 32 letters")
    .regex(/^[a-zA-Z0-9_ ]+$/, "Only letters, numbers and underscores can be used."),
});

type PersonalInfoValues = z.infer<typeof personalInfoSchema>;

const emailPasswordSchema = z
  .object({
    email: z.string().min(1, "Email is required").email("Enter a valid email address"),
    currentPassword: z.string(),
    newPassword: z.string(),
  })
  .superRefine((values, ctx) => {
    const currentPassword = values.currentPassword.trim();
    const newPassword = values.newPassword.trim();
    const changingPassword = currentPassword.length > 0 || newPassword.length > 0;

    if (!changingPassword) return;

    if (!currentPassword) {
      ctx.addIssue({
        code: "custom",
        path: ["currentPassword"],
        message: "Current password is required",
      });
    }

    if (!newPassword) {
      ctx.addIssue({
        code: "custom",
        path: ["newPassword"],
        message: "New password is required",
      });
      return;
    }

    if (!PASSWORD_RULES.every((rule) => rule.test(newPassword))) {
      ctx.addIssue({
        code: "custom",
        path: ["newPassword"],
        message: "Password does not meet all requirements",
      });
    }
  });

type EmailPasswordValues = z.infer<typeof emailPasswordSchema>;

function SectionHeader({ title, description }: { title: string; description: string }) {
  return (
    <div className="lg:col-span-1">
      <h2 className="text-base font-semibold">{title}</h2>
      <p className="mt-1 text-sm text-muted-foreground">{description}</p>
    </div>
  );
}

function PersonalInformationSection({ user }: { user?: CurrentUser }) {
  const queryClient = useQueryClient();
  const form = useForm<PersonalInfoValues>({
    resolver: zodResolver(personalInfoSchema),
    defaultValues: { name: "" },
  });

  useEffect(() => {
    if (user) {
      form.reset({ name: user.name });
    }
  }, [form, user]);

  const updateProfileMutation = useMutation({
    mutationFn: async (values: PersonalInfoValues) => {
      if (!user) throw new Error("Unable to load account details");

      return updateCurrentUser({
        name: values.name,
        email: user.email,
      });
    },
    onSuccess: (updatedUser) => {
      queryClient.setQueryData(currentUserQueryKey, updatedUser);
      form.reset({ name: updatedUser.name });
      toast.success("Personal information updated", {
        id: "settings-profile-success",
      });
    },
    onError: (error) => {
      toast.error(error.message, {
        id: "settings-profile-error",
      });
    },
  });

  async function onSubmit(values: PersonalInfoValues) {
    await updateProfileMutation.mutateAsync(values);
  }

  return (
    <form onSubmit={form.handleSubmit(onSubmit)} className="grid grid-cols-1 gap-8 lg:grid-cols-3">
      <SectionHeader
        title="Personal Information"
        description="Manage your account profile details."
      />

      <div className="space-y-6 lg:col-span-2">
        <FormField
          control={form.control}
          name="name"
          label="Name"
          placeholder="Silk Admin"
          autoComplete="name"
          required
        />

        <div className="flex justify-end">
          <Button
            type="submit"
            disabled={!user || updateProfileMutation.isPending}
            className="bg-blue-600 hover:bg-blue-700"
          >
            {updateProfileMutation.isPending ? "Saving..." : "Save Changes"}
          </Button>
        </div>
      </div>
    </form>
  );
}

function EmailPasswordSection({ user }: { user?: CurrentUser }) {
  const queryClient = useQueryClient();
  const form = useForm<EmailPasswordValues>({
    resolver: zodResolver(emailPasswordSchema),
    defaultValues: { email: "", currentPassword: "", newPassword: "" },
  });

  useEffect(() => {
    if (user) {
      form.reset({
        email: user.email,
        currentPassword: "",
        newPassword: "",
      });
    }
  }, [form, user]);

  const newPassword = form.watch("newPassword");
  const satisfiedCount = PASSWORD_RULES.filter((rule) => rule.test(newPassword)).length;
  const strengthColor =
    satisfiedCount === 0
      ? "bg-muted"
      : satisfiedCount <= 2
        ? "bg-red-500"
        : satisfiedCount <= 3
          ? "bg-orange-500"
          : satisfiedCount === 4
            ? "bg-yellow-500"
            : "bg-green-500";
  const filledSegments = newPassword
    ? Math.max(1, Math.ceil((satisfiedCount / PASSWORD_RULES.length) * 4))
    : 0;

  const updateSecurityMutation = useMutation({
    mutationFn: async (values: EmailPasswordValues) => {
      if (!user) throw new Error("Unable to load account details");

      const email = values.email.trim();
      const currentPassword = values.currentPassword.trim();
      const password = values.newPassword.trim();
      const emailChanged = email !== user.email;
      const passwordChanged = currentPassword.length > 0 || password.length > 0;
      let updatedUser = user;

      if (passwordChanged) {
        await changePassword({
          currentPassword,
          password,
          confirmPassword: password,
        });
      }

      if (emailChanged) {
        updatedUser = await updateCurrentUser({
          name: user.name,
          email,
        });
      }

      return { updatedUser, emailChanged, passwordChanged };
    },
    onSuccess: ({ updatedUser, emailChanged, passwordChanged }) => {
      queryClient.setQueryData(currentUserQueryKey, updatedUser);
      form.reset({
        email: updatedUser.email,
        currentPassword: "",
        newPassword: "",
      });

      if (emailChanged && passwordChanged) {
        toast.success("Email and password updated", {
          id: "settings-security-success",
        });
      } else if (emailChanged) {
        toast.success("Email updated", {
          id: "settings-security-success",
        });
      } else if (passwordChanged) {
        toast.success("Password updated", {
          id: "settings-security-success",
        });
      } else {
        toast.info("No changes to save", {
          id: "settings-security-info",
        });
      }
    },
    onError: (error) => {
      toast.error(error.message, {
        id: "settings-security-error",
      });
    },
  });

  async function onSubmit(values: EmailPasswordValues) {
    await updateSecurityMutation.mutateAsync(values);
  }

  return (
    <form onSubmit={form.handleSubmit(onSubmit)} className="grid grid-cols-1 gap-8 lg:grid-cols-3">
      <SectionHeader
        title="Email & Password"
        description="Manage your email and password settings."
      />

      <div className="space-y-6 lg:col-span-2">
        <FormField
          control={form.control}
          name="email"
          label="Email"
          type="email"
          placeholder="Email address"
          autoComplete="email"
          required
        />

        <FormField
          control={form.control}
          name="currentPassword"
          label="Current Password"
          type="password"
          placeholder="Password"
          autoComplete="current-password"
        />

        <div className="space-y-2">
          <FormField
            control={form.control}
            name="newPassword"
            label="New Password"
            type="password"
            placeholder="Password"
            autoComplete="new-password"
          />

          <div className="flex gap-1.5 pt-1">
            {Array.from({ length: 4 }).map((_, i) => (
              <div
                key={i}
                className={cn(
                  "h-1.5 flex-1 rounded-full bg-muted",
                  i < filledSegments && strengthColor,
                )}
              />
            ))}
          </div>

          <p className="pt-2 text-sm text-foreground">Enter a password. Must contain:</p>
          <ul className="space-y-1.5">
            {PASSWORD_RULES.map((rule) => {
              const passed = rule.test(newPassword);
              return (
                <li key={rule.id} className="flex items-center gap-2 text-sm">
                  {passed ? (
                    <CheckIcon className="h-3.5 w-3.5 shrink-0 text-green-600" />
                  ) : (
                    <XIcon className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
                  )}
                  <span className={passed ? "text-foreground" : "text-muted-foreground"}>
                    {rule.label}
                  </span>
                </li>
              );
            })}
          </ul>
        </div>

        <div className="flex justify-end">
          <Button
            type="submit"
            disabled={!user || updateSecurityMutation.isPending}
            className="bg-blue-600 hover:bg-blue-700"
          >
            {updateSecurityMutation.isPending ? "Saving..." : "Save Changes"}
          </Button>
        </div>
      </div>
    </form>
  );
}

function WorkspaceSection() {
  return (
    <div className="grid grid-cols-1 gap-8 lg:grid-cols-3">
      <SectionHeader title="Workspace" description="Manage your workspace preferences." />
      <div className="lg:col-span-2">
        <p className="text-sm text-muted-foreground">Workspace settings are not available yet.</p>
      </div>
    </div>
  );
}

export default function AccountSettingsPage() {
  const currentUserQuery = useQuery({
    queryKey: currentUserQueryKey,
    queryFn: getCurrentUser,
  });

  useEffect(() => {
    if (currentUserQuery.error) {
      toast.error(currentUserQuery.error.message, {
        id: "settings-current-user-error",
      });
    }
  }, [currentUserQuery.error]);

  return (
    <div className="w-full pb-10">
      <div className="mb-6">
        <h1 className="text-2xl font-bold tracking-tight">Account & User Management</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Manage your account settings and user preferences.
        </p>
      </div>

      <Tabs defaultValue="general" className="w-full">
        <TabsList className="h-auto w-full justify-start gap-6 rounded-none border-b bg-transparent p-0">
          <TabsTrigger value="general" className={tabTriggerClass}>
            General
          </TabsTrigger>
          <TabsTrigger value="workspace" className={tabTriggerClass}>
            Workspace
          </TabsTrigger>
        </TabsList>

        <TabsContent value="general" className="mt-8 space-y-10">
          <PersonalInformationSection user={currentUserQuery.data} />
          <Separator className="my-2" />
          <EmailPasswordSection user={currentUserQuery.data} />
        </TabsContent>

        <TabsContent value="workspace" className="mt-8">
          <WorkspaceSection />
        </TabsContent>
      </Tabs>
    </div>
  );
}
