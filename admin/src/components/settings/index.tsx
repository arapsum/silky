"use client";

import { useForm } from "react-hook-form";
import { useRef, useState, type ChangeEvent } from "react";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { CheckIcon, ImageIcon, TrashIcon, UploadSimpleIcon, XIcon } from "@phosphor-icons/react";

import { cn } from "@/lib/utils";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import FormField from "@/components/form-field";
import { Separator } from "../ui/separator";

// ---------------------------------------------------------------------------
// Static data
// ---------------------------------------------------------------------------

const COUNTRIES = [
  "United States",
  "United Kingdom",
  "Canada",
  "Kenya",
  "Nigeria",
  "South Africa",
  "Germany",
  "France",
  "India",
  "Australia",
] as const;

const COUNTRY_OPTIONS = COUNTRIES.map((country) => ({ label: country, value: country }));

const GENDER_OPTIONS = [
  { label: "Male", value: "male" },
  { label: "Female", value: "female" },
  { label: "Other", value: "other" },
];

const ROLES = ["admin", "editor", "viewer", "member"] as const;

const ROLE_OPTIONS = ROLES.map((role) => ({
  label: role.charAt(0).toUpperCase() + role.slice(1),
  value: role,
}));

const PASSWORD_RULES: { id: string; label: string; test: (v: string) => boolean }[] = [
  { id: "length", label: "At least 12 characters", test: (v) => v.length >= 12 },
  { id: "lowercase", label: "At least 1 lowercase letter", test: (v) => /[a-z]/.test(v) },
  { id: "uppercase", label: "At least 1 uppercase letter", test: (v) => /[A-Z]/.test(v) },
  { id: "number", label: "At least 1 number", test: (v) => /[0-9]/.test(v) },
  { id: "special", label: "At least 1 special character", test: (v) => /[^A-Za-z0-9]/.test(v) },
];

const tabTriggerClass =
  "relative rounded-none border-b-2 border-transparent bg-transparent px-0.5 pb-3 pt-0 font-medium text-muted-foreground shadow-none transition-colors hover:text-foreground data-[state=active]:border-foreground data-[state=active]:bg-transparent data-[state=active]:text-foreground data-[state=active]:shadow-none";

// ---------------------------------------------------------------------------
// Schemas
// ---------------------------------------------------------------------------

const personalInfoSchema = z.object({
  firstName: z.string().min(1, "First name is required"),
  lastName: z.string().min(1, "Last name is required"),
  mobile: z.string().optional(),
  country: z.string().min(1, "Select a country"),
  gender: z.enum(["male", "female", "other"]),
  role: z.enum(ROLES),
});

type PersonalInfoValues = z.infer<typeof personalInfoSchema>;

const emailPasswordSchema = z.object({
  email: z.string().min(1, "Email is required").email("Enter a valid email address"),
  currentPassword: z.string().min(1, "Current password is required"),
  newPassword: z
    .string()
    .min(1, "New password is required")
    .refine((v) => PASSWORD_RULES.every((rule) => rule.test(v)), {
      message: "Password does not meet all requirements",
    }),
});

type EmailPasswordValues = z.infer<typeof emailPasswordSchema>;

// ---------------------------------------------------------------------------
// Shared bits
// ---------------------------------------------------------------------------

