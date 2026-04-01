# AeroLedger

AeroLedger is a crypto-native aviation commerce platform for flight bookings and luxury aircraft purchases. The project combines a Rust backend with a premium Next.js frontend to support browsing, pricing, checkout, payment confirmation, and transaction tracking across multiple blockchains.

## What the project includes

- Flight booking flows and aircraft marketplace pages
- Crypto payment initialization and transaction-hash confirmation
- Multi-chain support for BTC, ETH, USDT, USDC, SOL, and BNB
- User authentication with JWT-based protected endpoints
- Admin and customer dashboard experiences
- Background reconciliation for pending payments

## Tech stack

### Backend

- Rust 2021
- Actix Web
- PostgreSQL
- Deadpool Postgres
- JWT authentication
- Chain integrations for EVM, Solana, and Bitcoin

### Frontend

- Next.js 14
- React 18
- TypeScript
- Tailwind CSS
- Framer Motion
- TanStack Query

## Repository layout

```text
.
|-- src/                    # Rust backend
|   |-- application/        # Use cases, commands, DTOs, queries
|   |-- domain/             # Entities, repositories, value objects, ports
|   |-- infrastructure/     # Config, DB, auth, blockchain, persistence, workers
|   |-- interfaces/         # HTTP handlers and middleware
|   |-- lib.rs
|   `-- main.rs
|-- frontend-luxury/        # Next.js frontend
|   |-- app/                # App Router pages
|   |-- components/         # Shared UI
|   |-- context/            # Client-side state/context
|   |-- lib/                # Utilities and API helpers
|   `-- styles/
|-- migrations/             # PostgreSQL schema and migration scripts
|-- deploy/                 # Production deployment assets
`-- tests/                  # Integration tests
```

## Backend architecture

The backend is organized around a layered design:

- `domain` contains the core business entities and contracts.
- `application` coordinates use cases such as booking creation, payment initialization, and payment confirmation.
- `infrastructure` provides PostgreSQL repositories, JWT handling, blockchain gateways, observability, and the reconciliation worker.
- `interfaces` exposes the HTTP API through Actix handlers and middleware.

## Core API routes

The Actix server exposes routes under `/api`.

- `GET /api/health` for health checks
- `POST /api/users/signup` to create an account
- `POST /api/users/login` to authenticate
- `POST /api/bookings` to create a booking
- `GET /api/bookings/latest` to fetch the latest booking for the current user
- `POST /api/payments/init` to initialize a crypto payment
- `POST /api/payments/confirm` to confirm a payment with a transaction hash
- `GET /api/transactions/{user_id}` to fetch user transactions

Some routes are public, while protected routes require a valid JWT.

## Environment setup

Create a local `.env` based on the project example. Important variables include:

```env
APP_ENV=development
RUN_MIGRATIONS=true
DATABASE_URL=postgresql://postgres:postgres@localhost/aeroledger
JWT_SECRET=replace-with-a-secure-secret
HOST=127.0.0.1
PORT=8080
FRONTEND_URL=http://localhost:3000
RECONCILIATION_INTERVAL_SECONDS=60
```

You will also need wallet addresses and chain API settings for payment verification features.

For production-style values, review `.env.production` and the deployment notes in [deploy/README.md](/e:/areo_ledger/deploy/README.md).

## Running the backend

1. Make sure PostgreSQL is running and `DATABASE_URL` points to an available database.
2. Start the API server from the repository root:

```bash
cargo run
```

The backend listens on `http://127.0.0.1:8080` by default.

## Running the frontend

Start the Next.js app from `frontend-luxury/`:

```bash
cd frontend-luxury
npm install
npm run dev
```

The frontend typically runs on `http://localhost:3000`.

## Database and migrations

The project uses PostgreSQL, not SQLite. Migration files live in `migrations/`.

Development flow:

- Use `RUN_MIGRATIONS=true` if you want the app to apply migrations on startup.
- For production, keep `RUN_MIGRATIONS=false` and run migrations as a separate deployment step.

## Tests

Integration coverage includes a PostgreSQL-backed test scaffold in `tests/integration_postgres.rs`.

Run the ignored integration tests with:

```bash
cargo test -- --ignored
```

Set `TEST_DATABASE_URL` before running those tests.

## Production notes

Production deployment assets live in `deploy/`, including:

- `docker-compose.prod.yml`
- `nginx.conf`
- `backup.ps1`

The deployment guide is in [deploy/README.md](/e:/areo_ledger/deploy/README.md).
