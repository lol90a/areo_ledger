# AeroLedger — Architecture

A marketplace for private-jet charter and tokenised luxury assets, settled in
cryptocurrency. This document describes what the system actually is, where the
trust boundaries fall, and — importantly — which parts are wired together today and
which are not.

---

## 1. System overview

```
┌──────────────────────────────────────────────────────────────────────────┐
│  Browser (React 18 + Vite + TypeScript)                                  │
│    pages: Landing, Signin/Signup, Dashboard, Flights, Marketplace,       │
│           AssetDetails, Cart, Checkout, Transactions, OrderTracking,     │
│           Profile                                                        │
│    lib/api.ts        fetch wrapper, bearer token, 401 -> session clear   │
│    lib/auth.ts       session in localStorage, client-side exp check      │
│    lib/backend.ts    typed API client + CheckoutPhase vocabulary         │
│    lib/wallet.ts     EIP-1193 + Solana provider, bigint balance checks   │
│    components/CheckoutQuotePanel.tsx   quote -> balance -> approve       │
└───────────────┬──────────────────────────────────────────────────────────┘
                │ HTTPS, JSON, Bearer JWT
┌───────────────▼──────────────────────────────────────────────────────────┐
│  Rust backend (Actix Web 4, Tokio) — clean architecture                   │
│                                                                           │
│   interfaces/http   handlers, auth middleware, CORS, rate limiting        │
│        │                                                                  │
│   application       use cases: CreateBooking, CreateQuote, AcceptQuote,   │
│        │            InitPayment, ConfirmPayment, AuthenticateUser, ...    │
│        │                                                                  │
│   domain            entities (Booking, PaymentQuote, Payment, User),      │
│        │            value objects (Money, CryptoToken, Chain, TxHash),    │
│        │            repository traits, port traits. Depends on nothing.   │
│        │                                                                  │
│   infrastructure    Postgres adapters, chain gateways, price oracle,      │
│                     JWT service, reconciliation worker                    │
└───────┬───────────────────────────┬───────────────────────────────────────┘
        │                           │
┌───────▼─────────┐     ┌───────────▼───────────────────────────────────────┐
│  PostgreSQL     │     │  Chain providers                                  │
│  users          │     │   Etherscan / BscScan   (receipts, ERC-20 logs)   │
│  flights,assets │     │   Solana JSON-RPC       (finalized getTransaction)│
│  bookings       │     │   Blockstream           (UTXO outputs)            │
│  payment_quotes │     │   CoinGecko             (USD rates)               │
│  payments       │     └───────────────────────────────────────────────────┘
│  transactions   │
│  audit_log      │     ┌───────────────────────────────────────────────────┐
└─────────────────┘     │  Smart contracts (built + tested, NOT yet wired)  │
                        │   Solana: aeroledger_escrow (Anchor 0.29)         │
                        │   EVM:    AeroLedgerEscrow.sol (solc 0.8.24)      │
                        └───────────────────────────────────────────────────┘
```

---

## 2. The commercial model

AeroLedger is a **marketplace intermediary**, not the merchant of record. Every
sale has three economic parties, and both escrow contracts now encode that
explicitly.

| Party | Who they are | What they get | Where they are recorded |
|---|---|---|---|
| **Buyer** | The customer | The service/asset; refunds | `Order.buyer` |
| **Seller / merchant** | Charter operator, asset owner | `amount − fee` | `Order.seller`, paid at `Order.payout_wallet` |
| **Platform treasury** | AeroLedger revenue | `fee` | `Order.treasury` |
| **Admin / owner** | Governance | *Nothing from a sale* | `Config.admin` / `owner()` |
| **Quote authority** | The backend's signing key | — | `Config.quote_authority` / `quoteSigner` |

The original contracts had no seller at all: funds went to the administrator, so
"admin" and "merchant" were the same entity by construction. That is the design
error the review flagged and it is fixed in both contracts — see
`SECURITY_AUDIT.md` SOL-01 and EVM-01.

Two properties are worth stating on their own because they are what make the model
hold up:

