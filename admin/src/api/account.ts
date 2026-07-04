import { apiRequest } from "#/api/client.ts";

export type CurrentUser = {
  pid: string;
  email: string;
  name: string;
  verified: boolean;
  createdAt: string;
  updatedAt: string;
};

export type UpdateProfileInput = {
  name: string;
  email: string;
};

export type ChangePasswordInput = {
  currentPassword: string;
  password: string;
  confirmPassword: string;
};

type MessageResponse = {
  message: string;
};

export function getCurrentUser() {
  return apiRequest<CurrentUser>("/auth/me", {
    fallback: "Unable to load account details",
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
