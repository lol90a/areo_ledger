# AeroLedger — Second Interview Preparation

The interviewer has already seen the insecure version. What they want to know is
whether you understood the feedback and can defend the engineering. This document
is built so you can walk through it top to bottom and have an answer, with evidence,
for every question they are likely to ask.

**Two rules for the conversation:**

1. **Never claim something is secure that you have not tested.** Section 9 is the
   list of things that are *not* verified. Volunteering it is the single strongest
   move you can make — senior engineers trust people who know the limits of their
   own work, and they will find these gaps anyway.
2. **Lead with the invariant, not the code.** "No signer can choose where money
   goes, because the destination is snapshotted at order creation" is a better
   opening than a tour of `#[account(...)]` attributes.

---

## 1. The ninety-second opening

> "You told me the project wasn't secure, and you were right. The core problem was
> that the escrow didn't model a marketplace at all: there was no seller, so
> `complete_order` paid the admin's token account, and the buyer passed the amount
> in as an instruction argument, so anyone could call the program directly and pay
> one lamport for a fifty-thousand-dollar booking.
>
> I redesigned both contracts around a real three-party relationship — buyer,
> registered seller, platform treasury — with payout destinations snapshotted into
> the order at creation so nobody, including the admin, can redirect funds. Price is
> now a signed commitment from the backend, verified on chain: Ed25519 instruction
> introspection on Solana, EIP-712 on EVM.
>
> On the way through I found four more issues I'd call critical that weren't in your
> feedback: the backend accepted *reverted* Ethereum transactions as payment, it
> accepted *pending* ones as deeply confirmed, every settlement path quietly allowed
> a half-percent underpayment, and the frontend showed 'Payment Successful' from a
> three-second `setTimeout`.
>
> There's a full quote lifecycle now with expiry and explicit re-approval, and
> wallet balance pre-flight checks. 127 tests pass across four suites. The Anchor
> integration tests are written but I couldn't execute them — no Solana toolchain on
> my machine — and the escrow contracts aren't wired into the backend yet. I'd
> rather tell you that than have you find it."

Then stop and let them drive.

---

## 2. Their three questions

### Q1 — "Who are the parties in the Solana contract?"

**Problem → Why it was insecure → Attack → Fix → Why the fix works → Evidence**

**Problem.** The original `Order` stored `buyer`, `admin`, `mint`, `amount`. There
was no seller field. `complete_order` transferred the whole escrow balance to
`admin_token_account`.

**Why it was insecure.** The escrow encoded a buyer ⇄ platform relationship. For a
marketplace that is wrong in two ways: the seller has no on-chain claim to their own
proceeds, and "the administrator" was implicitly the merchant for every sale — so a
single compromised key took 100% of every order and the chain showed it as routine.

**Attack.** A seller lists a jet share; a buyer escrows 50,000 USDC; on completion
every cent lands in the admin's wallet. There is nothing on chain tying those funds
to the seller — no record a seller could point at, no distinction between platform
revenue and seller proceeds.

**Fix.** Three parties, explicit:

```rust
// Governance registers a seller and where they get paid.
#[account] #[derive(InitSpace)]
pub struct Merchant {
    pub authority: Pubkey,       // seller's identity key; signs confirm_order
    pub payout_wallet: Pubkey,   // where proceeds land
    pub active: bool,
    pub bump: u8,
}

// create_order snapshots the destinations into the order.
order.seller        = ctx.accounts.merchant.authority;
order.payout_wallet = ctx.accounts.merchant.payout_wallet;
order.treasury      = ctx.accounts.config.treasury;
```

and completion pins both destinations with account constraints:

```rust
#[account(mut, seeds = [b"order", order.order_id_hash.as_ref()], bump = order.bump,
          has_one = mint, has_one = buyer,
          has_one = payout_wallet, has_one = treasury)]
pub order: Account<'info, Order>,

#[account(mut, associated_token::mint = mint, associated_token::authority = payout_wallet)]
pub seller_token_account: Account<'info, TokenAccount>,
```

**Why the fix works.** Two checks compose. `has_one = payout_wallet` forces the
passed account to equal the pubkey recorded *on the order*;
`associated_token::authority = payout_wallet` forces the token account to be that
key's canonical ATA for the order's mint. There is no parameter anywhere in the
settlement path that names a destination, so no signer — buyer, seller or admin —
has one to manipulate. Snapshotting matters independently: re-pointing a merchant's
payout wallet after funding does **not** retarget money already in escrow.

