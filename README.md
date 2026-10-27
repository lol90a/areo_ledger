# AeroLedger

A crypto-native commerce platform for private aviation charter and tokenized
luxury assets. A buyer browses a catalogue, books a flight or an asset share, and
pays in cryptocurrency; the backend prices the order itself, verifies the transfer
on-chain, and only then marks the order settled.

The interesting part of this system is not the checkout UI — it is the rule that
**the client never decides what it pays and never decides whether it paid**. Every
amount is computed server-side from an authoritative catalogue price and a recorded
exchange rate, and every settlement is confirmed by reading the chain, given nothing
but a transaction hash.

---

## How it works

A purchase moves through five server-controlled steps. The client supplies a
booking id and, at the end, a transaction hash. Nothing else it sends affects the
amount.

| Step | Endpoint | What the server does |
|---|---|---|
| 1. Book | `POST /api/bookings` | Reads the catalogue price for the flight or asset. The request body carries no price. |
| 2. Quote | `POST /api/payments/quote` | Converts to crypto using a live rate it fetches and records, with the markup and service fee applied. Returns an expiring quote. |
| 3. Approve | `POST /api/payments/quote/accept` | The client echoes back the exact figures it displayed. If either has moved, the server refuses and returns the new quote rather than charging an amount the user never saw. |
| 4. Initialize | `POST /api/payments/init` | Returns the receiving address, the exact amount in base units, and the network. |
| 5. Confirm | `POST /api/payments/confirm` | Looks the hash up on-chain at finalized commitment, checks the recipient and the amount, and settles only on a match. |

Properties this buys:

- **Amounts are integers in the token's smallest unit** (wei, lamports, satoshis)
  end to end. No float touches a balance comparison.
- **A price change between browsing and paying is surfaced**, not silently applied.
- **A transaction hash settles at most one booking.** Replay is rejected at the
  database level by a unique constraint, not only in application code.
- **Underpaying, paying the wrong address, or inventing a hash all fail**, and the
  error distinguishes *has not settled yet* from *will never settle* — so the UI
  never tells a user to wait for something that cannot happen.
- **The server decides the network.** Chain id (EVM) and RPC endpoint (Solana) are
  served from configuration, so the wallet cannot sign on a different cluster from
  the one the verifier reads. Startup refuses a configuration that mixes mainnet
  and test networks.

---

## Architecture

The backend follows Clean Architecture, and the dependency rule is enforced rather
than aspired to: `domain/` and `application/` import nothing from the outer layers
and contain no framework types.

```
interfaces/      HTTP handlers, middleware, error mapping      (Actix)
     |
application/     Use cases: create_quote, accept_quote,        (no framework)
     |           init_payment, confirm_payment, ...
domain/          Entities, value objects, repository and       (no framework)
     |           gateway traits — the ports
infrastructure/  Postgres repositories, blockchain gateways,
                 price oracle, config, workers                 (implements the ports)
```

Use cases are generic over the repository and gateway traits, so the direction of
dependency points inward at compile time. Each use case exposes exactly one
`execute`. Traits are kept small — most have one or two methods — so an
implementation is never forced to stub behaviour it does not have.

Two value objects carry most of the safety: `Money` (integer cents, no float
constructor) and `Network` (which realm an endpoint belongs to, used to reject a
deployment pointed at two different networks at once).

### Repository layout

```
src/                        Rust backend
  domain/                   entities, value objects, ports, repository traits
  application/              use cases and DTOs
  infrastructure/           Postgres, blockchain gateways, pricing, config, workers
  interfaces/http/          handlers, middleware, auth
  bin/migrate.rs            standalone migration runner
Frontend/                   React 18 + TypeScript + Vite
  src/app/pages/            routed pages
  src/app/components/       shared components (ui/ is shadcn/ui)
  src/app/lib/              API client, wallet access, cart, presentation helpers
contracts/solana/           Anchor escrow program + TypeScript integration suite
contracts/ethereum/         Solidity escrow contract + Hardhat suite
migrations/                 complete_schema.sql, applied at startup
tests/                      backend integration tests against real Postgres
```

### Stack

**Backend** — Rust 2021, Actix Web 4, `deadpool-postgres`, JWT auth, `reqwest`
blockchain gateways for EVM (Etherscan V2), Solana JSON-RPC and Bitcoin.

**Frontend** — React 18, TypeScript, Vite, React Router, Tailwind, Radix
primitives via shadcn/ui, Motion. Wallets are reached through the injected
EIP-1193 and Solana provider interfaces directly, with no wallet-adapter
dependency tree.

