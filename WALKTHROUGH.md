# AeroLedger — Live Walkthrough & Full-Stack Interview Guide

`INTERVIEW_PREPARATION.md` is the **security story**: what was broken, what you
changed, why it holds. This document is the other half — the one you need when the
interviewer says *"show me the app"* and *"walk me through your code."*

Read these in order on the day:

1. **§1–§3 here** — get the app running, rehearse the demo once.
2. **§4–§6 here** — the three code tours (backend, frontend, contracts).
3. **`INTERVIEW_PREPARATION.md`** — the security narrative.
4. **§9 here** — the honest-limits list. Volunteer it.

---

## 1. Running it — the verified sequence

Three terminals. Run them in this order.

### Terminal 1 — database

```bash
docker compose up -d postgres
```

Starts `areo-ledger-postgres` (image `postgres:16-alpine`), database `aeroledger`,
user `aeroledger`, published on host port **5435**. 5435 rather than 5432 so it
cannot collide with another Postgres on the machine — and `DATABASE_URL` in `.env`
points at 5435, so a container on 5432 would leave you debugging a connection
refusal.

Wait for it to be healthy (the compose healthcheck is `pg_isready`, ~10s):

```bash
docker compose ps
```

### Terminal 2 — backend

```bash
cargo run --bin areo_ledger
```

> **`--bin areo_ledger` is not optional.** The crate builds **two** binaries —
> `areo_ledger` (the server) and `migrate` (the standalone migration runner). A bare
> `cargo run` cannot choose, prints *"could not determine which binary to run"*, and
> exits without starting anything. If you get that error on the day, this is why.

You are up when the log reads:

```text
INFO areo_ledger::infrastructure::db] Database migrations applied successfully.
INFO areo_ledger] AeroLedger listening on 127.0.0.1:8071
```

Confirm from another shell:

```bash
curl http://127.0.0.1:8071/api/health
```

```json
{"service":"AeroLedger","status":"ok","version":"1.0.0"}
```

### Terminal 3 — frontend

```bash
cd Frontend
npm install        # first time only
npm run dev
```

Vite serves **http://127.0.0.1:3001**. `strictPort: true` in `vite.config.ts`, so if
3001 is taken it **fails loudly** instead of drifting to 3002 and silently breaking
CORS — which is the behaviour you want, and a fine thing to say out loud.

### The three values that must agree

Almost every "the page loads but nothing works" problem is one of these three out of
step. Know this table cold:

| Value | Lives in | Must equal | Current |
|---|---|---|---|
| `PORT` | root `.env` | the port in `VITE_API_BASE_URL` | `8071` |
| `VITE_API_BASE_URL` | `Frontend/.env` | `http://HOST:PORT` of the backend | `http://127.0.0.1:8071` |
| `FRONTEND_URL` | root `.env` | the Vite origin — it **is** the CORS allow-list entry | `http://127.0.0.1:3001` |

Two traps worth naming before the interviewer finds them:

- **`Frontend/.env` is not committed** (machine-specific). The fallback compiled into
  `Frontend/src/app/lib/api.ts` is `http://127.0.0.1:8081`. On a fresh clone the
  frontend therefore calls the *wrong port* until you create the file.
- The backend's own default (`config.rs`) is `8080`. This machine uses `8071` because
  8080 and 8081 are occupied by an unrelated stack.

```bash
echo "VITE_API_BASE_URL=http://127.0.0.1:8071" > Frontend/.env
```

### Shutting down

```bash
docker compose down          # add -v to drop the volume and reseed next start
```

---

## 2. The demo script — what to click, what to say

Rehearse this once before the call. It takes about four minutes.

### Step 0 — open with the shape of the thing (20s)

> "Rust backend on 8071, React 18 + TypeScript frontend on 3001, Postgres on 5435,
> and two escrow contracts — Anchor on Solana, Solidity on EVM — which are built and
> tested but not yet wired in. The live path is direct payment with on-chain
> verification. I'll show you the checkout, because that's where all the interesting
> decisions are."

### Step 1 — landing → marketplace → cart

Click **Browse Assets**, open an asset, **Add to cart**, go to the cart.

Say what the cart actually is:

> "The cart is `localStorage`, deliberately. It holds no prices the server will
> honour — it's a list of intents. The only number that can ever be charged comes
> from a server-issued quote. If you edited the cart in devtools you'd change what
> the page displays and nothing about what you'd pay."