**Evidence.** The equivalent EVM behaviour is executed and green:

- *"pays the seller's registered wallet and the treasury, never the owner"* —
  asserts exact balance deltas for seller, treasury and escrow, **and** that the
  owner's balance is unchanged.
- *"keeps paying the wallet recorded at creation even if the merchant is
  re-pointed"* — governance re-points mid-flight; the attacker receives exactly 0.
- *"rejects confirmation by anyone other than the seller"* — owner, buyer and a
  random account all revert.

The Anchor equivalents are written; see §9 for why they are unexecuted.

**Expect the follow-up:** *"So what CAN your admin do?"* Have the answer ready:

> Register and deactivate merchants and mints, rotate the quote authority and the
> treasury, pause new orders, complete a confirmed order — to the **registered**
> seller wallet — and refund an order to the **recorded** buyer. It cannot pick a
> destination, change a live order's amount, confirm on a seller's behalf, or block
> a buyer's exit: `timeout_refund` is unilateral to the buyer once the deadline
> passes. A compromised admin is a denial-of-service and a reputational problem, not
> a theft path. It should be a multisig with a timelock; today it is one key, and
> that's a real residual risk.

---

### Q2 — "What if the price changes while the user is booking?"

**Problem.** The price was fixed when the booking row was created and never
revisited. There was no expiry, no detection of a catalogue change, no concept of
the user approving a figure, and no lock once they had.

**Attack / failure.** Two directions, and it is worth naming both:

- Customer opens checkout at $5,000, the provider re-prices to $5,500, the customer
  pays $5,000 — the platform under-collects.
- Or the booking re-reads the catalogue at payment time and the customer is charged
  $5,500 having only ever seen $5,000 — a chargeback and, depending on jurisdiction,
  a consumer-law problem.

Separately, the crypto amount had no expiry at all: come back two days later and pay
at a rate that has moved 40%.

**Fix.** A full quote lifecycle — new table, entity, two use cases, two endpoints.

```rust
pub fn accept(&mut self, approved_total_cents: i64,
              approved_base_units: u128, now: DateTime<Utc>) -> Result<(), DomainError> {
    // ... status and expiry guards ...
    if approved_total_cents != self.pricing.total.cents() {
        return Err(DomainError::Conflict(format!(
            "Price changed: you approved {} but the current quote is {}. \
             Review and confirm the new price.", ..)));
    }
    if approved_base_units != self.crypto_base_units {
        return Err(DomainError::Conflict(/* token amount changed */));
    }
    self.status = QuoteStatus::Accepted;
    self.accepted_at = Some(now);
    Ok(())
}
```

Seven rules, each enforced in code:

1. Server owns the price — re-derived from the catalogue on every quote.
2. Quotes expire after 600 seconds.
3. Acceptance **echoes back** the exact fiat total and the exact token base units.
4. Acceptance locks the base-unit amount permanently for that payment.
5. One live quote per booking — superseding happens in the same DB transaction,
   backed by a partial unique index.
6. `Payment::from_quote` is the only constructor and refuses anything not
   `Accepted`. There is no code path that prices a payment any other way.
7. A re-price after acceptance voids it.

**Why the fix works.** Rule 3 is the one to emphasise. A stale browser tab cannot
charge a stale price, because the server compares what the client *says it showed
the user* against what the quote actually is. A mismatch is a 409 with the new
figures, and the UI renders old-struck-through / new-bold and asks again. Rule 5
stops the obvious counter-move of holding the old cheaper quote.

**Evidence — executed against real PostgreSQL:**

```
a_price_change_between_booking_and_checkout_is_surfaced_and_must_be_re_approved
```

It sets a flight to $5,000, quotes it ($5,750 with markup and fee), re-prices to
$5,500 mid-checkout, re-quotes, asserts `price_changed == true` and
`previous_total_cents == 575_000`, asserts accepting the **old** figures fails,
asserts the **superseded** quote also fails, then approves the new figures and
confirms $6,325.00 is what gets locked. Plus `an_expired_quote_cannot_be_accepted`,
`only_one_quote_per_booking_is_ever_active`, and 11 unit tests on the entity.

