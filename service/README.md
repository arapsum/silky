# Silk Service

Rust backend API for the Silk fashion e-commerce project.

## Stack

- Axum for HTTP routing
- SQLx with PostgreSQL for persistence and migrations
- Redis and Apalis for background mail jobs
- Redis-backed refresh token storage for single-use token rotation
- Lettre and Handlebars for email delivery and templates
- JWT authentication with RSA keys
- Insta, rstest, serial_test, and axum-test for tests

## Requirements

- Rust toolchain with Cargo
- PostgreSQL
- Redis
- Local SMTP server for development emails
- A Stripe test account and Stripe CLI for payment development

The root `compose.yaml` starts PostgreSQL, Redis, and Mailtutan:

```bash
cd ..
docker compose up -d
```

Mailtutan exposes its web inbox at `http://localhost:1080` and SMTP on
`localhost:1025`.

## Configuration

Configuration is loaded from `config/<environment>.yaml` in the current working
directory. Development is the default environment.

Available local configs:

- `config/development.yaml` - default service config, listens on port `7150`
- `config/testing.yaml` - test config, listens on port `7175`
- `config/base.yaml` - shared/reference values

Run with a specific environment:

```bash
cargo run -- --env testing
```

Environment variables prefixed with `APP_` override YAML config values. For
example:

```bash
APP_SERVER_PORT=8080 cargo run
```

JWT key paths are configured in YAML. Development keys live under
`secrets/keys/dev/`; test keys live under `secrets/keys/test/`.

### Stripe Checkout

Stripe is optional. The service enables Checkout only when all required
settings are present. Keep these values in `service/.env`; never commit them:

```bash
APP_STRIPE_SECRET_KEY=sk_test_...
APP_STRIPE_WEBHOOK_SECRET=whsec_...
APP_STRIPE_CHECKOUT_SUCCESS_URL=http://localhost:3000/checkout/success?session_id={CHECKOUT_SESSION_ID}
APP_STRIPE_CHECKOUT_CANCEL_URL=http://localhost:3000/checkout/cancel
APP_STRIPE_CHECKOUT_TTL_SECONDS=1800
APP_STRIPE_LIVE_MODE=false
```

Authenticate the Stripe CLI and forward only the events Silk handles:

```bash
stripe login
stripe listen \
  --events checkout.session.completed,checkout.session.expired,checkout.session.async_payment_succeeded,checkout.session.async_payment_failed \
  --forward-to http://127.0.0.1:7150/api/payments/stripe/webhook
```

Copy the command's `whsec_...` signing secret into
`APP_STRIPE_WEBHOOK_SECRET`, then restart the service. Development should use
Stripe test keys with `APP_STRIPE_LIVE_MODE=false`; production must use live
keys, an HTTPS webhook destination, and `APP_STRIPE_LIVE_MODE=true`.

## Development

Start the API:

```bash
cargo run
```

The development API listens on:

```text
http://127.0.0.1:7150
```

Seed initial data:

```bash
cargo run -- seed
```

Seed data is read from JSON files in `src/data/`. The seed command currently
loads users, roles, permissions, role-permission assignments, categories, product
catalog records, product variants, attributes, pictures, and user-role
assignments. After seeding, the process continues by starting the API and mail
workers.

## Commands

```bash
cargo run              # start the API
cargo run -- seed      # seed development data, then start the API
cargo test             # run tests
cargo fmt              # format Rust code
cargo clippy           # run lints
```

## API Routes

Routes are mounted under `/api` when the binary starts the full application.

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
| `PATCH` | `/api/auth/me` | Update the current user's name, email, and optional profile image URL |
| `GET` | `/api/categories` | List and filter categories with pagination |
| `GET` | `/api/categories/{pid}` | Return a category by public ID |
| `POST` | `/api/categories` | Create a category; requires `categories:create` |
| `PATCH` | `/api/categories/{pid}` | Update a category; requires `categories:update` |
| `DELETE` | `/api/categories/{pid}` | Soft-delete a category; requires `categories:delete` |
| `GET` | `/api/products` | List and filter products with pagination |
| `GET` | `/api/products/{pid}` | Return product details, variants, options, and pictures |
| `GET` | `/api/products/attributes` | List product attributes and values; requires `products:read` |
| `POST` | `/api/products` | Create a product; requires `products:create` |
| `PATCH` | `/api/products/{pid}` | Update product metadata; requires `products:update` |
| `DELETE` | `/api/products/{pid}` | Soft-delete a product; requires `products:delete` |
| `POST` | `/api/products/{pid}/variants` | Add a variant; requires `products:update` |
| `PATCH` | `/api/products/{pid}/variants/{variant_pid}` | Update a variant; requires `products:update` |
| `DELETE` | `/api/products/{pid}/variants/{variant_pid}` | Soft-delete a variant; requires `products:delete` |
| `POST` | `/api/products/{pid}/variants/{variant_pid}/default` | Make a variant the default; requires `products:update` |
| `POST` | `/api/products/{pid}/pictures` | Add a product picture; requires `products:update` |
| `PATCH` | `/api/products/{pid}/pictures/{picture_pid}` | Update product picture order; requires `products:update` |
| `DELETE` | `/api/products/{pid}/pictures/{picture_pid}` | Delete a product picture; requires `products:delete` |
| `POST` | `/api/products/{pid}/variants/{variant_pid}/pictures` | Add a variant picture; requires `products:update` |
| `PATCH` | `/api/products/{pid}/variants/{variant_pid}/pictures/{picture_pid}` | Update variant picture order; requires `products:update` |
| `DELETE` | `/api/products/{pid}/variants/{variant_pid}/pictures/{picture_pid}` | Delete a variant picture; requires `products:delete` |
| `GET` | `/api/orders` | List orders; customers are restricted to their own orders |
| `GET` | `/api/orders/{pid}` | Return an order and its immutable line snapshots |
| `POST` | `/api/orders/checkout` | Reserve stock and create Stripe Checkout; requires the `customer` role |
| `PATCH` | `/api/orders/{pid}` | Update staff-managed order and fulfillment state; requires `orders:update` |
| `POST` | `/api/payments/stripe/webhook` | Receive and verify Stripe lifecycle events |
| `GET` | `/api/users` | List users, optionally by role; requires `users:read` |
| `GET` | `/api/roles` | List roles; requires authentication |
| `GET` | `/api/roles/{pid}` | Return a role by public ID; requires authentication |
| `POST` | `/api/roles` | Create a role; requires authentication |
| `PATCH` | `/api/roles/{pid}` | Update a role; requires authentication |
| `POST` | `/api/roles/permissions` | Assign a permission to a role; requires `roles:update` |
| `GET` | `/api/permissions` | List permissions; accepts optional `role` query and requires authentication |
| `GET` | `/api/permissions/{pid}` | Return a permission by public ID; requires authentication |