`Frontend/src/app/lib/commerce.ts` is the whole cart module — worth having open.

### Step 2 — sign in

Sign up, or sign in with an account you made earlier. Point at what happened:

> "`POST /api/users/signup` → bcrypt hash, a JWT comes back, and it goes into
> `localStorage` under one key. I'll come back to why that's `localStorage` and not
> an httpOnly cookie — it's a documented, accepted trade-off, not an oversight."

### Step 3 — checkout: the quote (**the centrepiece**)

Go to checkout and pick **ETH**.

> "Nothing is priced until I ask the server. `POST /api/payments/quote` re-reads the
> *authoritative* catalogue price — not the price captured when the booking was
> created, and nothing the browser sent."

Point at each figure on screen and name where it came from:

- base $9,500 — the `flights` row
- markup $950 — `MARKUP_BPS = 1000`, i.e. 10%
- service fee $475 — `SERVICE_FEE_BPS = 500`, i.e. 5%
- total **$10,925** — all integer-cent arithmetic, no floats anywhere
- the ETH figure, and **`crypto_base_units`** — the integer wei amount, the only
  number settlement ever compares
- the 10-minute countdown — `QUOTE_TTL_SECONDS = 600`

> "`crypto_base_units` is a decimal **string** in the JSON, not a number — a u128 in
> wei does not survive JavaScript's `Number`. The browser parses it to `bigint` and
> never converts it to a float."

### Step 4 — the attack demo (do this; it lands)

With the quote on screen, open devtools → Network, or just run this in a second
terminal, and try to approve a different amount than the one displayed:

```bash
curl -s -X POST http://127.0.0.1:8071/api/payments/quote/accept \
  -H 'Content-Type: application/json' -H "Authorization: Bearer $TOKEN" \
  -d '{"booking_id":"'"$BID"'","quote_id":"'"$QID"'","approved_total_cents":1,"approved_crypto_base_units":"1"}'
```

```json
{"error":"Price changed: you approved 0.01 USD but the current quote is 10925.00 USD. Review and confirm the new price."}
```

> "Acceptance is an **echo**. The client sends back the exact figures it displayed. If
> they don't match what the server issued, the server refuses rather than charging an
> amount the user never saw — and the same mechanism is what catches a stale screen
> after a genuine price change."

### Step 5 — wallet pre-flight

Connect MetaMask if you have it. If you don't, say what the code does — that is just
as good, and `Frontend/src/app/lib/wallet.ts` is short enough to show:

> "It reads the balance in base units — `eth_getBalance` for native, a raw
> `eth_call` with the `balanceOf` selector `0x70a08231` for ERC-20 — compares it as
> `bigint` against the quoted amount plus a gas buffer, and tells you the shortfall
> before you sign anything. It re-reads the chain id rather than trusting what was
> captured at connect time, and it subscribes to `accountsChanged` / `chainChanged`
> so switching wallet mid-checkout invalidates the reading."

Then immediately give the caveat, unprompted — it is the strongest thing you can say:

> "None of that is a security control. A browser can report any balance it likes.
> It's a UX pre-flight so the customer isn't told 'you're short' by a failed
> transaction. The amount owed is fixed by the server quote, and the only things that
> enforce payment are the chain and the backend's verification of the settled
> transaction."

### Step 6 — accept, init, and what "confirmed" means

Approve the quote → `POST /api/payments/init`.

> "Notice `init` carries **no amount**. Its entire body is `{booking_id}`. The server
> copies the figures from the quote the user accepted. There is no request anywhere
> in this flow where the client names a price."

Then the thing you fixed:

> "The old version showed 'Payment Successful' from a three-second `setTimeout`.
> Now `payment_verified` is a state the UI can only enter because the backend checked
> the chain. There is deliberately no state a timer can advance into — the union type
> `CheckoutPhase` in `lib/backend.ts` is the exhaustive list, and you can read it."

In local dev `PAYMENT_VERIFICATION_MODE=manual`, so a submitted hash settles without
a real chain lookup. **Say so before they ask:**

> "Locally I run in `manual` mode so I can demo without spending real ETH. `config.rs`
> refuses to boot if `manual` is set with `APP_ENV` anything other than
> `development` — that's a test, not a convention."

### Step 7 — the record

Open **Transactions**. One row: amount, status `confirmed`, the tx hash.