**Expect the follow-up:** *"What's your price-lock policy after acceptance?"*

> An unaccepted quote lives ten minutes. Once accepted, the token amount is fixed
> for that payment indefinitely — the platform carries the FX risk from acceptance
> to settlement. That's deliberate: the alternative is telling a customer whose
> transaction is already on chain that they now owe more. It's an accepted,
> documented exposure, not an oversight, and if you wanted it bounded I'd cap the
> settlement window and void-and-refund past it rather than silently re-price.

---

### Q3 — "How do you know the customer has enough balance?"

**Problem.** It did not exist. No wallet library in the dependency tree, no balance
read anywhere in the frontend.

**Fix.** `Frontend/src/app/lib/wallet.ts` plus `CheckoutQuotePanel.tsx`:

- EVM via EIP-1193 (`window.ethereum`); Solana via the injected provider.
- Native balance: `eth_getBalance` / Solana `getBalance` at `confirmed`.
- **ERC-20 balance: `eth_call` to `balanceOf(address)`** — selector `0x70a08231`
  with the address left-padded to 32 bytes — against the contract address the
  **backend** supplied, not one hard-coded in the bundle.
- **Everything in `bigint` base units.** No float touches a balance comparison.
- Fee headroom: 0.002 ETH, or 0.003 SOL (covering the signature fee and one
  rent-exempt account), with a separate "native balance looks thin" warning on
  ERC-20 routes where gas comes from a different balance than the payment.
- Insufficient → show required, available and the exact shortfall; **disable the
  approve button**.
- Wrong network → detected against the server-supplied chain id, blocks approval,
  offers `wallet_switchEthereumChain`, and **re-reads the chain id afterwards**
  rather than assuming the switch happened — a user can dismiss that prompt.
- `accountsChanged` / `chainChanged` / `accountChanged` / `disconnect` invalidate
  the reading.
- The balance is **re-read immediately before approval**, not just on page load.
- Bitcoin is marked explicitly unsupported for balance checks — there is no browser
  wallet standard — rather than pretending.

**The sentence that matters.** Say this before they say it to you:

> This is a UX pre-flight, not a security control. A browser can report any balance
> it likes. Its only job is to tell someone they're short *before* they sign. What
> actually enforces payment is the chain — a transfer with insufficient funds simply
> fails — and the backend, which verifies the settled transaction against the locked
> amount. I wrote that in a comment at the top of the module so nobody downstream
> mistakes it for enforcement.

**Expect the follow-up:** *"Why no wagmi / wallet-adapter?"*

> `window.ethereum` and `window.solana` are stable injected standards, and the
> checkout needs connect-and-read-balance. Pulling wagmi + viem or
> `@solana/wallet-adapter` into a React 18 + Vite app adds a large dependency tree
> for that. The cost is real and I'd list it as a limitation: no WalletConnect, no
> mobile deep-linking, no multi-wallet modal. If we need mobile or hardware wallets,
> that's the point to adopt an SDK.

---

## 3. The four issues they did *not* mention

Bring these up yourself. They demonstrate that you audited rather than just patched.

### A reverted transaction was accepted as payment — critical

```rust
// BEFORE: the only source of truth was the transaction object.
let base_units = Self::hex_to_u128(result["value"].as_str().unwrap_or("0x0"));
```

`value` is what the sender *asked* to transfer. It is present and unchanged on a
transaction that reverted and moved nothing.

> Send ETH to a contract you control that takes it and reverts — or just run out of
> gas. No value moves. Submit the hash. The backend reads `value`, sees the `to`
> matches, counts confirmations, and settles. Free goods, with a failed transaction
> in the public record.

Fixed by fetching `eth_getTransactionReceipt` **first** and requiring
`status == 0x1`, with `succeeded` threaded into the decision.
Test: `a_reverted_transaction_is_never_verified`.

### A pending transaction looked deeply confirmed — critical

```rust
// BEFORE: a pending tx has blockNumber: null -> 0.
fn block_from_hex(hex: Option<&str>) -> u64 { ... .unwrap_or(0) }
let confirmations = current.saturating_sub(tx_block) as u32;   // ~21,000,000
```

> Broadcast with a low gas price, submit the hash immediately, order settles. Then
> replace the transaction so it never mines. Confirmed order, no money.

