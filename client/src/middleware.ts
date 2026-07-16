import { defineMiddleware } from "astro:middleware";

const protectedPaths = [/^\/checkout(?:\/|$)/, /^\/account(?:\/|$)/];

export const onRequest = defineMiddleware(async (context, next) => {
  if (
    protectedPaths.some((pattern) => pattern.test(context.url.pathname)) &&
    !context.cookies.has("access_token") &&
    !context.cookies.has("refresh_token")
  ) {
    const target = `${context.url.pathname}${context.url.search}`;
    return context.redirect(`/auth/login?next=${encodeURIComponent(target)}`);
  }
  return next();
});
