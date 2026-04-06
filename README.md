# AeroLedger

AeroLedger is a crypto-native commerce platform for private aviation and luxury marketplace purchases. It combines a Rust backend with a React + Vite frontend to support browsing, cart management, protected checkout, crypto payment initialization, transaction-hash confirmation, payment-proof uploads, and transaction tracking across multiple blockchains.

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
|-- deploy/                 # Deployment assets
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

<<<<<<< HEAD
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


## Running the Backend
=======

>>>>>>> 8cf27dd8bb519e54a89d3f05ebfaaebf1b010de0

Make sure PostgreSQL is available and `DATABASE_URL` is valid, then run:

```bash
cargo run
```

Default backend address:

```text
http://127.0.0.1:8080
```

## Running the Frontend

From the `Frontend/` directory:

```bash
cd Frontend
npm install
npm run dev
```

The frontend uses Vite. Set `VITE_API_BASE_URL` if you need a different backend base URL.

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

Production deployment assets are stored in `deploy/`.

Useful files include:

- `deploy/docker-compose.prod.yml`
- `deploy/nginx.conf`
- `Dockerfile`
- `.env.production`

## Current Status

The repository currently supports:

- backend checkout for flights and marketplace items
- transaction hash confirmation
- payment-proof uploads wired from frontend to backend
- transaction history and order tracking

If you change env values such as `JWT_SECRET`, restart the backend and sign in again so the frontend session stays valid.