Fixed: `mined_block()` returns `Option<u64>` and treats `null` and `0x0` as not
mined; `confirmations_for()` counts the mining block as one and returns 0 if the
node's tip lags. Tests: `a_pending_transaction_has_no_mined_block`,
`confirmations_count_the_mining_block_and_never_underflow`.

### A 0.5% underpayment was accepted everywhere — high

The same `tolerance = expected / 200` appeared in **five** places: three gateways,
the confirm use case, and the reconciliation worker.

> Not really an attack — a standing leak. 0.5% of every order forever, and anyone
> who notices just always sends 99.5%. On a $50,000 jet share that's $250. The
> justification was rounding, but the quote is already rounded *up* to the token's
> precision when it's issued, so the payer is told the exact figure. There's no
> slack to concede.

Now exact-or-more everywhere. Tests:
`an_underpayment_of_one_base_unit_is_rejected`,
`rejects_a_payment_one_base_unit_short_of_the_quote`, `accepts_an_overpayment`.

### "Payment Successful" came from a timer — critical

```ts
const handlePay = () => {
  setStep("processing");
  setTimeout(() => setStep("success"), 3000);   // settlement by stopwatch
};
```

> Nothing was priced, no wallet was touched, no chain was consulted, and the user
> got a green tick saying their purchase was confirmed. I deleted the simulated flow
> entirely rather than hiding it behind a demo flag, because a fake success state in
> the same component as a real one is how it leaks into production.

Also worth mentioning, because it is a subtler version of the same instinct: the
Solana gateway's confirmation check was `.max(if succeeded { 1 } else { 0 })` against
a minimum of 1 — arithmetically always true. A check that cannot fail is worse than
no check, because it looks like coverage.

---

## 4. Demonstrations — run these live

### Attacks failing, on EVM (7 seconds, fully executed)

```bash
cd contracts/ethereum && npx hardhat test
```

Point at these names as they scroll past:

```
server-authoritative pricing (signed quotes)
  ✔ rejects an order whose calldata amount differs from the signed amount
  ✔ rejects a quote issued to a different buyer (quote theft)
  ✔ rejects a quote signed with the wrong chain id (cross-chain replay)
  ✔ rejects replaying the same signed quote twice
buyer -> seller settlement (native)
  ✔ pays the seller's registered wallet and the treasury, never the owner
  ✔ keeps paying the wallet recorded at creation even if the merchant is re-pointed
native / ERC20 path separation (cross-asset drain)
  ✔ a tiny native deposit cannot drain pooled USDC
reentrancy and payout safety
  ✔ a malicious buyer cannot re-enter refundOrder to double-withdraw
```

### Price changes, against a real database

```bash
docker run --rm -d --name aeroledger-test \
  -e POSTGRES_PASSWORD=postgres -p 55439:5432 postgres:16
TEST_DATABASE_URL=postgres://postgres:postgres@127.0.0.1:55439/postgres \
  cargo test --test integration_postgres -- --ignored --test-threads=1
```

### Quote-forgery parsing, on Solana

```bash
cd contracts/solana/programs/aeroledger-escrow && cargo test --lib
```

Eleven cases covering a cheaper amount, a stolen quote, the wrong signer, offsets
pointing at another instruction, multi-signature and zero-signature payloads,
truncation and out-of-bounds offsets — plus
`account_sizes_match_their_field_layout`, which pins the account sizing. 14 in all.

### The frontend

Login → Marketplace/Flights → Cart → Checkout. At the payment step:

1. **Get price** → the breakdown, the exact token amount in both whole units and
   base units, the rate source and observation time, and a live countdown.
2. **Connect wallet** → address, chain id, balance, and either a green reading or a
   red "short by X" with the approve button disabled.
3. Switch to the wrong network in MetaMask → the panel blocks approval and offers to
   switch back.
4. **Approve** → the price locks and the deposit address appears.
5. Let a quote expire → "This quote expired. Refresh the price to continue."

---

## 5. Technical Q&A — Rust

