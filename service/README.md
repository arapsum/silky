# Silk Service

Rust backend API for the Silk fashion e-commerce project.

## Stack

- Axum for HTTP routing
- SQLx with PostgreSQL for persistence and migrations
- Redis and Apalis for background mail jobs
- Lettre and Handlebars for email delivery and templates
- JWT authentication with RSA keys
- Insta, rstest, serial_test, and axum-test for tests

## Requirements

- Rust toolchain with Cargo
- PostgreSQL
- Redis
- Local SMTP server for development emails

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

## Development

Start the API:

```bash
cargo run
```

The development API listens on:

```text
http://127.0.0.1:7150
```

Seed initial users:

```bash
cargo run -- seed
```

Seed data is read from `src/data/users.json`.

## Commands

```bash
cargo run              # start the API
cargo run -- seed      # seed development data
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
| `POST` | `/api/auth/login` | Log in and return an access token |
| `GET` | `/api/auth/verify/{token}` | Verify a user's email address |
| `POST` | `/api/auth/forgot-password` | Queue a password reset email when the account exists |
| `POST` | `/api/auth/reset-password` | Reset a password with a reset token |
| `GET` | `/api/auth/me` | Return the current authenticated user |

Request tests mount the controller router directly, so test paths omit the
outer `/api` prefix. For example, the service route `/api/auth/login` is tested
as `/auth/login`.

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
review generated `.snap.new` files before accepting them.

## Project Layout

```text
.
|-- config/          # Environment-specific YAML configuration
|-- migrations/      # SQLx migrations
|-- secrets/keys/    # Development and testing JWT keys
|-- src/
|   |-- controllers/ # HTTP route handlers
|   |-- middlewares/ # Auth and tracing layers
|   |-- models/      # Database models and persistence logic
|   |-- schemas/     # Request validation schemas
|   |-- views/       # Response DTOs
|   `-- workers/     # Background jobs
|-- templates/       # Email templates
`-- tests/           # Model and request tests
```
