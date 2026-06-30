# Silk Admin

React admin frontend for the Silk fashion e-commerce project.

## Stack

- TanStack Start and Vite
- React 19
- TanStack Router with file-based routes
- TanStack Query
- Tailwind CSS
- shadcn-compatible component tooling
- T3 Env for typed environment variables
- Vitest for tests

## Requirements

- Node.js
- pnpm
- The Silk service for backend API behavior

Start the backend from the repository root or `service/` directory before
working on API-backed admin features.

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
pnpm generate-routes   # regenerate the TanStack Router route tree
```

## Environment

Typed environment variables are defined in `src/env.ts`.

Supported variables:

- `SERVER_URL` - optional server-side API base URL
- `VITE_APP_TITLE` - optional client/browser app title

Client-side variables must use the `VITE_` prefix.

Example `.env`:

```env
SERVER_URL=http://127.0.0.1:7150
VITE_APP_TITLE=Silk Admin
```

Use environment values through the shared env module:

```ts
import { env } from "#/env";

console.log(env.VITE_APP_TITLE);
```

## Routing

Routes live in `src/routes/` and are managed by TanStack Router's file-based
routing.

Important files:

- `src/routes/__root.tsx` - root document, global styles, devtools, and router
  context
- `src/routes/index.tsx` - current home route
- `src/routeTree.gen.ts` - generated route tree

Regenerate the route tree after route changes:

```bash
pnpm generate-routes
```

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

## Project Layout

```text
.
|-- public/                         # Static assets and web manifest
|-- src/
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

## Current State

The admin app is still close to the TanStack Start starter template. Most
domain behavior currently lives in the backend service, so expect admin routes,
forms, and API integration to evolve as the product surface is built out.
