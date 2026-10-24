# AeroLedger

AeroLedger is a crypto-native commerce platform for private aviation and luxury marketplace purchases. It combines a Rust backend with a React + Vite frontend to support browsing, cart management, protected checkout, crypto payment initialization, transaction-hash confirmation, payment-proof uploads, and transaction tracking across multiple blockchains.

> ### Security documentation
>
> This repository carries a full security review and remediation record. Read these
> before changing anything in a payment path:
>
> | Document | What it covers |
> |---|---|
> | [`SECURITY_AUDIT.md`](SECURITY_AUDIT.md) | Every finding, its root cause, the attack scenario, the fix, the test that proves it, and an explicit list of what has **not** been verified |
> | [`ARCHITECTURE.md`](ARCHITECTURE.md) | System design, trust boundaries, the price-quote lifecycle, and which of the two payment paths is actually live |
> | [`INTERVIEW_PREPARATION.md`](INTERVIEW_PREPARATION.md) | Walkthrough of each change: problem, attack, fix, why it works, evidence |
> | [`WALKTHROUGH.md`](WALKTHROUGH.md) | How to run it, a rehearsed demo script, and a full-stack tour of the backend, the TypeScript frontend and both contracts |
>
> **Two things to know up front.** The on-chain escrow contracts are built and
> tested but **not yet wired into the backend** — the live flow is direct payment
> with on-chain verification (`ARCHITECTURE.md` §5). And credentials committed to
> this repository's history still require rotation (`SECURITY_AUDIT.md` OPS-01).

## Highlights

- Flight booking checkout and marketplace asset checkout
- Crypto payment flows for BTC, ETH, USDT, USDC, SOL, and BNB
- Booking creation, payment initialization, hash submission, and tracking
- Payment-proof image upload connected to the backend
- JWT-based authentication for protected routes
- Background reconciliation for pending payments
- PostgreSQL-backed persistence with startup migrations

## Tech Stack

### Backend

- Rust 2021
- Actix Web
- Actix CORS
- Actix Multipart
- PostgreSQL
- Deadpool Postgres
- JWT authentication
- Reqwest-based blockchain integrations for EVM, Bitcoin, and Solana

### Frontend

- React 18
- TypeScript
- Vite
- React Router
- Tailwind CSS
- Motion
- Radix UI primitives

## Repository Layout

```text
.
|-- src/                    # Rust backend source
|   |-- application/        # Use cases and DTOs
|   |-- domain/             # Entities, ports, repositories, value objects
|   |-- infrastructure/     # Config, DB, blockchain, persistence, workers
|   |-- interfaces/         # HTTP handlers and middleware
|   |-- lib.rs
|   `-- main.rs
|-- Frontend/               # React + Vite frontend
|   |-- src/app/            # Pages, routes, UI, helpers, local data
|   `-- package.json
|-- migrations/             # PostgreSQL schema and migration SQL
|-- Dockerfile              # Backend image
|-- docker-compose.prod.yml # Production stack (backend, migrate, frontend, postgres)
|-- Frontend/nginx.conf     # Frontend nginx config / API reverse proxy
|-- tests/                  # Test scaffolding
|-- .env.example
|-- .env.production
`-- README.md
```

## Backend Architecture

The backend follows a layered structure:

- `domain` contains business entities and contracts.
- `application` contains use cases such as booking creation, payment initialization, and payment confirmation.
- `infrastructure` contains PostgreSQL repositories, blockchain gateways, auth services, config, and workers.
- `interfaces` exposes the HTTP API through Actix handlers and middleware.

## Core API Routes

All API routes are served under `/api`.

- `GET /api/health`
- `POST /api/users/signup`
- `POST /api/users/login`
- `POST /api/bookings`
- `GET /api/bookings/latest`
- `GET /api/payments/options`
- `POST /api/payments/init`
- `POST /api/payments/confirm`
- `POST /api/payments/proof/{booking_id}`
- `GET /api/transactions/{user_id}`

Protected routes require a valid JWT.

## Checkout Flow

The current checkout flow supports both flight items and marketplace assets:

1. Sign in.
2. Add a flight or marketplace item to the cart.
3. Start checkout and provide client details.
4. Create a booking on the backend.
5. Initialize crypto payment instructions.
6. Submit a blockchain transaction hash.
7. Optionally upload a payment screenshot as proof.
8. Track status from initialized to awaiting to confirmed.

## Running it locally

Three processes, in this order: database, backend, frontend.

### 1. Database

`docker-compose.yml` defines the dev Postgres (database `aeroledger`, user
`aeroledger`, published on host port **5435** so it does not collide with any other
Postgres you are running):

```bash
docker compose up -d postgres
```

### 2. Backend

```bash
cp .env.example .env     # first time only; set DATABASE_URL and a real 32+ char JWT_SECRET
cargo run --bin areo_ledger
```

`--bin areo_ledger` is required: the crate ships **two** binaries, the server and
the standalone `migrate` runner, so a bare `cargo run` cannot pick one and exits.

Migrations run automatically at startup while `RUN_MIGRATIONS=true` (development
only — `validate()` rejects that combination when `APP_ENV=production`). To run them
as a separate deployment step instead, use `cargo run --bin migrate`.

The server binds `HOST:PORT`, which default to `127.0.0.1:8080`. The committed dev
`.env` overrides `PORT=8071`.

### 3. Frontend

```bash
cd Frontend
npm install
npm run dev
```

Vite serves on **`http://127.0.0.1:3001`** (`strictPort: true` — it fails rather
than silently moving to another port).

