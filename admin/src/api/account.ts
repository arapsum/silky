import { apiRequest } from "#/api/client.ts";

export const currentUserQueryKey = ["current-user"] as const;

export type CurrentUser = {
  pid: string;
  email: string;
  name: string;
  image: string | null;
  verified: boolean;
  createdAt: string;
  updatedAt: string;
};

export type UpdateProfileInput = {
  name: string;
  email: string;
  image?: string;
};

export type ChangePasswordInput = {
  currentPassword: string;
  password: string;
  confirmPassword: string;
};

type MessageResponse = {
  message: string;
};

type CurrentUserOptions = {
  sessionExpiredMode?: "handle" | "throw";
};

export function getCurrentUser() {
  return apiRequest<CurrentUser>("/auth/me", {
    fallback: "Unable to load account details",
  });
}

export function getCurrentUserForAuthGuard(options: CurrentUserOptions = {}) {
  return apiRequest<CurrentUser>("/auth/me", {
    fallback: "Unable to load account details",
    sessionExpiredMode: options.sessionExpiredMode,
  });
}

export function updateCurrentUser(input: UpdateProfileInput) {
  return apiRequest<CurrentUser>("/auth/me", {
    method: "PATCH",
    body: JSON.stringify(input),
    fallback: "Unable to update account details",
  });
}

export function changePassword(input: ChangePasswordInput) {
  return apiRequest<MessageResponse>("/auth/change-password", {
    method: "POST",
    body: JSON.stringify(input),
    fallback: "Unable to change password",
  });
}