function SectionHeader({ title, description }: { title: string; description: string }) {
  return (
    <div className="lg:col-span-1">
      <h2 className="text-base font-semibold">{title}</h2>
      <p className="mt-1 text-sm text-muted-foreground">{description}</p>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Personal Information
// ---------------------------------------------------------------------------

function PersonalInformationSection() {
  const [avatarUrl, setAvatarUrl] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const { control, handleSubmit } = useForm<PersonalInfoValues>({
    resolver: zodResolver(personalInfoSchema),
    defaultValues: {
      firstName: "",
      lastName: "",
      mobile: "",
      country: "",
      gender: "male",
      role: "admin",
    },
  });

  function handleAvatarChange(e: ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    if (!file) return;
    setAvatarUrl(URL.createObjectURL(file));
  }

  function handleRemoveAvatar() {
    setAvatarUrl(null);
    if (fileInputRef.current) fileInputRef.current.value = "";
  }

  async function onSubmit(values: PersonalInfoValues) {
    setSaving(true);
    // TODO: wire this up to your API
    console.log("personal info", values);
    await new Promise((resolve) => setTimeout(resolve, 500));
    setSaving(false);
  }

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="grid grid-cols-1 gap-8 lg:grid-cols-3">
      <SectionHeader
        title="Personal Information"
        description="Manage your personal information and role."
      />

      <div className="space-y-6 lg:col-span-2">
        <div>
          <Label>Your Avatar</Label>
          <div className="mt-2 flex items-center gap-3">
            <Avatar className="h-16 w-16 border">
              <AvatarImage src={avatarUrl ?? undefined} alt="Avatar" />
              <AvatarFallback className="bg-muted">
                <ImageIcon className="h-6 w-6 text-muted-foreground" />
              </AvatarFallback>
            </Avatar>
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => fileInputRef.current?.click()}
            >
              <UploadSimpleIcon className="mr-2 h-4 w-4" />
              Upload avatar
            </Button>
            <Button
              type="button"
              variant="ghost"
              size="icon"
              className="text-red-500 hover:bg-red-50 hover:text-red-600"
              onClick={handleRemoveAvatar}
            >
              <TrashIcon className="h-4 w-4" />
            </Button>
            <input
              ref={fileInputRef}
              type="file"
              accept="image/*"
              className="hidden"
              onChange={handleAvatarChange}
            />
          </div>
          <p className="mt-2 text-xs text-muted-foreground">Pick a photo up to 1MB.</p>
        </div>

        <div className="grid grid-cols-1 gap-6 sm:grid-cols-2">
          <FormField control={control} name="firstName" label="First Name" placeholder="John" />
          <FormField control={control} name="lastName" label="Last Name" placeholder="Doe" />
          <FormField
            control={control}
            name="mobile"
            label="Mobile"
            placeholder="+1 (555) 123-4567"
          />
          <FormField
            control={control}
            name="country"
            label="Country"
            type="select"
            placeholder="Select Country"
            options={COUNTRY_OPTIONS}
          />
          <FormField
            control={control}
            name="gender"
            label="Gender"
            type="select"
            options={GENDER_OPTIONS}
          />
          <FormField
            control={control}
            name="role"
            label="Role"
            type="select"
            options={ROLE_OPTIONS}
          />
        </div>

        <div className="flex justify-end">
          <Button type="submit" disabled={saving} className="bg-blue-600 hover:bg-blue-700">
            {saving ? "Saving..." : "Save Changes"}
          </Button>
        </div>
      </div>
    </form>
  );
}

// ---------------------------------------------------------------------------
// Email & Password
// ---------------------------------------------------------------------------

function EmailPasswordSection() {
  const [saving, setSaving] = useState(false);

  const { control, handleSubmit, watch } = useForm<EmailPasswordValues>({
    resolver: zodResolver(emailPasswordSchema),
    defaultValues: { email: "", currentPassword: "", newPassword: "" },
  });

  const newPassword = watch("newPassword");
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

  async function onSubmit(values: EmailPasswordValues) {
    setSaving(true);
    // TODO: wire this up to your API
    console.log("email & password", values);
    await new Promise((resolve) => setTimeout(resolve, 500));
    setSaving(false);
  }

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="grid grid-cols-1 gap-8 lg:grid-cols-3">
      <SectionHeader
        title="Email & Password"
        description="Manage your email and password settings."
      />

      <div className="space-y-6 lg:col-span-2">
        <FormField
          control={control}
          name="email"
          label="Email"
          type="email"
          placeholder="Email address"
          required
        />

        <FormField
          control={control}
          name="currentPassword"
          label="Current Password"
          type="password"
          placeholder="Password"
          required
        />

        <div className="space-y-2">
          <FormField
            control={control}
            name="newPassword"
            label="New Password"
            type="password"
            placeholder="Password"
            required
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

          <p className="pt-2 text-sm text-foreground">Enter a password. Must contain :</p>
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
          <Button type="submit" disabled={saving} className="bg-blue-600 hover:bg-blue-700">
            {saving ? "Saving..." : "Save Changes"}
          </Button>
        </div>
      </div>
    </form>
  );
}

// ---------------------------------------------------------------------------
// Workspace (placeholder — not specified in the design reference)
// ---------------------------------------------------------------------------

function WorkspaceSection() {
  return (
    <div className="grid grid-cols-1 gap-8 lg:grid-cols-3">
      <SectionHeader title="Workspace" description="Manage your workspace preferences." />
      <div className="lg:col-span-2">
        <p className="text-sm text-muted-foreground">
          No workspace fields were shown in the reference design — drop your own fields in here.
        </p>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Page
// ---------------------------------------------------------------------------

export default function AccountSettingsPage() {
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
          <PersonalInformationSection />
          <Separator className="my-2" />
          <EmailPasswordSection />
        </TabsContent>

        <TabsContent value="workspace" className="mt-8">
          <WorkspaceSection />
        </TabsContent>
      </Tabs>
    </div>
  );
}