> "And in Postgres there's an `audit_log` row for each step — `payment.quoted`,
> `payment.quote_accepted`, `payment.initialized`, `payment.confirmed` — each with
> actor, IP and user-agent."

---

## 3. The backend-only demo (no browser)

Useful if screen-sharing the UI is awkward, and it reads well as "I know my own API."
Every call below was run against this code; the responses are real.

```bash
API=http://127.0.0.1:8071

# 1. Sign up
SIGNUP=$(curl -s -X POST $API/api/users/signup -H 'Content-Type: application/json' \
  -d '{"email":"demo@aeroledger.test","name":"Demo","password":"Str0ng-Passw0rd!"}')
TOKEN=$(echo "$SIGNUP" | sed -E 's/.*"token":"([^"]+)".*/\1/')
UID=$(echo "$SIGNUP"  | sed -E 's/.*"id":"([^"]+)".*/\1/')

# 2. Catalogue (public)
curl -s $API/api/flights
curl -s $API/api/payments/options

# 3. Book a $9,500 flight
curl -s -X POST $API/api/bookings -H 'Content-Type: application/json' \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"user_id":"'"$UID"'","flight_id":"00000000-0000-0000-0000-000000000001","payment_method":"eth"}'
# -> {"booking_id":"...","total_cents":1092500,"currency":"USD","total_price":10925.0}

# 4. Quote
curl -s -X POST $API/api/payments/quote -H 'Content-Type: application/json' \
  -H "Authorization: Bearer $TOKEN" -d '{"booking_id":"'"$BID"'","method":"eth"}'
# -> base_cents 950000 | markup 95000 | service_fee 47500 | total 1092500
#    crypto_base_units "3641666666666666667" | seconds_remaining 600

# 5a. Tampered acceptance -> refused
# 5b. Honest acceptance    -> {"status":"accepted"}
# 6.  Init (body is only {booking_id})
# 7.  Confirm with the tx hash
# 8.  GET /api/transactions/$UID
```

Three one-liners that prove the boundaries hold:

```bash
# Another user's booking
curl -s -X POST $API/api/payments/quote -H "Authorization: Bearer $OTHER_TOKEN" \
  -d '{"booking_id":"'"$BID"'","method":"eth"}' -H 'Content-Type: application/json'
# -> {"error":"Unauthorized"}

# No token
curl -s $API/api/bookings/latest
# -> {"error":"Missing authorization token"}

# Nonexistent route
curl -s $API/api/nope
# -> {"error":"Route not found"}
```

---

## 4. Backend tour (Rust)

~8,200 lines across `src/`. Actix Web 4, `tokio-postgres` + `deadpool`, no ORM.

### 4.1 The layering, and why

Hexagonal / ports-and-adapters, four directories:

```text
src/domain/          entities, value objects, repository traits, port traits
src/application/     use cases (one struct per operation) + DTOs
src/infrastructure/  Postgres repos, blockchain gateways, config, auth, workers
src/interfaces/      HTTP handlers, middleware, error mapping
```

**The rule that makes it worth the indirection:** dependencies point *inward only*.
`domain` imports nothing from `infrastructure`. Repositories are **traits defined in
`domain`** and implemented in `infrastructure`, so a use case is generic over them:

```rust
pub struct ConfirmPayment<PR, SR, BG>
where PR: PaymentRepository, SR: SettlementRepository, BG: BlockchainGateway
```

If they ask *"isn't this over-engineered for this size?"* — answer honestly and
concretely:

> "For CRUD it would be. It earns its keep in exactly one place: the payment use cases
> are the part I most need under test, and they're the part that would otherwise need
> a live chain and a database to exercise. Because `ConfirmPayment` is generic over
> three traits, `confirm_payment.rs` has five unit tests with hand-written fakes —
> including 'a transaction one wei short of the quote is rejected' — that run in
> microseconds with no I/O. That's the whole return on the abstraction, and I'd have
> skipped it for the catalogue endpoints."

### 4.2 Request lifecycle — trace one end to end

`POST /api/payments/quote`:

1. **`main.rs`** — middleware stack, applied outermost-first:
   `AuthMiddleware` → `Governor` (rate limit) → `Logger` → `Cors`.
2. **`AuthMiddleware`** (`interfaces/http/middleware/auth_middleware.rs`) — lets
   `OPTIONS` through for preflight, checks a **boundary-aware** public-path list,
   otherwise requires `Authorization: Bearer`, validates the JWT, and inserts the
   claims into request extensions.