**Ownership and borrowing, concretely in this code.** `settle_refund` takes
`&mut Context<RefundOrder>` because it mutates `order.status` before the CPI;
everything after is immutable borrows of sibling accounts, which the borrow checker
allows because the mutable borrow has ended. The first version took `&Context` and
tried `ctx.accounts.order.clone()` — it did not compile, which is exactly the point:
the checker caught an accidental write to a copy that would have silently left the
real account in its old state.

**Lifetimes.** `close_vault<'info>(token_program: &Program<'info, Token>, ...)` ties
every account reference to the same `'info`, the lifetime of the deserialised
account data for one instruction. Anchor's generated structs carry it so you cannot
hold an account reference past the instruction.

**Traits and generics.** Use cases are generic over repository traits:
`CreateQuote<BR: BookingRepository, QR: QuoteRepository, ...>`. Static dispatch, no
vtable, and the real payoff is testability — `init_payment.rs` has six tests with
hand-written stubs and no database. The one `dyn` is `Arc<dyn PriceOracle>`, because
it is shared across threads in Actix app state and the dispatch cost is irrelevant
next to an HTTP call.

**Why `#[async_trait]`?** Native async fns in traits still don't give you
`dyn`-compatible async traits without extra machinery; `async_trait` boxes the
future so `Arc<dyn PriceOracle>` works. One allocation per call, on a path that
makes a network request.

**Tokio and the worker.** `PaymentReconciliationWorker::spawn` is a detached task
looping over `run_once` with `tokio::time::sleep`. Errors are isolated per payment
so one flaky provider cannot starve the batch. Multi-replica safety comes from the
database, not from the task: `SELECT ... FOR UPDATE` plus a conditional
`UPDATE ... WHERE status = 'pending'`, so concurrent replicas serialise and only one
settles.

**Actix extractors.** `web::Data<T>` for shared state, `web::Json<T>` for bodies
with a custom error handler so a malformed body is a clean 400. The auth middleware
implements `Transform` + `Service` and injects validated `Claims` into request
extensions; handlers read them from there and never trust a `user_id` in a body.

**Money.** `Money` is `i64` cents plus a currency; `apply_bps` is the only
percentage operation (`cents * bps / 10_000`, checked). There is no `f64` in any
financial path. Crypto amounts are `u128` base units. The conversion happens exactly
once, at quote issue, rounded **up**, and the integer result is what gets locked.

**Error handling.** `DomainError` in the domain, `InfraError` in infrastructure with
`From<InfraError> for DomainError` at the boundary, and one mapping to HTTP status
codes in `interfaces`. Nothing is swallowed: a provider failure is `pending`, never
`verified`.

---

## 6. Technical Q&A — Solana

**Accounts and PDAs.** Solana programs are stateless; all state is in accounts the
caller passes in, which is why *validating what you were handed* is the whole game.
PDAs are addresses off the ed25519 curve, derived from seeds plus the program id, so
only the program can sign for them. Here: `["config"]`, `["mint", mint]`,
`["merchant", authority]`, `["order", keccak256(order_id)]`.

**Why hash the order id?** A seed is capped at 32 bytes; a 36-character UUID exceeds
it. The seed is `keccak256(order_id)` and the full string is stored in account data
for off-chain correlation.

**PDA signer seeds.** To move tokens out of the vault the program signs as the order
PDA:

```rust
let seeds: &[&[u8]] = &[b"order", order_id_hash.as_ref(), &[bump]];
token::transfer(CpiContext::new_with_signer(.., Transfer { authority: order.to_account_info(), .. },
                                            &[seeds]), amount)?;
```

The bump is stored on the account so the canonical bump is reused — recomputing it
costs compute, and accepting a caller-supplied bump would allow a different valid
PDA.

**Which constraint does what.** Be ready for this; it is the question that separates
people who copied a tutorial from people who understand it.

| Constraint | Guarantees |
|---|---|
| `seeds = [...] , bump = x.bump` | the account is *this program's* PDA for those seeds |
| `has_one = field` | `account.field == ctx.accounts.field.key()` |
| `associated_token::mint/authority` | the token account is the canonical ATA of that authority for that mint |
| `Account<'info, T>` | owned by the expected program **and** the discriminator matches |
| `Program<'info, Token>` | the account is the legacy SPL Token program — which is how Token-2022 is excluded |
| `Signer<'info>` | signed this transaction — note it says *nothing* about **who** |