- **Payout destinations are snapshotted at order creation**, from
  governance-controlled state. Rotating a merchant's payout wallet, or transferring
  ownership of the contract, does not retarget money already in escrow.
- **The admin has no path to an arbitrary destination.** It can complete (→ the
  registered seller wallet) or refund (→ the recorded buyer). Those are the only two
  exits, and both are pinned by account constraints rather than by a parameter.

---

## 3. Trust boundaries

| Boundary | Everything on this side is | Enforcement |
|---|---|---|
| Browser → API | **Untrusted.** Identifiers and selections only. | Every price is server-derived; `InitPaymentInput` has no amount field at all |
| API → chain provider | **Semi-trusted.** Can be wrong or down. | Fail closed: no provider answer ⇒ no settlement; the worker retries |
| Chain provider → settlement | Transaction hashes are **user input** | Receipt status, mined block, recipient, amount and confirmations all re-checked |
| Backend → contract | Backend is the quote authority | Ed25519 / EIP-712 signature, scoped and time-bounded |
| Admin → funds | **Constrained.** | Destinations come from recorded state, never from a parameter |
| Frontend balance check | **Advisory only.** | Stated in the UI; the chain and the backend enforce |

The single most important line: **the client never supplies a price, and never
supplies an amount.** `POST /api/payments/init` takes exactly one field,
`booking_id`. The amount is copied from a quote the user approved.

---

## 4. Price-quote lifecycle

This is the answer to "what happens if the price changes while the user is booking?"

```
   catalogue price                        oracle rate (source + observed_at)
          │                                         │
          └──────────────┬──────────────────────────┘
                         ▼
    POST /api/payments/quote     ── supersedes any previous active quote
                         │           (same DB transaction + partial unique index)
                         ▼
             ┌───────────────────────┐
             │  status = active      │  TTL 600s, countdown shown in the UI
             │  total_cents          │  price_changed + previous_total_cents
             │  crypto_base_units    │  rate_source, rate_observed_at
             └───────────┬───────────┘
                         │
      user reviews ──────┤
                         │
    POST /api/payments/quote/accept
      { approved_total_cents, approved_crypto_base_units }   ← echoed back exactly
                         │
         ┌───────────────┴────────────────┐
         │ figures match        mismatch  │
         ▼                                ▼
  status = accepted                 409 Conflict
  AMOUNT LOCKED                     + fresh quote issued
                         │          + UI shows old vs new, asks again
                         ▼
             POST /api/payments/init        ← carries no amount
                         │
                  Payment::from_quote(...)  ← the ONLY constructor
                         ▼
                  payments row, status = pending
```

**Rules, each enforced in code, not convention:**

1. The server owns the price. Quotes are re-derived from the catalogue every time.
2. Quotes expire (`QUOTE_TTL_SECONDS` = 600). Expired ⇒ not acceptable.
3. Acceptance echoes both the fiat total and the token base units. Either differing
   ⇒ 409 and the user must approve again.
4. Acceptance locks the base-unit amount permanently for that payment.
5. One live quote per booking — issuing supersedes, backed by
   `idx_payment_quotes_one_active`.
6. `Payment::from_quote` refuses anything not `Accepted`. There is no constructor
   that takes a caller-supplied amount.
7. A re-price after acceptance voids it (`InitPayment` re-checks the totals agree).
8. A rate older than `MAX_RATE_AGE_SECONDS` (300) cannot be quoted from.

**Price-lock policy.** An unaccepted quote lives 10 minutes. Once accepted, the
token amount is fixed for that payment indefinitely: the platform carries the FX
risk from acceptance onward, deliberately, because the alternative is telling a
customer whose transaction is already on chain that they now owe more. This is an
accepted, documented exposure, not an oversight.

---

## 5. Two payment paths — and which one is live

**This is the most important structural caveat in the codebase, and it is stated
plainly rather than papered over.**

### 5a. Direct payment — IMPLEMENTED END TO END, TESTED