3. **`Governor`** — ~5 req/s sustained, burst 60, keyed by a custom
   `RealIpKeyExtractor` that reads `X-Forwarded-For` → `X-Real-IP` → peer address, so
   every user behind one nginx container doesn't share a single bucket.
4. **Handler** (`payment_handler::create_quote`) — `authorize_booking_access` loads
   the booking and compares `booking.user_id` with the JWT subject (admins exempt),
   constructs the use case with concrete Postgres repos, calls it, writes an
   `audit_log` row, serialises the DTO.
5. **Use case** (`application/use_cases/create_quote.rs`) — the real logic, below.
6. **Error mapping** (`interfaces/http/error_response.rs`) — `DomainError` → status
   code. Internal errors log the detail and return a generic body.

Have an answer ready for *"why is the auth list boundary-aware?"*:

```rust
path == *p || path.strip_prefix(p).is_some_and(|rest| rest.starts_with('/'))
```

> "A bare `starts_with` on `/api/users` would make a future `/api/users-admin` public
> by accident. Matching on an exact path or a `/` boundary means a new protected route
> under a public prefix can't be silently exposed."

### 4.3 `create_quote` — the use case to know by heart

In order, and every step is a guard someone could ask about:

1. Resolve `(chain, token)` from the method string — *one* source of truth,
   `resolve_chain_token`, shared with the verifier, so the API can never advertise a
   route the verifier doesn't understand.
2. Load the booking; reject if the caller isn't the owner; reject unless status is
   `pending`.
3. Reject if a payment already exists — a settled amount is fixed.
4. Reject if a quote is **already accepted** — acceptance *is* the lock. (This also
   turns what would be a raw unique-index violation into a clear 409.)
5. **Re-read the catalogue price.** This is the answer to "what if the price changes
   while the user is booking?" If it moved, recompute the breakdown, persist it, and
   set `price_changed` with the previous total.
6. Convert USD → token base units via the oracle, rounding **up**.
7. Issue the quote. `PaymentQuote::issue` refuses if the rate is older than
   `MAX_RATE_AGE_SECONDS` (300s).
8. `quotes.issue()` supersedes any previous active quote **atomically in the
   repository**, so a stale cheaper quote can't be accepted afterwards.

### 4.4 Money — three representations, and why

A guaranteed question. The answer:

| Representation | Where | Why |
|---|---|---|
| `Money { cents: i64 }` | all fiat arithmetic | integer cents; floats cannot represent 0.1 |
| `Decimal` (`rust_decimal`) | the USD→crypto conversion only | needs fractional precision without binary float error |
| `u128` base units | everything settlement touches | wei/lamports/satoshis — exact integers, no rounding |

`PriceBreakdown::calculate` is pure integer basis-point arithmetic:
`MARKUP_BPS = 1000` (10%), `SERVICE_FEE_BPS = 500` (5%). $9,500 → $950 + $475 →
**$10,925**, and the test asserts exactly that.

> "The float in `total_price` on a couple of API responses is for display only.
> `total_cents` is the figure, and the frontend's type comment says so in words."

### 4.5 Transaction verification

`MultiChainGateway` routes to `EvmGateway` (Ethereum/BSC via Etherscan-family APIs),
`SolanaGateway` (JSON-RPC) or `BitcoinGateway`. For EVM, the two fixes worth naming:

```rust
/// Block number of a MINED transaction. `None` for a pending transaction.
fn mined_block(hex: Option<&str>) -> Option<u64> { ... }

fn confirmations_for(current: u64, tx_block: u64) -> u32 {
    if current < tx_block { return 0; }
    (current - tx_block + 1).min(u32::MAX as u64) as u32
}
```

> "A pending transaction has a null `blockNumber`. The old code defaulted that to 0,
> so `current_block - 0` was a seven-digit confirmation count — a pending transaction
> looked more confirmed than anything on the chain. And the receipt `status` field is
> now required to be `0x1`: before, a **reverted** transaction was accepted as
> payment, because the code only checked the transaction existed."

Confirmation thresholds: 12 (Ethereum), 15 (BSC). Amount check is `>=` exact, no
tolerance — the quote was already rounded up, so there's no slack to concede.

### 4.6 Settlement, idempotency, replay