List endpoints use camel-case query parameters. Category filters are `page`,
`limit`, `search`, `name`, `slug`, `parentId`, `hasParent`, and
`includeDeleted`. Product filters are `page`, `limit`, `search`, `name`,
`categoryId`, `categorySlug`, `sku`, `minPrice`, `maxPrice`, `stockStatus`
(`inStock` or `outOfStock`), and `includeDeleted`. Deleted records are excluded
unless `includeDeleted=true` is supplied.

Request tests mount the controller router directly, so test paths omit the
outer `/api` prefix. For example, the service route `/api/auth/login` is tested
as `/auth/login`.

## Authentication

Login returns a JSON body with the user and access token, sets an `access_token`
cookie, and sets an HTTP-only `refresh_token` cookie. The access token may be
sent through the `Authorization: Bearer <token>` header or through the
`access_token` cookie for authenticated routes.

Refresh tokens are stored in Redis by token identifier when they are issued.
`POST /api/auth/refresh` consumes the stored identifier atomically before
issuing a new token pair. Reusing an already consumed refresh token is rejected.
`POST /api/auth/logout` removes the current refresh token identifier from Redis
and sends expired auth cookies.

Password endpoints:

- `POST /api/auth/forgot-password` accepts `{ "email": "user@example.com" }`
  and queues a reset email when the account exists.
- `POST /api/auth/reset-password` accepts `{ "token": "...", "password": "...",
  "confirmPassword": "..." }`.
- `POST /api/auth/change-password` requires authentication and accepts
  `{ "currentPassword": "...", "password": "...", "confirmPassword": "..." }`.

Profile endpoints:

- `GET /api/auth/me` returns the authenticated user, including `pid`, `email`,
  `name`, optional `image`, `verified`, `createdAt`, and `updatedAt`.
- `PATCH /api/auth/me` requires authentication and accepts
  `{ "name": "...", "email": "...", "image": "https://..." }`. The `image`
  field is optional and must be a valid URL when provided. Omitting it preserves
  the user's existing profile image.

Media uploads use a server-signed Cloudinary flow. Configure
`APP_CLOUDINARY_CLOUD_NAME`, `APP_CLOUDINARY_API_KEY`, and
`APP_CLOUDINARY_API_SECRET` in the service environment. Clients request a
signature from `POST /api/media/sign`, upload directly to Cloudinary, then
finalize the asset with `PUT /api/media/:pid/finalize`. The resulting asset PID
can be supplied as `mediaAssetPid` when updating a profile, category, or
product picture. The `media_assets` table is the registry and supports future
reference reconciliation and quarantine cleanup without deleting files that
are still linked.

## Testing

Run all service tests:

```bash
cargo test
```

Run only auth request tests:

```bash
cargo test requests::auth
```

The test environment uses `config/testing.yaml`. It connects to PostgreSQL and
can recreate the schema before tests run, so use an isolated development
database.

Snapshot tests are stored under `tests/**/snapshots/`. When behavior changes,
review generated `.snap.new` files before accepting them. To update snapshots
intentionally, run the relevant test with `INSTA_UPDATE=always`.

## Project Layout

```text
.
|-- config/          # Environment-specific YAML configuration
|-- migrations/      # SQLx migrations
|-- secrets/keys/    # Development and testing JWT keys
|-- src/
|   |-- controllers/ # HTTP route handlers
|   |-- data/        # JSON seed data
|   |-- middlewares/ # Auth and tracing layers
|   |-- models/      # Database models and persistence logic
|   |-- schemas/     # Request validation schemas
|   |-- views/       # Response DTOs
|   `-- workers/     # Background jobs
|-- templates/       # Email templates
`-- tests/           # Model and request tests
```