That last row is the original bug in one line: a `Signer` named `admin` is just a
signature. Authority has to come from state — `config.admin`, `merchant.authority` —
not from a variable name.

**Instruction introspection, in your own words.** The Instructions sysvar lets a
program read the other instructions in its transaction. We require the one at
`current_index − 1` to be the native Ed25519 program, then verify that what the
runtime checked was our message by our signer. The runtime does the cryptography;
the program does the *binding*. Without that binding an attacker just adds a
harmless Ed25519 instruction and the check means nothing.

**Why reject Token-2022?** Transfer fees break "what went in is what comes out";
transfer hooks run arbitrary code during the transfer; confidential transfers hide
the amount. Each needs its own analysis. Rejecting is fail-closed and honest.

**Account sizing.** `space = 8 + Order::INIT_SPACE` via `#[derive(InitSpace)]`, with
`#[max_len(64)]` on the `order_id` string, and an executed test asserting the derived
number equals the hand-audited layout. `OrderStatus` implements `Space` explicitly
rather than relying on a derive's treatment of enums.

**Why not close the Order account?** Closing would free the PDA and let
`create_order` re-initialise the same `order_id`, re-opening a settled order. The
account *is* the replay guard. The vault ATA is closed (rent back to the buyer)
because it is empty and re-creatable harmlessly.

---

## 7. Technical Q&A — EVM

**EIP-712 vs `eth_sign`.** Typed structured data: the domain separator binds
`chainId` and `verifyingContract`, so a signature is useless on another chain or
another deployment. The wallet can also show the user what they are signing. We
`tryRecover` and compare with `quoteSigner`, rejecting on any `RecoverError` rather
than ignoring it.

**Reentrancy.** `nonReentrant` on every value-moving path plus strict
checks-effects-interactions: status is written and `escrowBalance` debited *before*
the payout. The test drives a malicious buyer contract that re-enters `refundOrder`
from `receive()`; the re-entrant call reverts, which surfaces as a failed native
payout, and the escrow balance is unchanged.

**`.call` vs `.transfer`.** `.transfer` forwards 2300 gas, which is not enough for a
Gnosis Safe's `receive()`. We use a checked low-level `.call` so a contract payout
wallet is not bricked — and there is a test with a multisig mock whose `receive()`
writes storage.

**`SafeERC20`.** USDT returns no bool from `transfer`; a naive
`IERC20(t).transfer(...)` reverts on ABI decoding. `SafeERC20` handles the empty
return and turns a silent `false` into a revert. Tested with a hand-rolled
no-bool token.

**Fee-on-transfer — and why I changed the behaviour.** The previous version credited
whatever arrived. With a signed price that *is* the bug: a 1% burn means the escrow
holds 1% less than the agreed price on every order, silently. Now the deposit
reverts if the measured delta is below the quoted amount, which makes such tokens
unusable — correctly, since the owner shouldn't allow-list one. I flagged it as a
deliberate behaviour change in the audit rather than letting it look like a
regression.

**Cross-asset drain.** Native and ERC-20 have separate entry points and
`escrowBalance` is tracked per asset, so a payout can never exceed what was
deposited for that asset. Test: *"a tiny native deposit cannot drain pooled USDC"*.

**Stuck funds.** `sweepUnaccounted` can move only `held − escrowBalance[token]`, so
escrowed funds are arithmetically out of reach. The test force-sends tokens, proves
escrowed native is *not* sweepable, sweeps the surplus, then refunds the live order.

**Upgradeability.** This is a new deployment, not an upgrade. Not proxied; the
storage layout changed incompatibly. Any existing deployment must be drained through
its own refund paths first. I make no upgrade-compatibility claim.

---

## 8. Technical Q&A — system design

**PostgreSQL transactions and the settlement path.** One transaction:
`SELECT ... FOR UPDATE OF p, b` to serialise concurrent confirms, re-check status
(`confirmed` → idempotent `AlreadyConfirmed`), reject a hash attached to a different
booking, then `UPDATE payments ... WHERE status = 'pending'` — zero rows means
someone else won the race — then the booking and the ledger row, then commit.

**Three layers of replay protection, and why three.** The use case checks, the
settlement transaction re-checks under a lock, and a partial unique index on
`payments(tx_hash)` is the backstop. The first two are races by nature; the index is
not. Test: a triple confirm produces exactly one ledger row.