`PgSettlementRepository::confirm_payment` runs one transaction that
`SELECT ... FOR UPDATE OF p, b` locks payment and booking, re-checks status, writes
payment + booking + transaction rows, and commits once. Backed by two partial unique
indexes:

```sql
CREATE UNIQUE INDEX idx_payments_booking_unique ON payments(booking_id) WHERE booking_id IS NOT NULL;
CREATE UNIQUE INDEX idx_payments_tx_hash_unique ON payments(tx_hash)    WHERE tx_hash    IS NOT NULL;
```

plus `idx_payment_quotes_one_active` and `idx_payment_quotes_one_accepted` on quotes.

> "The application checks and the database constraint are deliberately redundant. The
> application check gives a good error message; the index is what's true under
> concurrency."

### 4.7 The reconciliation worker

`PaymentReconciliationWorker` is a spawned Tokio task polling pending payments that
carry a hash, re-verifying them, settling the ones that have since confirmed. It
**disables itself in manual mode** — you'll see
`payment reconciliation disabled (manual verification mode)` in the startup log, so
have that ready if they spot it.

### 4.8 Config that fails closed

`AppConfig::validate()` exits the process on a bad config, and production is stricter:
`RUN_MIGRATIONS` must be off, `PAYMENT_VERIFICATION_MODE` must be `live`,
`FRONTEND_URL` must not be localhost, every wallet address must parse for its chain,
every API key must be present, and `JWT_SECRET` must be 32+ chars and not a known
placeholder. Seven executed tests cover it.

---

## 5. Frontend tour (TypeScript + React 18)