```
Browser                 Backend                     Chain            Database
   │ create booking ──────▶ catalogue price ───────────────────────▶ bookings
   │ request quote  ──────▶ catalogue + oracle ────────────────────▶ payment_quotes
   │ ◀── total, token amount, expiry, price_changed
   │ connect wallet, read balance (bigint, base units)
   │ accept quote ────────▶ echo check ───────────────────────────▶ status=accepted
   │ init payment  ──────▶ Payment::from_quote ───────────────────▶ payments
   │ user sends funds from their own wallet ──▶ platform address
   │ submit tx hash ─────▶ gateway verify:
   │                        receipt status == 0x1
   │                        mined block present
   │                        recipient matches
   │                        amount >= locked base units (exact-or-more)
   │                        confirmations >= chain minimum
   │                      ├─ replay check (hash not used elsewhere)
   │                      └─ settlement transaction ───────────────▶ payments
   │ ◀── verified                                                    bookings
   │                      reconciliation worker retries pending      transactions
```

This is what runs today. It is covered by 10 integration tests against a real
PostgreSQL instance and 46 backend unit tests.

### 5b. On-chain escrow — BUILT AND TESTED, NOT WIRED TO THE BACKEND

Both escrow contracts are complete, hardened and tested (52 Hardhat tests executed;
14 Solana unit tests executed; a 30-case Anchor suite written but not executed —
see `SECURITY_AUDIT.md` §7). **No backend code calls them, and no frontend code
builds their instructions.** They exist in the repository as a correct
implementation of the escrow model, not as the live settlement path.

What a real integration would require:

1. The backend signs the quote in the contract's format — Ed25519 over the
   canonical message for Solana, EIP-712 `Quote` for EVM — and returns the signature
   alongside the quote. The signing key becomes `quote_authority` / `quoteSigner`.
2. The frontend builds the transaction: for Solana, the Ed25519 pre-instruction plus
   `create_order`; for EVM, `createOrder`/`createOrderERC20` with the `QuoteInput`
   struct. The user signs in their wallet.
3. Merchant onboarding writes `register_merchant` / `setMerchant` on chain, so the
   platform's seller records and the contract's merchant registry agree.
4. The gateways parse program/contract **events** (`OrderCreated`, `OrderCompleted`)
   rather than raw balance deltas, and the backend's order state machine mirrors the
   on-chain one.
5. Fulfilment drives `confirm_order` (seller) and `complete_order`, with the refund
   and timeout paths exposed in the UI.

Until that is done, the honest description is: **the website settles by direct
payment; the escrow contracts are a tested implementation awaiting integration.**

### Which path per chain, if 5b were enabled

| Chain | Direct payment | Escrow contract | Notes |
|---|---|---|---|
| Ethereum | live | available (ETH + allow-listed ERC-20) | |
| BSC | live | available (same contract) | |
| Solana | live (native SOL only) | available (allow-listed SPL mints) | SPL settlement via direct payment is explicitly rejected, see BE-10 |
| Bitcoin | live | none | No contract platform; direct payment only, by design |

---

## 6. Backend layering

Dependencies point inward only.

**`domain/`** — no framework, no I/O.
- `entities/`: `Booking`, `PaymentQuote`, `Payment`, `User`, `Transaction`. State
  machines live here (`PaymentQuote::accept` owns the price-change rules).
- `value_objects/`: `Money` (integer cents, `apply_bps`, no `f64` anywhere),
  `CryptoToken`/`Chain` (decimals, ERC-20 contracts, `resolve_chain_token` as the
  single source of truth), `TxHash`, `WalletAddress`, `IdempotencyKey`.
- `repositories/` and `ports/`: traits only.

**`application/`** — use cases over those traits. Each is a struct generic over its
repositories, which is what makes them unit-testable with stubs and no database.

**`infrastructure/`** — the adapters: `persistence/postgres/*`,
`blockchain/{evm,solana,bitcoin,router}_gateway`, `pricing/market_price_oracle`,
`auth/jwt_service`, `workers/payment_reconciliation`, `config`.

**`interfaces/http/`** — Actix handlers, auth middleware, error mapping, CORS, rate
limiting. Handlers resolve the actor from the JWT and never trust a `user_id` in a
body.