### Keeping the three in agreement

Three values have to line up, and a mismatch is the usual cause of a dead
checkout with CORS errors in the console:

| Value | Set in | Must equal |
|---|---|---|
| `PORT` | root `.env` | the port in `VITE_API_BASE_URL` |
| `VITE_API_BASE_URL` | `Frontend/.env` | `http://HOST:PORT` of the backend |
| `FRONTEND_URL` | root `.env` | the Vite origin (`http://127.0.0.1:3001`) — it is the CORS allow-list entry |

`Frontend/.env` is **not** committed (it is machine-specific), and the fallback
baked into `Frontend/src/app/lib/api.ts` is `http://127.0.0.1:8081`. On a fresh
clone you must create it:

```bash
echo "VITE_API_BASE_URL=http://127.0.0.1:8071" > Frontend/.env
```

### Verify the stack is up

```bash
curl http://127.0.0.1:8071/api/health
```

```json
{"service":"AeroLedger","status":"ok","version":"1.0.0"}
```

## Migrations

The backend migration runner executes `migrations/complete_schema.sql` on startup when `RUN_MIGRATIONS=true`.

That schema includes:

- users
- flights
- bookings
- payments
- transactions
- audit log
- supporting marketplace and portfolio tables

## Payment Proof Uploads

Payment-proof images uploaded from checkout are stored by the backend under:

```text
uploads/payment_proofs/<booking_id>/
```

The payment row also stores proof metadata such as path, content type, and upload timestamp.

## Verification Commands

Backend:

```bash
cargo check
```

Frontend:

```bash
cd Frontend
npm run build
```

## Deployment Notes

Production deployment assets live at the repository root and under `Frontend/`.

Useful files include:

- `./docker-compose.prod.yml`
- `Frontend/nginx.conf`
- `Dockerfile`
- `.env.production` (create from `.env.production.example`; never commit real secrets)

To launch the production stack, create `.env.production` from `.env.production.example`,
fill in real values, then run:

```bash
docker compose --env-file .env.production -f docker-compose.prod.yml up -d --build
```

## Current Status

The repository currently supports:

- backend checkout for flights and marketplace items
- transaction hash confirmation
- payment-proof uploads wired from frontend to backend
- transaction history and order tracking

If you change env values such as `JWT_SECRET`, restart the backend and sign in again so the frontend session stays valid.