~5,500 lines of application code in `Frontend/src/app` (plus 48 shadcn/Radix UI
primitives in `components/ui` that you did not hand-write — **say that**, don't let
them assume you're claiming it).

Stack: React 18.3, **Vite 6**, React Router 7, Tailwind 4, Radix primitives,
`lucide-react`, `motion`, `recharts`, `sonner`.

### 5.1 The files that matter

| File | Lines | What it is |
|---|---|---|
| `lib/api.ts` | 90 | the single `fetch` wrapper — `ApiError`, 401 handling |
| `lib/backend.ts` | 317 | **every API type and call.** The one to open first |
| `lib/wallet.ts` | 382 | EIP-1193 + Solana provider, `bigint` balance checks |
| `lib/auth.ts` | 115 | session storage, client-side JWT expiry check |
| `lib/commerce.ts` | 155 | the `localStorage` cart |
| `components/CheckoutQuotePanel.tsx` | 586 | quote → approve → wallet → init |
| `pages/Checkout.tsx` | 603 | checkout page composition |

### 5.2 The TypeScript questions they will actually ask

**"Walk me through your API layer."** — `apiRequest<T>` in `lib/api.ts`:

```ts
type RequestOptions = Omit<RequestInit, "body"> & {
  body?: BodyInit | Record<string, unknown> | unknown[];
  token?: string;
};
```

> "`Omit<RequestInit, 'body'>` because I widen `body` to accept a plain object and
> JSON-encode it, while still passing `FormData` straight through for the proof
> upload. `token` is lifted into a first-class option so auth isn't a header string
> copy-pasted at twelve call sites. Both custom fields are destructured out before
> the rest is spread into `fetch`, so they can't leak into `RequestInit`."

**"How do you narrow the error body?"**

```ts
const message =
  typeof data === "object" && data !== null && "error" in data && typeof data.error === "string"
    ? data.error
    : ...
```

> "`data` is `unknown` — `JSON.parse` returns `any` and I don't want that spreading.
> The chain is a hand-written type guard: object, non-null, has the key, key is a
> string. After those four checks TypeScript narrows `data.error` to `string` with no
> cast and no `any`."

**"Why `bigint`?"**

> "Wei amounts exceed `Number.MAX_SAFE_INTEGER`. 3641666666666666667 wei is already
> past 2^53, so as a float it's a *different number*. The server sends
> `crypto_base_units` as a **string**, `parseBaseUnits` validates `/^\d+$/` and
> converts to `bigint`, and every comparison is integer. `formatUnits` does the
> display conversion with `10n ** BigInt(decimals)` — also pure integer arithmetic.
> No float touches a balance comparison."

**"Show me a union type you're proud of."** — `CheckoutPhase`:

```ts
export type CheckoutPhase =
  | "idle" | "quote_requested" | "quote_ready" | "quote_expired"
  | "price_changed" | "quote_accepted" | "awaiting_wallet"
  | "insufficient_balance" | "wrong_network" | "transaction_submitted"
  | "pending_confirmation" | "payment_verified" | "failed";

export const CHECKOUT_PHASE_LABELS: Record<CheckoutPhase, string> = { ... };
```

> "Two things. First, every state is reached by a real event — a server response, a
> wallet result, a confirmed transaction. There is deliberately no state a timer can
> advance into, which is the bug I was fixing. Second, `Record<CheckoutPhase, string>`
> means adding a phase and forgetting its label is a **compile error**, not a blank
> string in the UI."

**"How do you handle module augmentation for wallets?"**

```ts
declare global {
  interface Window {
    ethereum?: Eip1193Provider;
    solana?: SolanaProvider;
  }
}
```

> "Both optional, so every access is guarded. I typed the two provider interfaces by
> hand rather than pulling in wagmi/viem or `@solana/wallet-adapter` — EIP-1193 is a
> stable injected standard and the checkout only needs connect and read-balance. The
> trade-off is no WalletConnect and no mobile deep-linking, and that's recorded in the
> audit as FE-06."

### 5.3 React patterns in `CheckoutQuotePanel`

The hooks, and the reason for each — they will ask "why a `ref` here?":

- `useMemo(() => routeFrom(option), [option])` — stable `PaymentRoute` object, so the
  effects that depend on it don't re-fire every render.
- `useRef(option.method)` — compares the *previous* method against the current one to
  reset quote/payment/balance when the user switches token. A quote is bound to one
  token on one chain, and so is a balance reading. A ref, not state, because changing
  it must not itself trigger a render.
- `useEffect` + `setInterval` for the countdown, **with the cleanup returning
  `clearInterval`** — and it reads `expires_at` from the server rather than counting
  down from a client-side number, so a slow tab can't appear to extend a quote.
- `useCallback` for `refreshBalance`, because it's in another effect's dependency
  array.
- `watchWallet` returns an **unsubscribe function** that the effect returns directly —
  so account/network changes mid-checkout invalidate the balance reading.

> "The one `eslint-disable` for `exhaustive-deps` is deliberate and narrow: I want the
> balance to refresh on `wallet.address`, `wallet.chainId` and `quote.crypto_base_units`
> specifically, not on every new quote object identity."

### 5.4 Auth on the client

`getAuthSession()` base64url-decodes the JWT payload (restoring stripped `=`
padding), and if `exp` is past, clears the session and fires an event.

**Lead with the caveat:**

> "That decode is explicitly *not* verification — the comment says so. It's a UX check
> so an expired session logs out rather than firing a doomed request. The signature is
> only ever checked server-side. And `api.ts` only treats a 401 as 'session expired'
> when the request actually carried a token, so a wrong password on login surfaces the
> backend's real message instead of 'your session expired'."

On `localStorage` — their likely challenge. Don't be defensive:

> "XSS-readable, and that's a real cost. It's FE-06 in the audit as an accepted,
> documented risk. The fix is an httpOnly, SameSite cookie with CSRF protection, which
> is a backend change too. I chose to document it rather than half-do it."

### 5.5 Build and types

```bash
npx tsc --noEmit    # clean
npx vite build      # succeeds
```

`vite.config.ts` has manual chunking (`radix-vendor`, `charts-vendor`,
`icons-vendor`, `vendor`). `charts-vendor` is 312 kB — recharts. If they press on
bundle size:

> "Recharts is a third of the bundle for the dashboard. The honest fix is a dynamic
> `import()` so it only loads on the routes that chart anything — I haven't done it."

---

## 6. Smart contracts tour

**Open with the status, every time:**

> "Both contracts are written and tested. Neither is wired into the backend. The live
> payment path is direct payment with on-chain verification. I'd rather say that than
> have you discover it."

### 6.1 EVM — `contracts/ethereum/contracts/AeroLedgerEscrow.sol` (516 lines)

`ReentrancyGuard, Pausable, Ownable2Step, EIP712` from OpenZeppelin 5.

**Three parties, destinations snapshotted at creation.** Buyer, registered seller,
platform treasury. `_openOrder` records `payoutWallet` and `treasury` **into the
order**, so re-pointing a merchant afterwards cannot retarget money already escrowed.
There is a test for exactly that: *"keeps paying the wallet recorded at creation even
if the merchant is re-pointed"* — the attacker receives 0.

**Price is a signed commitment.** `_verifyQuote` checks an EIP-712 signature from the
configured quote signer over the order hash and the `QuoteInput` struct. The buyer
cannot name their own amount.

> "EIP-712 rather than a raw `eth_sign` hash because the domain separator binds the
> signature to this chain id and this contract address. A signature harvested from a
> testnet deployment is useless on mainnet. `domainSeparator()` is exposed as a view
> so a client can check it's signing for what it thinks it is."

**Fee-on-transfer safety.** `_pullTokens` measures the contract's *actual* balance
delta and reverts if it's short of the quoted amount, rather than trusting the
`transferFrom` argument. (EVM-07; there's a `FeeOnTransferToken` mock.)

**Exits.** `refundOrder` (buyer, before the window closes, pending only),
`timeoutRefund` (buyer, after the deadline, including a confirmed-but-never-completed
order), `sweepUnaccounted` (owner, and *only* funds above what the escrow ledger
accounts for, so it can't touch a live order).

52 tests, `npx hardhat test`, ~10s. Run it live if there's time — it's persuasive.

### 6.2 Solana — `contracts/solana/programs/aeroledger-escrow` (1,693 lines)

Anchor. Accounts: `Config`, `SupportedMint`, `Merchant`, `Order`. Instructions:
`initialize`, `set_config`, `set_paused`, `transfer_admin`/`accept_admin`,
`register_mint`/`revoke_mint`, `register_merchant`/`update_merchant`, `create_order`,
`confirm_order`, `complete_order`, `refund_order`, `timeout_refund`.

**The two-check composition — know this exactly.** On `complete_order`:

```rust
#[account(mut, seeds = [b"order", order.order_id_hash.as_ref()], bump = order.bump,
          has_one = mint, has_one = buyer,
          has_one = payout_wallet, has_one = treasury)]
pub order: Account<'info, Order>,

#[account(mut, associated_token::mint = mint, associated_token::authority = payout_wallet)]
pub seller_token_account: Account<'info, TokenAccount>,
```

> "`has_one = payout_wallet` forces the passed account to equal the key recorded **on
> the order**. `associated_token::authority = payout_wallet` forces the token account
> to be that key's canonical ATA for the order's mint. No instruction argument anywhere
> in the settlement path names a destination — so no signer, buyer, seller or admin,
> has one to manipulate."

**Quote attestation.** `quote.rs` verifies an **Ed25519 signature via instruction
introspection**: it reads the `Ed25519SigVerify` instruction from the instructions
sysvar and checks the signature is over the message *this* instruction expects.

> "The subtle part is that it must be self-contained. The tests cover rejecting
> multi-signature instructions, rejecting data borrowed from another instruction, and
> rejecting a message that's a prefix of the expected one. Those are the ways
> introspection goes wrong."

**Who confirms.** Only `Merchant.authority` can `confirm_order` — the admin
deliberately cannot forge a seller's acceptance. `complete_order` is callable by the
seller *or* the admin, but as shown above neither can redirect the funds.

```bash
cd contracts/solana/programs/aeroledger-escrow && cargo test --lib   # 14 tests
```

**The Anchor integration suite is written and NOT executed** —
`contracts/solana/tests/aeroledger-escrow.ts`, 30+ adversarial cases. No Solana
toolchain on this machine. State it plainly; the EVM suite covers the equivalent
behaviours and *is* executed.

### 6.3 If they ask "which would you wire up first?"

> "EVM, for one reason: the test suite actually runs, so I'd be integrating against
> something I can prove. I'd also put `Config.admin` and `owner()` behind a multisig
> with a timelock, and move the quote signer to a KMS/HSM key, before either touched
> real money. Both are in the known-limitations list."

---

## 7. How the three layers fit together

The sentence that ties the whole system up — worth memorising:

> "The **price** is decided in one place and only one place: the backend, against the
> live catalogue, snapshotted into an expiring quote the user explicitly approves. The
> **frontend** never computes a price — it displays the server's figures and echoes
> them back on approval. The **contracts** don't trust the buyer either: the price
> arrives as a signed commitment from that same backend authority, and the payout
> destinations are snapshotted at order creation so no signer can redirect them. Three
> layers, one source of truth, and each of the other two refuses to invent a number."

### Test evidence — know the numbers

| Suite | Command | Result |
|---|---|---|
| Backend unit | `cargo test --lib` | **51 passed** |
| Backend integration (real Postgres) | `cargo test --test integration_postgres -- --ignored --test-threads=1` | **10 passed** |
| Solana program unit | `cargo test --lib` in the program dir | **14 passed** |
| EVM contracts | `npx hardhat test` | **52 passed** |
| Rust format / lint | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` | clean |
| Frontend types / build | `npx tsc --noEmit`, `npx vite build` | clean / succeeds |

**127 executed, 0 failing.** Plus the unexecuted Anchor suite (§6.2).

---

## 8. Rapid-fire answers

**"Why Rust?"**
> "Payment code where the compiler enforces that I handled the error case. No
> `null`, no unchecked exception path, and `Result` means an unhandled failure is a
> compile error. The `u128` base-unit arithmetic is also exact by construction —
> there's no float in the language's way."

**"Why no ORM / why raw SQL?"**
> "The settlement path needs `SELECT ... FOR UPDATE OF p, b` with a re-check inside
> one transaction, and partial unique indexes with `WHERE` clauses. That's the part I
> most want to read literally. Diesel or SeaORM would be fine for the catalogue, and
> the cost is that my repositories are hand-written and I own the mapping."

**"Why Actix over Axum?"**
> "Mature middleware story and `actix-governor` for rate limiting off the shelf.
> Axum would have been a fine choice; I'd weigh it on the Tower ecosystem if starting
> again."

**"How would you scale this?"**
> "Nothing in the backend holds per-user state — the only in-process state is the
> oracle's 60-second price cache, which is a cache, so it scales horizontally behind a
> load balancer today. The real constraints come first: Postgres connection limits via
> the deadpool size, and the external chain APIs, which are rate-limited and the thing
> I'd put a shared Redis cache in front of. The reconciliation worker is the piece
> that *doesn't* scale — it runs per instance, so N instances means N pollers. It
> needs a leader election or a proper job queue before you run more than one."

**"What's your testing strategy?"**
> "Three layers. Unit tests on pure domain logic — money arithmetic, confirmation
> counting, quote staleness. Use-case tests with hand-written fakes for the three
> ports, which is what the hexagonal layering buys. Integration tests against a real
> Postgres for anything where the database *is* the invariant — the unique indexes,
> the row locking, the replay guard. Integration tests are `#[ignore]`d so a plain
> `cargo test` doesn't require a database."

**"Your worst code?"**
> Pick one and mean it. For example:
> "`pages/Checkout.tsx` at 603 lines. It's composition plus flow orchestration plus
> error presentation. `CheckoutQuotePanel` was extracted from it, and the booking
> creation and status presentation should come out the same way."

**"What would you do next?"**
> "In order: rotate the leaked credentials — that's OPS-01 and it blocks everything
> else; wire the EVM escrow to the backend; move governance to a multisig and the
> quote signer to a KMS; move the JWT to an httpOnly cookie; and get a Solana
> toolchain installed so the Anchor suite actually runs."

---

## 9. Say this unprompted

Near the end, before they find it. It is the strongest move available:

> "Four things I'd want you to know rather than discover.
>
> **One — there are live credentials in this repository's public history.** An
> Etherscan key and a QuickNode RPC endpoint whose URL path *is* the auth token are on
> the default branch right now, in `.env.example` and `.env.production`, and a database
> URL and JWT secret are retrievable from earlier commits. My branch replaces them with
> placeholders, which stops the ongoing leak but does not undo the exposure — those
> keys have to be revoked and rotated, and purging history doesn't remove the need to.
> It's OPS-01, marked critical and open.
>
> **Two — the escrow contracts aren't wired in.** They're built and tested; the live
> path is direct payment with on-chain verification.
>
> **Three — the Anchor integration suite has never been executed.** It's written, it's
> committed, and there's no Solana toolchain on my machine. The EVM equivalents do run.
>
> **Four — flight availability isn't reserved.** A booking records a price; it doesn't
> hold a seat. The catalogue is seeded data with no provider behind it."

Also have ready, if pushed: single-key governance on both contracts, the quote
signer is a hot key, Solana settles native SOL only (SPL is explicitly rejected
rather than approximated), stablecoins are pinned to $1 by policy so a de-peg between
quote and settlement is unhedged, and payment-proof uploads aren't virus-scanned.

### When you don't know

Say so, then show how you'd find out:

> "I don't know that off the top of my head. It'd be in `<file>` — want me to open
> it?"

That reads as competence. Guessing does not.