**Idempotency.** `ConfirmPayment` returns `Ok(())` for an already-confirmed payment
rather than erroring, so a client retry or a worker overlap is safe.

**Why `NUMERIC(40,0)` for base units?** `u128` covers every supported token's
supply, and keeping it as an exact integer in the database — read back as text and
parsed — means no float ever touches the figure the chain is compared against.

**Clean architecture, and what it bought here.** `domain` depends on nothing, so
`PaymentQuote::accept` — where the price-change rules live — is unit-testable with
no database and no HTTP. That is why there are 11 executed tests on quote semantics
and 9 against real Postgres for the parts that genuinely need a database.

**Fail-closed, concretely.** No provider answer ⇒ pending, never verified. No
trustworthy rate in live mode ⇒ refuse to quote rather than invent a price. Config
validation refuses to boot production with a placeholder secret, `manual`
verification mode, startup migrations, or a localhost CORS origin.

---

## 9. What is NOT done — say this unprompted

**The most important slide in your deck.**

1. **Neither contract has been deployed to any cluster.** The Anchor suite does
   now run — 38 cases against a local `solana-test-validator` — covering quote
   forgery, account substitution, PDA seed mismatch, authorisation, state-machine
   violations, double completion, double refund, timeouts and pause. What a local
   validator does not prove: behaviour under a real fee market, compute limits on
   a congested cluster, or anything about a mainnet deploy. The program id is
   still the placeholder, deliberately.

   Worth saying out loud: running that suite is what found SOL-14. `create_order`
   overflowed the 4KB BPF stack frame, so it could never have executed on-chain —
   and every host-target check stayed green the whole time. A written test is not
   a passing test.

2. **The escrow contracts are not wired into the backend.** This is the honest
   headline. The live flow is direct payment with on-chain verification: quote →
   accept → deposit to a platform address → verify → settle. The escrow contracts
   implement a custodial-escrow model with a seller and a fee split. Both are
   coherent; they are not the same flow yet. `ARCHITECTURE.md` §5 lists the five
   steps to connect them. I am not going to tell you an order placed on the website
   is settled by the Anchor program, because it isn't.

3. **Nothing is deployed anywhere.** No program keypair, no contract address, no gas
   numbers. The Solana program id is still the Anchor placeholder — deliberately, so
   it can't be deployed by accident.

4. **Flight availability is not reserved.** A booking records a price; it doesn't
   hold a seat. The catalogue is seeded data with no provider integration. Payment
   confirmation and fulfilment are correctly separate states, but there's no real
   reservation behind them and the product shouldn't claim one.

5. **No end-to-end browser run, and no real wallet extension test.** The frontend
   type-checks and builds, the endpoints are integration-tested, but no MetaMask or
   Phantom session has driven the full flow.

6. **No live chain-provider verification.** Gateway logic is unit-tested against
   recorded response shapes, not against Etherscan or a Solana RPC.

7. **Credentials are still in git history.** `.env.production` was deleted from the
   working tree, but it is retrievable at three earlier commits with a live
   `DATABASE_URL` and a live `ETHERSCAN_API_KEY`. Deleting a file does not remove it
   from history. Those must be rotated regardless of whether we rewrite history.

8. **Single-key governance and a hot quote-signing key.** Both should be a multisig
   and an HSM/KMS signer with per-period caps before real funds move.

9. **No independent audit.** None of this replaces one.

---

## 10. Hard questions, and honest answers

**"Your quote signer is a single hot key. Isn't that the same centralisation
problem?"**

> It's a real risk and it's in the audit. It's narrower than the old one, though:
> the old design let *anyone* choose their own price, and the admin was the payout
> destination. Now a leak of the quote key lets an attacker authorise under-priced
> orders until it's rotated — it can't redirect funds, and it can't touch an order
> that already exists. It's rotatable via `set_config` without touching the admin
> key, quotes expire in 15 minutes on chain, and every order emits `OrderCreated`
> with the amount so an anomaly is detectable. The next step is an HSM signer with a
> per-period value cap.

**"Why not make the price fully on-chain with an oracle?"**