### Money, and why there are three representations

| Representation | Type | Used for |
|---|---|---|
| Fiat minor units | `Money` (`i64` cents + currency) | all fiat arithmetic |
| Whole token units | `Decimal` | display only |
| Token base units | `u128` / `NUMERIC(40,0)` | **every on-chain comparison** |

The conversion happens exactly once, at quote issue
(`usd_to_crypto_amount` → `crypto_base_units`), rounded **up** to the token's
precision, and the integer result is what gets locked and later compared. A USD
cent value is never compared with a token amount; the types make it awkward to try.

---

## 7. Persistence

| Table | Role | Key invariants |
|---|---|---|
| `users` | accounts | unique email; Argon2 hashes |
| `flights`, `assets` | **authoritative catalogue** | the only source of a base price |
| `bookings` | what is being bought | `flight_id` xor `asset_id`; pricing re-written on re-quote |
| `payment_quotes` | priced, expiring offers | `idx_payment_quotes_one_active` and `idx_payment_quotes_one_accepted` (partial unique); `CHECK (expires_at > created_at)`; `CHECK (crypto_base_units > 0)` |
| `payments` | settlement obligation | one per booking (partial unique); `tx_hash` globally unique (partial unique) — the replay backstop |
| `transactions` | user-visible ledger | written only inside the settlement transaction |
| `audit_log` | who did what | quote issued/accepted, payment initialised/confirmed/reconciled |

**Settlement is one transaction.** `PgSettlementRepository::confirm_payment`:

```sql
SELECT ... FROM payments p JOIN bookings b ON b.id = p.booking_id
WHERE p.booking_id = $1 FOR UPDATE OF p, b;     -- serialise concurrent confirms
-- status must be 'pending'; 'confirmed' returns AlreadyConfirmed (idempotent)
-- reject a tx_hash already attached to a different booking
UPDATE payments ... WHERE booking_id = $3 AND status = 'pending';  -- 0 rows ⇒ lost race
UPDATE bookings ...;
INSERT INTO transactions ...;
COMMIT;
```

Three layers guard replay: the use case, this transaction, and the unique index on
`payments(tx_hash)`. Executed tests cover a hash reused across bookings and a triple
confirm producing exactly one ledger row.

---

## 8. Transaction verification

| Chain | Source | Checks |
|---|---|---|
| Ethereum / BSC, native | `eth_getTransactionReceipt` **then** `eth_getTransactionByHash` | receipt `status == 0x1`; mined block present (not `null`/`0x0`); `to` matches; `value >= locked`; confirmations ≥ 12 (ETH) / 15 (BSC) |
| Ethereum / BSC, ERC-20 | receipt logs | receipt status; log `address` == the configured token contract; `topic0` == `Transfer`; `topics[2]` == platform address; summed `data >= locked` |
| Solana | `getTransaction` at `commitment: finalized` | RPC error surfaced, not swallowed; `meta.err` null; recipient located **by address** in `accountKeys`; its own pre/post balance delta `>= locked`; SPL-token requests rejected outright |
| Bitcoin | Blockstream `/tx/{id}` | sum `vout` paying the platform address `>= locked`; confirmed; depth ≥ 3 |

Uniform rules: **exact-or-more**, never a tolerance; the recipient is matched by
address, never by position; a provider failure yields *pending*, never *verified*.

`MultiChainGateway` routes by chain. In `manual` mode it returns a synthetic
success — and `AppConfig::validate` **refuses to start** with
`PAYMENT_VERIFICATION_MODE=manual` outside `APP_ENV=development`, while the
reconciliation worker refuses to run in that mode so it cannot rubber-stamp unpaid
bookings.

---

## 9. Reconciliation worker

A Tokio task that polls pending payments carrying a submitted hash
(`LIMIT 200`, oldest first) and re-runs the same verification and the same
settlement transaction as the synchronous path.

- Disabled entirely in `manual` mode.
- Per-payment error isolation: one flaky provider call cannot starve the batch.
- Idempotent by construction — it calls `confirm_payment`, which returns
  `AlreadyConfirmed` rather than writing twice.
