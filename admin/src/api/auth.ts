import { API_BASE_URL, getErrorResponse } from "#/api/client.ts";

export type LoginCredentials = {
  email: string;
  password: string;
};

export type LoginResponse = {
  pid: string;
  email: string;
  name: string;
  token: string;
  verified: boolean;
};

export async function login(credentials: LoginCredentials): Promise<LoginResponse> {
  const response = await fetch(`${API_BASE_URL}/auth/login`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
    credentials: "include",
    body: JSON.stringify(credentials),
  });

  if (!response.ok) {
    const fallback = response.status === 401 ? "Invalid email or password" : "Unable to sign in";
    throw new Error(await getErrorResponse(response, fallback));
  }

  return (await response.json()) as LoginResponse;
}
