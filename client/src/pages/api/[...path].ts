import type { APIRoute } from "astro";

const DEFAULT_API_URL = "http://127.0.0.1:7150/api";

const routes = [
  /^(GET)\/(products|categories)(\/.*)?$/,
  /^(POST)\/cart\/quote$/,
  /^(GET|POST|PATCH)\/auth\/(login|register|refresh|logout|forgot-password|reset-password|me)(\/.*)?$/,
  /^(GET)\/auth\/verify\/[0-9a-f-]+$/i,
  /^(GET|POST)\/addresses\/?$/,
  /^(GET|PUT|DELETE)\/addresses\/[0-9a-f-]+$/i,
  /^(GET|POST)\/orders\/?$/,
  /^(POST)\/orders\/checkout$/,
  /^(GET)\/orders\/[0-9a-f-]+$/i,
  /^(GET)\/orders\/[0-9a-f-]+\/checkout-session$/i,
  /^(POST)\/orders\/[0-9a-f-]+\/checkout-session\/cancel$/i,
];

function isAllowed(method: string, path: string) {
  return routes.some((route) => route.test(`${method}${path}`));
}

export const ALL: APIRoute = async ({ request, params }) => {
  const path = `/${params.path ?? ""}`;
  if (!isAllowed(request.method, path)) {
    return Response.json({ error: "This storefront operation is not available." }, { status: 404 });
  }

  const baseUrl = (import.meta.env.API_URL ?? DEFAULT_API_URL).replace(/\/$/, "");
  const incomingUrl = new URL(request.url);
  const target = `${baseUrl}${path}${incomingUrl.search}`;
  const headers = new Headers(request.headers);
  headers.delete("host");
  headers.delete("content-length");
  headers.set("accept", "application/json");

  const upstream = await fetch(target, {
    method: request.method,
    headers,
    body: ["GET", "HEAD"].includes(request.method) ? undefined : await request.arrayBuffer(),
    redirect: "manual",
  });
  const responseHeaders = new Headers(upstream.headers);
  responseHeaders.delete("content-encoding");
  responseHeaders.delete("content-length");
  responseHeaders.delete("set-cookie");
  for (const cookie of upstream.headers.getSetCookie()) {
    responseHeaders.append("set-cookie", cookie);
  }

  return new Response(upstream.body, {
    status: upstream.status,
    statusText: upstream.statusText,
    headers: responseHeaders,
  });
};
