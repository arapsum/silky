# Silk

Open-source fashion e-commerce project built with a Rust API and a TypeScript
admin frontend.

## Overview

Silk is currently split into two applications:

- `service/` - Rust backend API built with Axum, SQLx, PostgreSQL, Redis,
  Apalis workers, JWT authentication, and Handlebars email templates.
- `admin/` - React admin frontend built with TanStack Start, TanStack Router,
  TanStack Query, Vite, Tailwind CSS, Base UI, and shadcn-compatible UI
  tooling.

Local infrastructure is defined in `compose.yaml` and includes PostgreSQL,
Redis, and Mailtutan for catching development emails.

## Requirements

- Rust toolchain with Cargo
- Node.js and pnpm
- Docker or another Compose-compatible runtime
- PostgreSQL client tools are optional, but useful for inspecting the database

## Quick Start

Create a root `.env` file for Docker Compose. The development configuration
expects these values:

```env
POSTGRES_USER=username
POSTGRES_PASSWORD=password
POSTGRES_DB=database

DATABASE_URL=postgresql://username:password@localhost:5432/database
```

Start the local dependencies:

```bash
docker compose up -d
```

Run the backend API:

```bash
cd service
cargo run
```

The service listens on `http://127.0.0.1:7150` in development.

Run the admin frontend in a second terminal:

```bash
cd admin
pnpm install
pnpm dev
```

Vite will print the local admin URL when the dev server starts.

The admin app calls the backend through `VITE_SERVER_URL`. If it is not set,
the client defaults to `http://127.0.0.1:7150/api`.

## Backend

The backend loads configuration from `service/config/<environment>.yaml`.
Development is the default environment. You can choose another environment with
the `--env` flag:

```bash
cd service
cargo run -- --env testing
```

Configuration values can also be overridden with `APP_`-prefixed environment
variables. For example, `APP_SERVER_PORT=8080` overrides `server.port`.

Useful commands:

```bash
cd service
cargo run              # start the API
cargo run -- seed      # seed initial users from src/data/users.json
cargo test             # run backend tests
cargo fmt              # format Rust code
cargo clippy           # run Rust lints
```

Development JWT keys are configured under `service/secrets/keys/dev/`. Test
keys are configured under `service/secrets/keys/test/`.

### API Routes

All backend routes are mounted under `/api`.

| Method | Path | Description |
| --- | --- | --- |
| `GET` | `/api/health` | Health check |
| `POST` | `/api/auth/register` | Register a user and queue a verification email |
| `POST` | `/api/auth/login` | Log in, issue auth cookies, and return an access token |
| `POST` | `/api/auth/refresh` | Rotate refresh token cookies and return a new access token |
| `POST` | `/api/auth/logout` | Revoke the refresh token and clear auth cookies |
| `GET` | `/api/auth/verify/{token}` | Verify a user's email address |
| `POST` | `/api/auth/forgot-password` | Queue a password reset email when the account exists |
| `POST` | `/api/auth/reset-password` | Reset a password with a reset token |
| `POST` | `/api/auth/change-password` | Change the authenticated user's password |
| `GET` | `/api/auth/me` | Return the current authenticated user |

Mailtutan exposes the local email inbox at `http://localhost:1080`; SMTP is
available on `localhost:1025`.

## Admin Frontend

Useful commands:

```bash
cd admin
pnpm install           # install dependencies
pnpm dev               # start the Vite dev server
pnpm build             # build for production
pnpm preview           # preview the production build
pnpm test              # run Vitest
pnpm lint              # run oxlint
pnpm fmt:check         # check formatting with oxfmt
pnpm fmt               # format with oxfmt
pnpm generate-routes   # regenerate TanStack Router route tree
```

The admin app supports these environment variables:

- `SERVER_URL` - optional server-side API base URL
- `VITE_SERVER_URL` - optional browser/client API base URL
- `VITE_APP_TITLE` - optional browser/client app title

The admin currently includes a sign-in flow backed by React Hook Form, Zod,
TanStack Query, and Sonner toasts. The dashboard shell includes a floating
sidebar, top navbar, command-search trigger, notifications popover, theme
toggle, and account menu.

## Project Layout

```text
.
|-- admin/                 # TanStack Start admin frontend
|   |-- src/api/           # Frontend API client functions and shared errors
|   |-- src/components/    # App and UI components
|   |-- src/routes/        # TanStack Router file routes
|-- service/               # Axum backend API
|   |-- config/            # Environment-specific YAML config
|   |-- migrations/        # SQLx database migrations
|   |-- secrets/keys/      # Development and testing JWT keys
|   |-- src/               # Application source
|   |-- templates/         # Email templates
|   `-- tests/             # Integration and model tests
|-- compose.yaml           # PostgreSQL, Redis, and Mailtutan services
`-- README.md
```

## Development Notes

- Database migrations run automatically when `database.auto_migrate` is enabled
  in the selected service config.
- The testing config uses port `7175` and can recreate the database schema when
  tests start.
- Redis is used by Apalis-backed mail workers and by auth refresh-token storage
  so refresh tokens are single-use and can be revoked on logout.
- Auth cookies are issued by the backend. Browser requests that need cookies
  should use credentials-enabled requests from the frontend.
- `admin/src/routeTree.gen.ts` is generated by TanStack Router and should be
  regenerated with `pnpm generate-routes` after route changes.