> For a Chainlink-style feed on token/USD, yes, and I'd take that for the FX leg.
> But the *product* price — what a specific charter costs on a specific date — isn't
> an oracle-able quantity; it comes from a commercial system. So there's always a
> trusted attestation in the loop. What I can do is make that attestation
> narrowly-scoped, time-bounded, and verifiable on chain, which is what the signed
> quote is.

**"Admin can refund a confirmed order. Isn't that theft?"**

> It returns funds to the recorded buyer and nowhere else, so it's a
> denial-of-service, not theft — the admin can waste a seller's time, not take their
> money. I kept it because real marketplaces need an operator-initiated refund for
> fraud and chargebacks. The trade-off I'd make next is a timelock on admin refunds
> for confirmed orders.

**"Why did you change the fee-on-transfer behaviour? That's a regression."**

> Fair challenge, and I flagged it as a deliberate change rather than hiding it.
> Previously the contract credited whatever arrived. With a server-signed price,
> that means a 1%-burn token leaves the escrow 1% short of the agreed price on every
> order, silently — the seller is underpaid and nobody notices. Reverting makes
> those tokens unusable, which is correct: the owner shouldn't allow-list one. If we
> ever need to support them, the right fix is quoting gross-of-fee, not accepting a
> short deposit.

**"Your frontend balance check is pointless — I can fake it."**

> Completely agree, and the module says so at the top. It's a pre-flight so a
> customer finds out they're short before signing, not after a failed transaction.
> It isn't a security control and nothing downstream treats it as one: the chain
> rejects an underfunded transfer, and the backend verifies the settled transaction
> against the locked amount. If I'd presented it as enforcement, that would be a
> finding.

**"How do I know your tests actually test anything?"**

> Three things I'd point at. Every adversarial test asserts a *specific* error — a
> custom error name on EVM, a named `Err` variant in Rust — not just that something
> reverted, so a different bug can't make it pass. The settlement tests assert real
> balance deltas for every party including the ones that should receive nothing,
> like `owner` in the seller-payout test. And they already caught two of my own
> bugs: the quote repository read a `NUMERIC` column without casting it to text,
> and a `CHECK (expires_at > created_at)` constraint broke my first expiry test —
> both found by running against a real Postgres, not a mock.

**"What would you do next, with a week?"**

> Wire the escrow contracts to the backend end to end — that's the biggest honesty
> gap. In parallel, get the Anchor suite running in CI with a local validator, move
> the quote signer to KMS with a value cap, put the admin behind a multisig, and add
> `cargo audit` / `npm audit` / secret scanning as CI gates. Then rotate the
> credentials that are in git history, which actually ought to happen today.

---

## 11. One-page cheat sheet

| Thing | Where | Evidence |
|---|---|---|
| Seller exists; payouts snapshotted | `lib.rs` `Merchant`/`Order`; `AeroLedgerEscrow.sol` `merchants` | EVM: "never the owner", "even if the merchant is re-pointed" |
| Buyer can't choose the price | `quote.rs`; EIP-712 `_verifyQuote` | 11 Rust unit tests; 11 Hardhat tests |
| Quote expiry + re-approval | `domain/entities/quote.rs` | 11 unit + 8 Postgres integration tests |
| Wallet balance pre-flight | `lib/wallet.ts`, `CheckoutQuotePanel.tsx` | manual; typed + builds |
| Reverted tx rejected | `evm_gateway.rs::receipt_status` | `a_reverted_transaction_is_never_verified` |
| Pending tx rejected | `evm_gateway.rs::mined_block` | `a_pending_transaction_has_no_mined_block` |
| No underpayment tolerance | gateways + confirm + worker | `an_underpayment_of_one_base_unit_is_rejected` |
| No fake success | `PaymentModal.tsx` rewritten | timer deleted; `CheckoutPhase` vocabulary |
| Replay protection | use case + tx + unique index | `a_transaction_hash_cannot_settle_two_bookings` |
| Idempotent settlement | `pg_settlement_repository.rs` | `confirming_the_same_payment_twice_settles_once` |
| Buyer can always exit | `timeout_refund` / `timeoutRefund` | 10 Hardhat refund/timeout tests |

**Totals:** 51 backend unit + 10 Postgres integration + 14 Solana unit + 38
Anchor integration + 52 EVM = **165 executed, 0 failing**. Plus `cargo fmt`,
`clippy -D warnings`, `tsc --noEmit` and a production `vite build`, all clean.