**Contracts** — Anchor 0.29 (Solana) and Solidity 0.8.24 with OpenZeppelin
(EVM). Both implement a buyer/seller/treasury escrow with server-signed price
quotes.

---

## Running it locally

Three processes, in this order.

### 1. Database

```bash
docker compose up -d postgres
```

Dev Postgres, published on host port **5435** so it does not collide with another
instance. The schema is applied by the backend at startup.

### 2. Backend

```bash
cp .env.example .env     # then set DATABASE_URL and a real 32+ character JWT_SECRET
cargo run --bin areo_ledger
```

`--bin areo_ledger` is required: the crate ships two binaries, the server and the
standalone `migrate` runner, so a bare `cargo run` cannot choose.

Migrations run at startup while `RUN_MIGRATIONS=true`. That combination is rejected
when `APP_ENV=production`, where migrations are a deployment step: `cargo run --bin
migrate`.

### 3. Frontend

```bash
cd Frontend
echo "VITE_API_BASE_URL=http://127.0.0.1:8071" > .env
npm install
npm run dev
```

Vite serves on **http://127.0.0.1:3001** with `strictPort`, so it fails loudly
rather than moving to another port and breaking CORS.

### Three values that must agree

A mismatch here is the usual cause of a dead checkout with CORS errors in the
console.

| Value | Set in | Must equal |
|---|---|---|
| `PORT` | root `.env` | the port in `VITE_API_BASE_URL` |
| `VITE_API_BASE_URL` | `Frontend/.env` (not committed) | `http://HOST:PORT` of the backend |
| `FRONTEND_URL` | root `.env` | the Vite origin — it is the CORS allow-list entry |

Check the stack is up:

```bash
curl http://127.0.0.1:8071/api/health
```

---

## Testing

Five suites. CI runs all of them on every pull request.

```bash
# Backend unit tests
cargo test --lib

# Backend integration tests against a real Postgres (not a mock)
docker run -d --name pg-test -e POSTGRES_PASSWORD=postgres -p 5433:5432 postgres:16-alpine
TEST_DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5433/postgres \
  cargo test --test integration_postgres -- --ignored --test-threads=1

# Solana program unit tests
cd contracts/solana && cargo test --lib

# Solana integration tests against a local validator
cd contracts/solana && yarn install --frozen-lockfile && anchor test

# EVM contract tests
cd contracts/ethereum && npm ci && npx hardhat test

# Frontend types and production build
cd Frontend && npx tsc --noEmit && npx vite build
```

The Anchor suite needs an exact toolchain: **Anchor 0.29.0**, **Solana 1.18.26**
and **Node >= 20.18**. Earlier Solana releases ship a rustc too old for the pinned
`solana-program`, and Node 18 is refused by `@solana/codecs-*`. `Cargo.lock` is
committed in lockfile v3 format and `yarn.lock` is committed, so neither set of
dependencies can drift.

The Solana program id in the repository is the Anchor template placeholder. Before
any deploy, generate a real keypair and sync the id — `Anchor.toml` carries the
exact command, and `anchor keys sync` must not be used (it updates only one of the
three `[programs.*]` entries per run).

---

## Deployment

```bash
cp .env.production.example .env.production   # fill in real values; never commit it
docker compose --env-file .env.production -f docker-compose.prod.yml up -d --build
```

`Dockerfile` builds the backend, `Frontend/Dockerfile` and `Frontend/nginx.conf`
build and serve the frontend with an API reverse proxy.

Configuration is validated at startup, and production fails closed: it rejects a
weak or placeholder `JWT_SECRET`, a localhost `FRONTEND_URL`, startup migrations,
manual payment verification, a malformed wallet address for any configured chain,
and any mix of mainnet and test networks.

---

## Status

Stated plainly, because a reader would otherwise reasonably assume otherwise:

- **The live payment path is direct payment with on-chain verification.** Both
  escrow contracts are implemented and tested, but they are **not wired into the
  backend** — nothing in `src/` calls them yet.
- **`infrastructure/blockchain/retry.rs`** implements a retrying gateway with a
  circuit breaker. It is tested but not currently constructed in `main.rs`; the
  confirm endpoint is retried by the client instead.
- **SPL-token settlement on Solana is deliberately unimplemented.** Only native SOL
  is verified there. A half-right reading of a token transfer is worse than
  refusing the route.
- The frontend has no test suite; it is covered by type-checking and a production
  build in CI.