- Multi-replica safe: `FOR UPDATE` plus the conditional `UPDATE ... WHERE status =
  'pending'` means concurrent replicas serialise and only one settles.
- Writes an audit row only on a real transition.

*Known gap:* retries are unbounded and un-backed-off, and there is no dead-letter
state for a payment that will never verify. Recorded in `SECURITY_AUDIT.md` §8.

---

## 10. Solana program

`contracts/solana/programs/aeroledger-escrow/` — Anchor 0.29, SPL Token (not
Token-2022).

**Accounts**

| Account | Seeds | Holds |
|---|---|---|
| `Config` | `["config"]` | admin, pending_admin, quote_authority, treasury, refund_window_secs, paused |
| `SupportedMint` | `["mint", mint]` | mint allow-list entry + decimals |
| `Merchant` | `["merchant", authority]` | seller authority, payout_wallet, active |
| `Order` | `["order", keccak256(order_id)]` | buyer, seller, mint, payout_wallet, treasury, amount, fee_amount, status, created_at, refund_deadline, quote_expiry, order_id, bump |
| escrow vault | ATA of the `Order` PDA | the escrowed tokens |

`order_id` is hashed for the seed because a 36-byte UUID would exceed the 32-byte
seed limit; the full string is stored in account data for off-chain correlation.

**State machine**

```
                     create_order (buyer + signed quote)
                               │
                               ▼
                           ┌────────┐  refund_order (buyer|seller|admin)   ┌──────────┐
                           │Pending ├──────────────────────────────────────▶│ Refunded │
                           └───┬────┘  timeout_refund (buyer, after dline)  └──────────┘
             confirm_order     │                                                 ▲
             (SELLER only)     ▼                                                 │
                           ┌──────────┐  refund_order (seller|admin)             │
                           │Confirmed ├──────────────────────────────────────────┘
                           └───┬──────┘  timeout_refund (buyer, after deadline)
         complete_order        │
         (seller|admin)        ▼
                           ┌───────────┐   terminal
                           │ Completed │   no transition out
                           └───────────┘
```

Terminal states are set **before** any CPI (checks-effects-interactions), so
double-completion and double-refund are blocked by the status guard regardless of
anything a token program might do.

**Settlement maths.** `seller_amount = amount − fee_amount` goes to the ATA of
`order.payout_wallet`; the remainder of the vault (the fee, plus any unsolicited
donation into the vault) goes to the ATA of `order.treasury`. The vault therefore
always lands on exactly zero and is closed, returning its rent to the buyer. The
`Order` account itself is deliberately never closed — it is the replay guard.

**Quote attestation.** See `src/quote.rs` and `SECURITY_AUDIT.md` SOL-02. The
canonical message is
`"AEROLEDGER_ORDER_QUOTE_V1" || order_id_hash || buyer || seller || mint ||
amount_le || fee_le || expiry_le`, verified by the native Ed25519 program in the
same transaction and pinned to `current_index − 1` by instruction introspection.

---

## 11. EVM contract

`contracts/ethereum/contracts/AeroLedgerEscrow.sol` — solc 0.8.24, `evmVersion:
cancun`, OpenZeppelin 5.6 (`ReentrancyGuard`, `Pausable`, `Ownable2Step`,
`SafeERC20`, `EIP712`, `ECDSA`).

Mirrors the Solana model: merchant registry, snapshotted payout and treasury,
EIP-712 signed quotes, the same five-state machine, the same authorisation table,
the same buyer timeout exit.

EVM-specific properties:

- Native and ERC-20 escrow are **strictly separate entry points**, and
  `escrowBalance[asset]` is tracked per asset, so a payout can never exceed what was
  deposited for that asset — the "label a tiny ETH deposit as USDC and drain the
  USDC pool" class of attack is structurally impossible (and tested).
- `SafeERC20` everywhere, so USDT's non-standard no-bool `transfer` works and a
  silent-false transfer reverts.
