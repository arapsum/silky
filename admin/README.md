# Silk Admin

React admin frontend for the Silk fashion e-commerce project.

## Stack

- TanStack Start and Vite
- React 19
- TanStack Router with file-based routes
- TanStack Query
- Tailwind CSS
- shadcn-compatible component tooling with Base UI primitives
- React Hook Form, Zod, and `@hookform/resolvers`
- Sonner for toast notifications
- next-themes for theme state
- T3 Env for typed environment variables
- Oxlint and oxfmt
- Vitest for tests

## Requirements

- Node.js
- pnpm
- The Silk service for backend API behavior

Start the backend from the repository root or `service/` directory before using
API-backed admin features such as sign-in.

## Getting Started

Install dependencies:

```bash
pnpm install
```

Start the development server:

```bash
pnpm dev
```

Vite prints the local URL when the server starts.

## Commands

```bash
pnpm dev               # start the Vite dev server
pnpm build             # build for production
pnpm preview           # preview the production build
pnpm test              # run Vitest
pnpm lint              # run oxlint
pnpm lint:fix          # run oxlint fixes
pnpm fmt               # format with oxfmt
pnpm fmt:check         # check formatting with oxfmt
pnpm generate-routes   # regenerate the TanStack Router route tree
```

## Environment

Typed environment variables are defined in `src/env.ts`.

Supported variables:

- `VITE_SERVER_URL` - browser API base URL; defaults to
  `http://127.0.0.1:7150/api`
- Cloudinary credentials are not exposed to the browser. Configure the service
  with `APP_CLOUDINARY_CLOUD_NAME`, `APP_CLOUDINARY_API_KEY`, and
  `APP_CLOUDINARY_API_SECRET`.
- `SERVER_URL` and `VITE_APP_TITLE` - optional values declared in the typed
  environment schema but not currently consumed by the app

Client-side variables must use the `VITE_` prefix.

Example `.env`:

```env
VITE_SERVER_URL=http://127.0.0.1:7150/api
```

The service signs uploads into the `silk/users`, `silk/categories`, and
`silk/products` folders respectively. Uploaded files are registered in the
service's `media_assets` table before they are linked to a user, category, or
product picture.

Use environment values through the shared env module:

```ts
import { env } from "#/env";

console.log(env.VITE_SERVER_URL);
```

## Routing

Routes live in `src/routes/` and are managed by TanStack Router's file-based
routing.

Important files:

- `src/routes/__root.tsx` - root document, global styles, devtools, and router
  context
- `src/routes/_auth/sign-in/index.tsx` - sign-in page
- `src/routes/_main/route.tsx` - authenticated dashboard layout and session
  guard
- `src/routes/_main/` - protected catalogue, people, access-control, and
  settings routes
- `src/routeTree.gen.ts` - generated route tree

Regenerate the route tree after route changes:

```bash
pnpm generate-routes
```

`src/routeTree.gen.ts` is generated and should not be edited manually.

## Authentication And API

API helpers live in `src/api/`. `src/api/client.ts` owns the API base URL,
credentialed requests, shared error parsing, and session refresh. Resource
modules cover authentication, the current account, categories, products,
users, roles, permissions, and Cloudinary uploads.

The sign-in page uses `LoginForm`, React Hook Form, Zod validation,
`@hookform/resolvers`, TanStack Query mutation state, and Sonner toasts. On a
successful sign-in it redirects to `/`.

The service sets auth cookies during login. The frontend sends credentialed
requests so the browser can accept and return those cookies; it does not create
or overwrite service auth cookies on the client. Protected routes verify the
current user before rendering. If an access session expires, the API client
coalesces concurrent refresh attempts, retries the original requests, and
redirects to `/sign-in` if refresh fails.

## Data Fetching

TanStack Query is wired through `src/integrations/tanstack-query/`. Use it for
server state and API-backed admin views. The root provider owns the shared query
client and devtools integration.

## Styling And UI

Global styles live in `src/styles.css`. Tailwind CSS is configured through the
Vite plugin.

The project includes shadcn-compatible setup in `components.json`. Add UI
components with:

```bash
pnpm dlx shadcn@latest add button
```

Use existing component and utility conventions before introducing new styling
patterns.

## Implemented Screens

The authenticated dashboard shell provides a floating sidebar, navbar,
command-search trigger, notifications, theme toggle, and account menu. Its
API-backed screens currently include:

- Category listing, filtering, creation, image upload, and deletion
- Product listing, filtering, creation, detail, editing, variants, image
  management, and deletion
- Customer and staff user lists
- Role creation and editing, permission assignment, and permission browsing
- Account profile, avatar, and password settings

Several additional sidebar entries are roadmap placeholders and currently use
`#` links. Sidebar and navbar components live under `src/components/sidebar/`
and `src/components/navbar.tsx`.

## Project Layout

```text
.
|-- public/                         # Static assets and web manifest
|-- src/
|   |-- api/                         # API clients and shared error helpers
|   |-- components/                  # App components and UI primitives
|   |-- integrations/tanstack-query/ # Query provider and devtools
|   |-- lib/                         # Shared frontend utilities
|   |-- routes/                      # File-based routes
|   |-- env.ts                       # Typed environment variables
|   |-- router.tsx                   # Router setup
|   |-- routeTree.gen.ts             # Generated route tree
|   `-- styles.css                   # Global styles
|-- components.json                  # shadcn-compatible UI config
|-- package.json
|-- tsconfig.json
`-- vite.config.ts
```