- Native payouts use a checked low-level `.call`, not `.transfer`, so a Gnosis-Safe
  payout wallet is not bricked by the 2300-gas stipend (tested).
- `unaccountedBalance` / `sweepUnaccounted` recover misdirected tokens, capped by
  the escrow ledger so escrowed funds are out of reach.
- The EIP-712 domain binds chain id and contract address: a quote is useless on
  another chain or another deployment (tested).

---

## 12. Authentication and authorisation

- Argon2 password hashing; HS256 JWT with an `exp` claim.
- `AuthMiddleware` enforces bearer auth on everything except an explicit, boundary-
  aware public allow-list (`/api/health`, signup, login, payment options, the
  flights and assets catalogues) — `strip_prefix` plus a `/` check, so a new
  protected route under a public prefix cannot be silently exposed.
- Every booking/quote/payment handler re-resolves the actor from the token and
  compares it with the resource owner. `user_id` in a request body is ignored for
  non-admins.
- Rate limiting is per client IP, proxy-aware (`x-forwarded-for` → `x-real-ip` →
  peer), ~5 req/s sustained with a burst of 60.
- CORS: a strict origin allow-list; a wider set only when `APP_ENV=development`.
- Config validation refuses to boot production with a placeholder JWT secret,
  `manual` verification, startup migrations, a localhost frontend origin, a
  malformed wallet address, or a missing provider key.

---

## 13. Local development

```bash
# Database (docker-compose.yml: db/user `aeroledger`, host port 5435)
docker compose up -d postgres

cp .env.example .env            # set DATABASE_URL and a real 32+ char JWT_SECRET
cargo run --bin areo_ledger     # migrations run automatically outside production

cd Frontend && npm install && npm run dev   # Vite, strictPort, 127.0.0.1:3001
```

`--bin areo_ledger` is not optional: the crate has two binaries (the server and the
`migrate` runner), so a bare `cargo run` is ambiguous and exits without starting
anything.

The backend binds `HOST:PORT` (default `127.0.0.1:8080`; the dev `.env` sets
`PORT=8071`). Three values must agree or the browser gets CORS failures: backend
`PORT`, `VITE_API_BASE_URL` in the untracked `Frontend/.env`, and `FRONTEND_URL` in
the root `.env`, which is the CORS allow-list entry for the Vite origin.

### Test commands

```bash
# Backend
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --lib

# Backend integration (real Postgres, throwaway container on 55439)
docker run --rm -d --name aeroledger-test \
  -e POSTGRES_PASSWORD=postgres -p 55439:5432 postgres:16
TEST_DATABASE_URL=postgres://postgres:postgres@127.0.0.1:55439/postgres \
  cargo test --test integration_postgres -- --ignored --test-threads=1

# Solana program (unit tests; no validator required)
cd contracts/solana/programs/aeroledger-escrow && cargo test --lib

# Solana integration (requires the Anchor CLI — NOT run in this environment)
cd contracts/solana && yarn install && anchor keys sync && anchor test

# EVM
cd contracts/ethereum && npm install && npx hardhat test

# Frontend
cd Frontend && npx tsc --noEmit && npx vite build
```

---

## 14. Known architectural limitations

1. **The escrow contracts are not wired to the backend** (§5b). The honest
   description of the live system is "direct payment with on-chain verification".
2. **Flight availability is not reserved.** A booking records a price; it does not
   hold a seat. Payment confirmation and fulfilment are correctly separate states,
   but there is no provider integration behind the catalogue — the `flights` and
   `assets` tables are seeded data, and no real reservation is made. Nothing in the
   product should claim otherwise.
3. **Solana settles native SOL only.** SPL-token settlement is explicitly rejected
   rather than approximated.
4. **Single-key governance.** `Config.admin` / `owner()` should be a multisig with a
   timelock before real funds are handled.
5. **The quote authority is a hot key.** It should move to an HSM/KMS signer with
   per-period value caps.
6. **Bitcoin has no escrow option** and no browser balance pre-flight — there is no
   wallet standard to use.
7. **No CSP, and sessions live in `localStorage`** (`SECURITY_AUDIT.md` FE-06).
