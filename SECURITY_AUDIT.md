# AeroLedger — Security Audit and Remediation

**Scope** — the whole repository: the Solana (Anchor) escrow program, the EVM
(Solidity) escrow contract, the Rust payment backend, and the React checkout.

**Method** — manual review of every payment-critical path, followed by implementing
fixes and writing adversarial tests that assert the attack *fails* with a specific
error rather than merely that the happy path still works.

**Status of this document** — it records what was found, what was changed, and what
was actually executed. Where something could not be run in this environment it says
so explicitly and gives the command. Nothing here claims a result that was not
observed.

---

## 0. Test evidence summary

Everything below was run on this machine, on the code as it now stands.

| Suite | Command | Result |
|---|---|---|
| Backend unit | `cargo test --lib` | **46 passed, 0 failed** |
| Backend integration (real Postgres) | `TEST_DATABASE_URL=... cargo test --test integration_postgres -- --ignored --test-threads=1` | **10 passed, 0 failed** |
| Solana program unit | `cd contracts/solana/programs/aeroledger-escrow && cargo test --lib` | **14 passed, 0 failed** |
| EVM contracts | `cd contracts/ethereum && npx hardhat test` | **52 passed, 0 failed** |
| Rust format gate | `cargo fmt --all -- --check` | clean |
| Rust lint gate | `cargo clippy --all-targets -- -D warnings` | clean |
| Frontend types | `cd Frontend && npx tsc --noEmit` | clean |
| Frontend production build | `cd Frontend && npx vite build` | succeeded |

**Not executed here:** the Anchor integration suite
(`contracts/solana/tests/aeroledger-escrow.ts`, 30+ adversarial cases). The Anchor
CLI and the Solana toolchain are not installed in this environment — `anchor` and
`solana` are both absent from `PATH`. The tests are written and committed; see
[§7](#7-what-has-not-been-verified) for the exact commands to run them.

---

## 1. Findings index

Severity uses the usual scale: **Critical** = direct loss of funds or free goods;
**High** = loss under realistic conditions or a broken security control; **Medium** =
meaningful weakening of a control, or availability/stuck-funds; **Low** = hardening
and hygiene.

| ID | Severity | Title | Status |
|---|---|---|---|
| [SOL-01](#sol-01) | Critical | Escrow had no seller; payouts went to the admin | Fixed, unit-tested + Anchor tests written (not executed) |
| [SOL-02](#sol-02) | Critical | Buyer chose the amount they paid | Fixed, unit-tested |
| [SOL-03](#sol-03) | High | `initialize` could be front-run to seize admin | Fixed |
| [SOL-04](#sol-04) | High | Any SPL mint could be escrowed | Fixed |
| [SOL-05](#sol-05) | Medium | Buyer had no exit if the counterparty went silent | Fixed |
| [SOL-06](#sol-06) | Medium | Escrow vault rent was stranded permanently | Fixed |
| [SOL-07](#sol-07) | Medium | No pause; no way to stop new orders during an incident | Fixed |
| [SOL-08](#sol-08) | Medium | Admin rotation would have bricked confirmation | Fixed |
| [SOL-09](#sol-09) | Medium | No events; nothing for an off-chain indexer | Fixed |
| [SOL-10](#sol-10) | Low | Token-2022 behaviour undefined | Fixed (fail-closed, documented) |
| [SOL-11](#sol-11) | Low | Single-step admin handover | Fixed |
| [SOL-12](#sol-12) | Info | Program id is still the Anchor placeholder | **Open — deployment blocker** |
| [EVM-01](#evm-01) | Critical | Payouts went to `owner()`; no seller existed | Fixed, tested |
| [EVM-02](#evm-02) | Critical | Buyer chose `msg.value` | Fixed, tested |
| [EVM-03](#evm-03) | Medium | Payout followed the *live* owner/treasury, not the order | Fixed, tested |
| [EVM-04](#evm-04) | Medium | No pause | Fixed, tested |
| [EVM-05](#evm-05) | Medium | Single-step ownership transfer | Fixed, tested |
| [EVM-06](#evm-06) | Medium | Misdirected tokens were permanently stuck | Fixed, tested |
| [EVM-07](#evm-07) | Medium | Fee-on-transfer tokens under-funded a quoted order | Fixed (behaviour change), tested |
| [EVM-08](#evm-08) | Low | Contract did not compile against the installed OpenZeppelin | Fixed |
| [BE-01](#be-01) | Critical | A **reverted** Ethereum transaction was accepted as payment | Fixed, unit-tested |
| [BE-02](#be-02) | Critical | A **pending** Ethereum transaction looked deeply confirmed | Fixed, unit-tested |
| [BE-03](#be-03) | High | Every settlement path accepted a 0.5% underpayment | Fixed, unit-tested |
| [BE-04](#be-04) | High | No quote lifecycle: no expiry, no re-approval on price change | Fixed, integration-tested |
| [BE-05](#be-05) | High | Solana confirmation check was a no-op | Fixed, unit-tested |
| [BE-06](#be-06) | Medium | Price oracle invented a price when every source failed | Fixed, unit-tested |
| [BE-07](#be-07) | Medium | Exchange rates had no provenance or staleness check | Fixed, unit-tested |
| [BE-08](#be-08) | Medium | Marketplace bookings discarded their asset id | Fixed, integration-tested |
| [BE-09](#be-09) | Medium | `BookingRepository::update` silently dropped price changes | Fixed, integration-tested |
| [BE-10](#be-10) | Medium | Solana SPL-token payments were mis-measured | Fixed (explicit rejection), unit-tested |
| [BE-11](#be-11) | Low | Solana JSON-RPC errors read as "transaction not found" | Fixed |
| [BE-12](#be-12) | Medium | The browser had no authoritative chain id or token contract | Fixed |
| [FE-01](#fe-01) | Critical | A 3-second timer displayed "Payment Successful" | Fixed |
| [FE-02](#fe-02) | High | No wallet connection and no balance check existed at all | Fixed |
| [FE-03](#fe-03) | High | Submitting a hash auto-created the booking and payment | Fixed |
| [FE-04](#fe-04) | Medium | Offline fallback advertised payment routes as available | Fixed |
| [FE-05](#fe-05) | Medium | Status vocabulary did not describe real settlement states | Fixed |
| [FE-06](#fe-06) | Low | JWT in `localStorage` | **Accepted risk, documented** |
| [OPS-01](#ops-01) | High | Live credentials remain in git history | **Open — rotation required** |

---

## 2. Solana program

Original: `contracts/solana/programs/aeroledger-escrow/src/lib.rs` (422 lines).
Now: same file (rewritten) plus `src/quote.rs`.

### SOL-01
**Escrow had no seller; payouts went to the admin** — Critical

*Location* — `complete_order` / `CompleteOrder` in the original `lib.rs`.

*Root cause.* The `Order` account stored `buyer`, `admin`, `mint`, `amount`. There
was no seller field. `complete_order` moved the entire escrow balance to
`admin_token_account`, the associated token account of whoever signed as `admin`.
The contract therefore modelled a **buyer ⇄ platform** relationship and treated the
administrator as the merchant for every sale. This is exactly the issue raised in
review: for a marketplace, the seller is a party to the trade and must be bound to
the transaction.

*Failure scenario.* A seller lists a jet share. A buyer escrows 50,000 USDC. On
completion every cent lands in the admin's wallet, with nothing on chain tying the
funds to the seller, nothing a seller could point to as evidence of a claim, and no
on-chain distinction between platform revenue and seller proceeds. A compromised or
rogue admin key keeps 100% of every order and the chain shows it as normal
operation.

*Impact.* Total loss of seller funds; no on-chain accountability for a marketplace.

*Fix.* The escrow is now an explicit three-party relationship.

- A `Merchant` PDA (`seeds = [b"merchant", merchant_authority]`) is created by
  governance and records the seller's `authority` and their `payout_wallet`.
- `create_order` **snapshots** `merchant.payout_wallet` and `config.treasury` into
  the `Order` account.
- `complete_order` pays `amount - fee_amount` to the associated token account of
  `order.payout_wallet` and `fee_amount` to the ATA of `order.treasury`. Both are
  pinned by `has_one` on the order plus `associated_token::authority` constraints,
  so no signer — buyer, seller or admin — can substitute a destination.
- Snapshotting matters on its own: re-pointing a merchant's payout wallet after an
  order is funded does **not** retarget money already in escrow.

*Code* — `lib.rs`: `Merchant`, `register_merchant`, `update_merchant`, `Order.payout_wallet`,
`Order.treasury`, `complete_order`, `CompleteOrder`.

*Tests* — `contracts/solana/tests/aeroledger-escrow.ts`: "records the buyer, seller
and both payout destinations on the order", "pays the seller the net amount and the
treasury the fee", "rejects a substituted payout token account", "rejects
confirmation by anyone other than the seller". The equivalent EVM behaviour **is**
executed: see [EVM-01](#evm-01).

*Residual risk.* Governance chooses who a merchant is and where they are paid. A
compromised admin can register a merchant they control — but it cannot redirect an
*existing* order, and merchant registration emits `MerchantUpdated` for monitoring.

### SOL-02
**Buyer chose the amount they paid** — Critical

*Location* — `create_order(ctx, order_id: String, amount: u64)` in the original.

*Root cause.* `amount` was an instruction argument supplied by the caller. The only
validation was `amount > 0`. Nothing on chain connected that number to a price the
platform had quoted.

*Failure scenario.* A buyer does not use the website at all. They build the
instruction directly — the IDL is public, the program id is public — and call
`create_order("<some order id>", 1)`. One base unit is escrowed. The backend sees an
order with a matching id and, in the original design, had no on-chain amount
commitment to check it against. Goods worth $50,000 settle for a fraction of a cent.

*Impact.* Free goods. This is the single most severe issue in the original program.

*Fix.* A **signed quote attestation**, verified by instruction introspection.

1. The backend holds an Ed25519 key, `config.quote_authority`, separate from the
   admin key so it can be rotated cheaply.
2. It signs a canonical message binding every term of the order:
   `"AEROLEDGER_ORDER_QUOTE_V1" || order_id_hash || buyer || seller || mint ||
   amount_le || fee_le || expiry_le` (171 bytes, fixed layout, little-endian).
3. The buyer puts Solana's **native Ed25519 program** instruction in the same
   transaction, immediately before `create_order`. The runtime verifies the
   signature cryptographically and fails the whole transaction if it is invalid.
4. `create_order` reads the Instructions sysvar, requires that the instruction at
   `current_index - 1` is the Ed25519 program, and then checks that what the runtime
   verified is *our* message by *our* signer.

Step 4 is the part that matters. An attacker can put any Ed25519 instruction they
like into a transaction; the program must pin down exactly which signature it is
relying on. `verify_quote_signature_data` therefore rejects: a different signer, a
different message (any field: cheaper amount, different buyer, different seller,
different mint, different expiry), multi-signature instructions, and any instruction
whose signature/pubkey/message offsets point at a *different* instruction
(`signature_instruction_index != u16::MAX`), which would otherwise let bytes be
smuggled in from somewhere this program never inspected.

The quote also carries an expiry, bounded on chain by `MAX_QUOTE_LIFETIME_SECS`
(15 minutes) so even a backend bug cannot mint an indefinitely-valid price, and
replay is prevented because the order PDA can only be initialised once.

*Code* — `contracts/solana/programs/aeroledger-escrow/src/quote.rs`;
`lib.rs::verify_quote_attestation`, `create_order`.

*Tests* — **executed**, `cargo test --lib`, 11 cases in `quote::tests`:
`rejects_a_signature_over_a_cheaper_amount`, `rejects_a_quote_issued_to_a_different_buyer`,
`rejects_a_signature_from_the_wrong_authority`, `rejects_data_borrowed_from_another_instruction`,
`rejects_multi_signature_instructions`, `rejects_zero_signature_instructions`,
`rejects_truncated_and_out_of_bounds_payloads`,
`rejects_a_message_that_is_a_prefix_of_the_expected_one`,
`message_has_the_documented_length_and_layout` and `every_field_changes_the_message`
(which asserts that changing *any* field -- order id, buyer, seller, mint, amount,
fee or expiry -- produces a different message). Plus the Anchor cases
"rejects a buyer who signs a cheaper amount than the order claims" and
"rejects a quote issued to a different buyer (quote theft)" (not executed).

*Residual risk.* `quote_authority` is a hot key. If it leaks, an attacker can
authorise under-priced orders until it is rotated via `set_config`. Mitigations in
place: it is a separate role from `admin`; quotes expire within 15 minutes on chain;
every order emits `OrderCreated` with the amount. Not in place: a per-merchant or
per-period cap on authorised value, which an HSM-backed signer should enforce.

### SOL-03
**`initialize` could be front-run to seize admin** — High

*Root cause.* `initialize(ctx, admin: Pubkey)` had no authorisation at all: the
config PDA is `seeds = [b"config"]` with `init`, so whoever called it first became
the admin of a freshly deployed program.

*Failure scenario.* An attacker watches for the program deployment and lands their
own `initialize` before the deployer's, installing themselves as admin.

*Fix.* `Initialize` now requires the signer to be the program's **upgrade
authority**, proved through the `ProgramData` account:

```rust
#[account(constraint = program.programdata_address()? == Some(program_data.key()))]
pub program: Program<'info, program::AeroledgerEscrow>,
#[account(constraint = program_data.upgrade_authority_address == Some(payer.key()))]
pub program_data: Account<'info, ProgramData>,
```

*Residual risk.* A program deployed as non-upgradeable (`--final`) has no
`ProgramData`, so it can never be initialised. Initialise first, then finalise.

### SOL-04
**Any SPL mint could be escrowed** — High

*Root cause.* `CreateOrder` accepted any `Account<'info, Mint>`.

*Failure scenario.* An attacker creates a worthless mint, escrows a large nominal
quantity of it against an order, and the system treats it as a funded order. More
subtly, a mint with unexpected decimals silently changes what an "amount" means.

*Fix.* A `SupportedMint` PDA (`seeds = [b"mint", mint]`) created by `register_mint`
and required by `CreateOrder`. `register_mint` also snapshots `mint.decimals` for
off-chain consumers. `revoke_mint` de-lists a mint without affecting live orders.

### SOL-05
**Buyer had no exit if the counterparty went silent** — Medium

*Root cause.* Refunds were only possible inside a fixed 24-hour window, and only the
buyer or admin could trigger one. Once that window closed on a `Pending` or
`Confirmed` order, nothing in the program could move the funds except admin action.

*Failure scenario.* The platform confirms an order and then does nothing — through
incompetence, an outage, or malice. After 24 hours the buyer's money is
unrecoverable through any code path.

*Fix.* `timeout_refund`: callable **only** by `order.buyer`, on a `Pending` or
`Confirmed` order, once `refund_deadline` has passed. It is a unilateral escape
hatch that no other party can block. The refund window is now governance-configured
within hard bounds (`MIN_REFUND_WINDOW_SECS` 1 hour .. `MAX_REFUND_WINDOW_SECS`
30 days) rather than a bare constant.

*Policy now in force* — the question "is 24 hours right?" is answered explicitly:
- `Pending` — buyer, seller or admin may refund (the seller has not committed yet).
- `Confirmed` — seller or admin only; the buyer cannot unilaterally cancel work the
  seller has accepted.
- After `refund_deadline` — the buyer, and only the buyer, may `timeout_refund`.

*Tests* — the identical EVM policy **is** executed; see "refunds and the buyer's
timeout exit" (10 cases) in the Hardhat suite.

### SOL-06
**Escrow vault rent was stranded permanently** — Medium

The escrow associated token account was created with `init` (paid for by the buyer)
and never closed. Every completed order left its rent locked forever.

*Fix.* `complete_order`, `refund_order` and `timeout_refund` all end by
`token::close_account`-ing the vault with the order PDA as authority and the
**buyer** as the rent destination. The payout maths is arranged so the vault always
lands on exactly zero: the seller receives precisely the quoted net amount, and any
surplus — an unsolicited donation into the vault — sweeps to the treasury.

*Deliberate non-change.* The `Order` account itself is **not** closed. Closing it
would free the PDA and allow `create_order` to re-initialise the same `order_id`,
re-opening a settled order. Keeping the account is the replay guard; the rent
(~0.003 SOL per order) is the cost of it.

### SOL-07
**No pause** — Medium

*Fix.* `Config.paused`, set by `set_paused`, blocks `create_order` only. Settlement,
refunds and `timeout_refund` stay available while paused: a pause must never be able
to trap customer funds. The EVM equivalent is tested
("blocks new orders but still allows settlement and refunds").

### SOL-08
**Admin rotation would have bricked confirmation** — Medium

`ConfirmOrder` required both `config.has_one = admin` and `order.has_one = admin`,
where `order.admin` was a snapshot taken at creation. Rotating `Config.admin` would
make every in-flight order permanently unconfirmable.

*Fix.* Roles were redesigned. `Order` no longer stores an admin. Confirmation is the
seller's action (`has_one = seller`), and completion accepts `order.seller` or the
*current* `config.admin`.

### SOL-09
**No events** — Medium

*Fix.* `ConfigUpdated`, `MintRegistered`, `MintRevoked`, `MerchantUpdated`,
`OrderCreated`, `OrderConfirmed`, `OrderCompleted`, `OrderRefunded` (with a
`timed_out` flag). Each carries the `order_id_hash` and the amounts needed to drive
an off-chain indexer. No secrets are logged.

### SOL-10
**Token-2022 behaviour undefined** — Low

*Fix.* Explicitly unsupported, and now fail-closed by construction:
`Program<'info, Token>` plus `anchor_spl::token::{Mint, TokenAccount}` pin every
account to the legacy SPL Token program, so a Token-2022 mint fails
deserialization. This is documented at the top of `lib.rs` with the reason:
transfer fees, transfer hooks and confidential transfers each break the
"what went in is what comes out" accounting, and supporting them safely needs
per-extension analysis that has not been done.

### SOL-11
**Single-step admin handover** — Low

*Fix.* `transfer_admin` nominates, `accept_admin` requires the nominee to sign. A
mistyped address is recoverable.

### SOL-12
**Program id is still the Anchor placeholder** — Info, **OPEN**

`declare_id!("Fg6PaFpoGXkYsidMpWxTWqK6W2BeZ7FEfcYkgqPLF6j")` and all three
`Anchor.toml` entries are the template placeholder. This is a deployment blocker by
design — no real keypair has been generated or committed, which is correct. Before
any deploy: `anchor keys sync`, which derives the id from
`target/deploy/aeroledger_escrow-keypair.json` and rewrites both files. That keypair
must never be committed.

### Solana account sizing (§4.8 of the brief)

`Order::SPACE` was hand-computed in the original. It is now derived with Anchor's
`#[derive(InitSpace)]` and `space = 8 + Order::INIT_SPACE`, and **pinned by an
executed test** (`account_sizes_match_their_field_layout`) that asserts the derived
number equals the hand-audited field layout for all four account types. `order_id`
is capped at 64 bytes and validated in the handler before the account is allocated,
so uncontrolled input cannot cause an allocation failure. `OrderStatus` implements
`Space` explicitly (one discriminant byte) rather than relying on a derive macro's
treatment of enums.

---

## 3. EVM contract

`contracts/ethereum/contracts/AeroLedgerEscrow.sol`.

### EVM-01
**Payouts went to `owner()`; no seller existed** — Critical

Identical in substance to [SOL-01](#sol-01): `completeOrder` did
`_payout(token, owner(), amount)`.

*Fix.* `merchants[seller] = { payoutWallet, active }`, set by governance. Orders
record `seller`, `payoutWallet` and `treasury` at creation. `completeOrder` pays
`amount - fee` to `order.payoutWallet` and `fee` to `order.treasury`.

*Tests — executed:*
- "pays the seller's registered wallet and the treasury, never the owner" — asserts
  the exact ETH balance deltas of `sellerPayout`, `treasury` and the escrow, **and**
  that `owner`'s balance is unchanged.
- "rejects confirmation by anyone other than the seller" — owner, buyer and a random
  account all revert with `NotAuthorized`.
- "rejects an order against an unregistered or deactivated seller".

### EVM-02
**Buyer chose `msg.value`** — Critical

*Fix.* Every order carries an EIP-712 `Quote` signed by `quoteSigner`, binding
`(orderHash, buyer, seller, token, amount, fee, expiry)`. `createOrder` requires
`msg.value == amount`; `createOrderERC20` requires the measured balance delta to be
at least `amount`. The EIP-712 domain includes the chain id and the contract
address, so a signature is useless on another chain or another deployment.

*Tests — executed, 11 cases* under "server-authoritative pricing (signed quotes)":
a tampered signature, a signature by the wrong key, paying less than the signed
amount, **editing the amount in the calldata while holding a genuine signature for a
different amount**, replaying another buyer's quote, re-pointing a quote at a
different seller, an expired quote, an over-long quote lifetime, a zero amount, a
fee that swallows the order, replaying the same quote twice, and a
**cross-chain-replay** case that signs with a foreign chain id.

### EVM-03
**Payout followed the live owner/treasury** — Medium

*Root cause.* The original read `owner()` at settlement time, so an ownership
transfer between funding and completion silently retargeted money already in escrow.

*Fix.* Snapshot into the order at creation.

*Tests — executed:* "keeps paying the wallet recorded at creation even if the
merchant is re-pointed" and "keeps paying the treasury recorded at creation even if
it is re-pointed" — both re-point governance mid-flight and assert the attacker
receives exactly 0.

### EVM-04
**No pause** — Medium. *Fix.* OpenZeppelin `Pausable` on the two creation
functions only. *Test — executed:* "blocks new orders but still allows settlement
and refunds".

### EVM-05
**Single-step ownership transfer** — Medium. *Fix.* `Ownable2Step`.
*Test — executed:* "requires a two-step handover to change the owner".

### EVM-06
**Misdirected tokens were permanently stuck** — Medium

*Fix.* `unaccountedBalance(token)` = held − `escrowBalance[token]`, and
`sweepUnaccounted` can move **only** that difference. Escrowed funds are
mathematically out of reach of the sweep.

*Test — executed:* "can sweep only funds that are not backing a live order" —
force-sends tokens to the contract, asserts the escrowed native balance is *not*
sweepable (`EscrowUnderflow`), sweeps the surplus, then proves the live order is
still fully refundable afterwards.

### EVM-07
**Fee-on-transfer tokens under-funded a quoted order** — Medium
**(deliberate behaviour change)**

The previous version credited whatever actually arrived. With a server-signed price
that is the vulnerability: a token that burns 1% on transfer means the escrow holds
1% less than the price the seller agreed to, on every order, silently.

*Fix.* `_pullTokens` reverts with `AmountMismatch` when the measured delta is below
the quoted amount. Fee-on-transfer tokens are therefore unusable — correctly, since
the owner simply should not allow-list one.

*This removes previously-working behaviour.* It is called out here because the brief
asks for no silent removals. The old test "credits the actual received amount, and
refunds exactly that" is replaced by "rejects a fee-on-transfer token rather than
under-funding a signed quote". Surplus (a rebase upward) is still credited.

### EVM-08
**Contract did not compile against the installed OpenZeppelin** — Low

OpenZeppelin 5.6.1 requires `^0.8.24` and uses `MCOPY`; the Hardhat config pinned
solc 0.8.20 with no `evmVersion`. *Fix.* solc 0.8.24 with `evmVersion: "cancun"`,
and a comment recording that Ethereum (Dencun) and BSC (Haber/Bohr) both enable it
and that any additional target chain must be checked before deploying this bytecode.

### EVM — retained properties

Already correct before this round and preserved, with tests: strict native/ERC-20
path separation and the per-asset `escrowBalance` ledger (so a tiny native deposit
can never drain the pooled USDC — test executed), `SafeERC20` throughout including
a USDT-style token that returns no bool, checked low-level `.call` for native
payouts so a multisig payout wallet is not bricked by the 2300-gas stipend,
checks-effects-interactions plus `nonReentrant`, and a reentrancy test driving a
malicious buyer contract that tries to re-enter `refundOrder` from `receive()`.

### EVM — upgrade safety

This is a **new deployment**, not an upgrade. The contract is not proxied and the
storage layout has changed incompatibly (`Order` gained four fields). Any existing
deployment must be drained through its own refund paths before the new one is used.
No claim of upgrade compatibility is made.

---

## 4. Rust backend

### BE-01
**A reverted Ethereum transaction was accepted as payment** — Critical

*Location* — `src/infrastructure/blockchain/evm_gateway.rs::verify_native`.

*Root cause.* Verification used `eth_getTransactionByHash` only, and read the
transaction's `value` field. That field is what the sender *asked* to transfer. It
is present and unchanged on a transaction that reverted and moved nothing.

*Failure scenario.* An attacker sends a transaction to a contract they control that
takes the ETH and then reverts — or simply a transaction that runs out of gas. No
value moves. They submit the hash. The backend reads `value: 0x...`, sees the
`to` address matches, counts confirmations, and settles the order. **Goods for
free, and the chain record shows a failed transaction.**

*Fix.* `receipt_status()` fetches `eth_getTransactionReceipt` **first** and requires
`status == 0x1`. If the transaction did not succeed, `base_units` is forced to `0`
and `succeeded = false` is threaded into `decide()`, which now requires it.

*Tests — executed:* `a_reverted_transaction_is_never_verified`.

### BE-02
**A pending Ethereum transaction looked deeply confirmed** — Critical

*Root cause.* `block_from_hex` returned `0` when `blockNumber` was `null` — which is
exactly what a **pending** transaction returns. Confirmations were then computed as
`current_block - 0`, i.e. around 21 million. The `confirmations >= 12` check passed
trivially.

*Failure scenario.* An attacker broadcasts a transaction with a deliberately low gas
price (or to a private mempool), immediately submits the hash, and the backend
settles the order. They then replace the transaction with a 0-value self-send at a
higher nonce fee, and it never mines. The order is confirmed; no money moved.

*Fix.* `mined_block()` returns `Option<u64>` and treats both `null` and `0x0` as
"not mined"; the caller errors out rather than defaulting. `confirmations_for()`
counts the mining block as one and returns `0` if the node's tip lags the
transaction, so it can never underflow into a large number.

*Tests — executed:* `a_pending_transaction_has_no_mined_block`,
`confirmations_count_the_mining_block_and_never_underflow`,
`too_few_confirmations_is_rejected`.

### BE-03
**Every settlement path accepted a 0.5% underpayment** — High

*Root cause.* Five independent places computed
`tolerance = expected_base_units / 200` and accepted `received + tolerance >=
expected`: the EVM gateway, the Solana gateway, the Bitcoin gateway,
`ConfirmPayment` and the reconciliation worker.

*Failure scenario.* It is not an attack so much as a standing leak: 0.5% of every
order, forever, and a buyer who notices can simply always send 99.5%. On a $50,000
jet share that is $250 per order.

*Fix.* Exact-or-more everywhere: `received >= expected_base_units`. The justification
for the tolerance — rounding — does not apply, because `usd_to_crypto_amount`
already rounds the quote **up** to the token's precision, so the payer is told the
exact figure.

*Tests — executed:* `an_underpayment_of_one_base_unit_is_rejected` (gateway),
`rejects_a_payment_one_base_unit_short_of_the_quote` (use case),
`accepts_an_overpayment`.

### BE-04
**No quote lifecycle: no expiry, no re-approval on price change** — High

This is the interviewer's second question, and the original answer was incomplete. A
previous round had already removed client-supplied prices (the booking price is read
from the catalogue). What did **not** exist: an expiry, detection of a price change
between browsing and paying, any notion of the user approving a specific figure, or
a lock once they had.

*Failure scenario.* A customer opens checkout at $5,000. The provider re-prices to
$5,500. The customer pays. Under the old flow the booking still carried $5,000, so
the platform under-collected by $500 — or, had the booking re-read the catalogue,
the customer would have been charged $5,500 without ever seeing it. Separately, a
crypto quote computed at initialisation had no expiry at all: a user could return
days later and pay at a rate that had moved 40%.

*Fix.* A full quote lifecycle. New table `payment_quotes`; entity
`src/domain/entities/quote.rs`; use cases `create_quote.rs` and `accept_quote.rs`;
endpoints `POST /api/payments/quote` and `POST /api/payments/quote/accept`.

The rules, each enforced in code:

1. **Server owns the price.** Every quote is re-derived from the catalogue at issue
   time. The client sends identifiers, never amounts.
2. **Quotes expire.** 600 seconds (`QUOTE_TTL_SECONDS`). An expired quote cannot be
   accepted and marks itself `Expired`.
3. **Acceptance echoes the numbers.** The client must send back the exact
   `approved_total_cents` **and** `approved_crypto_base_units`. Either differing →
   `409 Conflict` with the new figures, and the user must approve again.
4. **Acceptance locks the amount.** The base-unit figure becomes the settlement
   target; later FX movement cannot change what is owed.
5. **One live quote per booking.** Issuing supersedes the previous one in the same
   DB transaction, backed by a partial unique index
   (`idx_payment_quotes_one_active`), so an old cheaper quote cannot be replayed.
6. **Payment is impossible without an accepted quote.** `Payment::from_quote` is the
   *only* constructor, and it refuses a quote that is not `Accepted`. There is no
   code path that creates a payment for a price nobody approved.
7. **A re-price after acceptance voids it.** `InitPayment` re-checks that the
   accepted quote's total still equals the booking's.

*Tests — executed against real Postgres:*
`a_price_change_between_booking_and_checkout_is_surfaced_and_must_be_re_approved`
(re-prices a flight $5,000 → $5,500 mid-checkout, asserts `price_changed`, asserts
the old figures are rejected, asserts the superseded quote is dead, approves the new
one and confirms $6,325.00 is what gets locked), `an_expired_quote_cannot_be_accepted`,
`only_one_quote_per_booking_is_ever_active`,
`a_quote_cannot_be_accepted_by_another_user`,
`another_user_cannot_quote_someone_elses_booking`,
`a_booking_cannot_be_re_quoted_once_payment_is_initialised`,
`quote_accept_pay_confirm_end_to_end`.
Plus 11 executed unit tests in `domain::entities::quote::tests`.

*Price-lock policy, stated explicitly.* An unaccepted quote lives 10 minutes. Once
accepted, the token amount is locked for that payment indefinitely — the platform
carries the FX risk from acceptance to settlement, deliberately, because the
alternative is telling a customer whose transaction is already on chain that they
now owe more. The exposure is bounded by the fact that the customer has already
signed; it is not bounded in time, which is a **known residual risk**.

### BE-05
**Solana confirmation check was a no-op** — High

*Root cause.* `getTransaction` was called with no explicit commitment, and
confirmations were synthesised from a slot delta with
`.max(if succeeded { 1 } else { 0 })` — which forced the count to at least 1 whenever
the transaction succeeded, making `confirmations >= MIN_CONFIRMATIONS` (1) always
true.

*Fix.* `"commitment": "finalized"` is passed explicitly, so the RPC only returns the
transaction once it cannot be rolled back. The synthetic slot arithmetic is gone.
A transaction the RPC does not return yet is reported as unverified with zero
confirmations so the reconciliation worker keeps retrying, rather than erroring.

### BE-06
**Price oracle invented a price when every source failed** — Medium

*Root cause.* `static_floor()` returned a hard-coded price (ETH = $3,000) whenever
both CoinGecko and the database failed, in **all** modes including live.

*Failure scenario.* CoinGecko rate-limits during a market move. ETH is actually
$4,200. Every customer is quoted at $3,000/ETH and pays ~29% less than the listed
price, and the platform's own logs show nothing but a warning.

*Fix.* Fail closed in live mode: `DomainError::InternalError` and no quote. The
static floor survives only outside live mode, labelled `static-floor-dev`.

### BE-07
**Exchange rates had no provenance or staleness check** — Medium

*Root cause.* `PriceOracle::usd_price` returned a bare `Decimal`. A cached value,
a 3-day-old database row and a live quote were indistinguishable.

*Fix.* `TokenRate { unit_price_usd, source, observed_at }`. The DB path reads the
row's own `last_updated` as the observation time. The cache preserves the original
`observed_at`, so caching can never make a stale price look fresh. `PaymentQuote::issue`
refuses any rate older than `MAX_RATE_AGE_SECONDS` (300s). The source and
observation time are stored on the quote and shown in the UI.

*Tests — executed:* `rejects_a_quote_built_from_a_stale_rate`.

### BE-08
**Marketplace bookings discarded their asset id** — Medium

`CreateBooking` received `asset_id` and never stored it — `bookings` had no such
column. An asset booking recorded only a price, with no reference to what was bought.

*Impact.* Marketplace bookings could not be re-priced at checkout at all, so the
whole price-change protection in [BE-04](#be-04) would have been inert for them.

*Fix.* `ALTER TABLE bookings ADD COLUMN IF NOT EXISTS asset_id UUID`; `Booking.asset_id`;
persisted on insert and read back in every projection.

### BE-09
**`BookingRepository::update` silently dropped price changes** — Medium

`update` wrote only `status` and `tx_hash`. Re-pricing a booking in memory and
calling `update` would have persisted nothing, leaving the stored booking and the
approved quote in disagreement about what was owed.

*Fix.* `update` now writes `base_price` and `total_price` too. Covered by the
executed re-pricing integration test, which reads the booking back through
`InitPayment`.

### BE-10
**Solana SPL-token payments were mis-measured** — Medium

The gateway measured native lamport balance deltas (`preBalances`/`postBalances`).
Those are SOL only. An SPL-token transfer (USDC on Solana) would show a *negative*
native delta for the fee payer and nothing for the recipient, so such payments could
never verify — but the code did not say so, it just silently returned zero.

*Fix.* The gateway rejects any `VerifyRequest` carrying a `token_contract` with an
explicit error. Supporting SPL tokens properly means parsing
`meta.preTokenBalances`/`postTokenBalances` for the right mint *and* owner; that is
deliberately not approximated.

*Also fixed here.* The recipient is located **by address** in the account-key list,
never by a fixed index, and the object form of `accountKeys` is handled.

*Tests — executed, 6 cases:* `credits_the_recipients_own_balance_delta`,
`locates_the_recipient_by_address_not_by_index`, `a_failed_transaction_credits_nothing`,
`a_transaction_that_never_touches_our_wallet_does_not_match`,
`handles_the_object_form_of_account_keys`,
`a_balance_decrease_is_never_read_as_a_credit`.

### BE-11
**Solana JSON-RPC errors read as "transaction not found"** — Low

A JSON-RPC error body arrives with HTTP 200 and a null `result`. *Fix.* The `error`
member is checked and surfaced.

### BE-12
**The browser had no authoritative chain id or token contract** — Medium

`/api/payments/options` returned only a label and an address, so any client-side
network or token-contract knowledge would have had to be hard-coded in the bundle —
the classic route to "right address, wrong chain".

*Fix.* The endpoint now serves `wallet_kind`, `chain_id` (from `ETHEREUM_CHAIN_ID` /
`BSC_CHAIN_ID` config), `token_contract` and `decimals`, all derived from the same
`resolve_chain_token` source of truth the verifier uses.

### Backend — properties verified and retained

These were already correct and are confirmed by executed tests rather than assumed:

- **Replay protection.** A transaction hash cannot settle two bookings — enforced in
  the use case, in `PgSettlementRepository` inside the transaction, and by a partial
  unique index on `payments(tx_hash)`. Test: `a_transaction_hash_cannot_settle_two_bookings`.
- **Idempotency.** Confirming the same payment three times produces exactly one
  ledger row. Test: `confirming_the_same_payment_twice_settles_once`.
- **Transactional settlement.** `confirm_payment` locks the payment and booking rows
  (`FOR UPDATE OF p, b`), re-checks status, writes payment + booking + transaction,
  and commits once.
- **Authorisation.** Every booking/payment/quote route resolves the actor from the
  JWT and compares it with the booking's owner; admins are the only exception.
  Tests: `a_quote_cannot_be_accepted_by_another_user`,
  `another_user_cannot_quote_someone_elses_booking`.
- **Config fails closed.** Production refuses to start with a placeholder JWT
  secret, `manual` verification mode, migrations enabled, a localhost frontend
  origin, or a malformed wallet address. 7 executed tests.
- **Rate limiting and CORS** are configured in `main.rs` with a proxy-aware key
  extractor; the auth middleware's public-path matching is boundary-aware.

---

## 5. Frontend

### FE-01
**A 3-second timer displayed "Payment Successful"** — Critical

*Location* — `Frontend/src/app/components/PaymentModal.tsx`:

```ts
const handlePay = () => {
  setStep("processing");
  setTimeout(() => setStep("success"), 3000);   // <- settlement by stopwatch
};
```

Nothing had been priced, no wallet had been touched, no chain had been consulted.
The modal then rendered a green tick and "Your purchase of X has been confirmed."

*Impact.* A customer is told they have bought something when no money has moved.
Beyond the obvious dispute exposure, it is the kind of thing that makes every other
status in the product untrustworthy.

*Fix.* The simulated flow is deleted outright. The modal now summarises the item,
labels the catalogue figure as indicative, explains that the amount is quoted by the
backend and verified on-chain, and routes to the real checkout. There is no success
state in it at all.

### FE-02
**No wallet connection and no balance check existed** — High

This is the interviewer's third question, and the answer was simply that it did not
exist: the dependency tree contained no wallet library, and nothing read a balance.

*Fix.* `Frontend/src/app/lib/wallet.ts` and `components/CheckoutQuotePanel.tsx`:

- Connect/disconnect for EVM (EIP-1193) and Solana (injected provider).
- Connected address shown, truncated; live chain id displayed.
- Native balance via `eth_getBalance`; **ERC-20 balance via `eth_call` to
  `balanceOf(address)`** using the contract the *backend* supplied.
- Solana lamports via `getBalance` at `confirmed`.
- **All comparisons in `bigint` base units.** No floats touch a balance check.
- Gas/fee headroom: 0.002 ETH on EVM, 0.003 SOL on Solana (covering the signature
  fee and one rent-exempt account), with a separate "native balance looks thin"
  warning on ERC-20 routes.
- Insufficient balance shows the required amount, the available amount and the exact
  shortfall, and **disables the approve button**.
- Wrong network is detected against the server-supplied chain id, blocks approval,
  and offers `wallet_switchEthereumChain` — after which the chain id is **re-read**
  rather than assumed, because a user can dismiss the prompt.
- `accountsChanged` / `chainChanged` / `accountChanged` / `disconnect` all
  invalidate the reading.
- The balance is **re-read immediately before the approve call**, not only on load.
- Bitcoin is explicitly marked unsupported for balance checking (no browser wallet
  standard) rather than pretending.

The UI states plainly, in the panel itself, that this is a pre-flight convenience
and that the chain and the backend are what actually enforce payment.

*Why no wallet SDK.* `window.ethereum` and `window.solana` are stable injected
standards. wagmi/viem or `@solana/wallet-adapter` would add a large dependency tree
to a React 18 + Vite app for connect-and-read-balance. The trade-off — no
WalletConnect, no mobile deep-linking, no multi-wallet modal — is a real limitation
and is listed in [§7](#7-what-has-not-been-verified).

### FE-03
**Submitting a hash auto-created the booking and payment** — High

`handleSubmitHash` called `ensurePaymentInitialized()`, which created a booking
*and* initialised a payment if none existed. A user could skip the entire pricing
step and go straight to "Go to Hash Submission"; the amount would be whatever the
server happened to compute, never displayed, never approved.

*Fix.* `ensureBooking()` creates a booking and nothing else. Submitting a hash or a
proof now requires an initialised payment, which requires an accepted quote. The
button is disabled and labelled "Approve the price first" until then.

### FE-04
**Offline fallback advertised payment routes as available** — Medium

`FALLBACK_PAYMENT_OPTIONS` had `available: true` with `wallet_address: null`, so a
backend outage produced a checkout that looked payable.

*Fix.* Every fallback entry is `available: false` with no address and no chain id,
and the quote panel refuses to quote an unavailable route with an explicit message.

### FE-05
**Status vocabulary did not describe real settlement states** — Medium

Three states (`initialized`/`awaiting`/`confirmed`) with `confirmed` set optimistically
on a resolved promise.

*Fix.* `CheckoutPhase` — thirteen states covering quote requested/ready/expired,
price changed, price locked, awaiting wallet, insufficient balance, wrong network,
transaction submitted, pending confirmation, payment verified, failed. Crucially a
backend rejection containing "not verified yet" maps to **`pending_confirmation`**,
not `failed`, because the reconciliation worker will settle it later — and
`payment_verified` is reachable only from a successful `confirmPayment` response,
which the backend only returns after checking the chain.

### FE-06
**JWT in `localStorage`** — Low, **accepted risk**

The session token is in `localStorage`, readable by any script on the origin, so an
XSS becomes a full session takeover. The alternative — an `HttpOnly`, `Secure`,
`SameSite` cookie plus CSRF tokens — needs backend session work and a CORS/credential
redesign that was out of scope for this round.

*Partial mitigations in place:* the token is never logged; expiry is checked
client-side and the session cleared on `exp`; a 401 on an authenticated request
clears the session and fires an auth event; React escapes by default and there is no
`dangerouslySetInnerHTML` in the app code; a strict CORS allow-list is enforced
server-side.

*Not in place:* a Content-Security-Policy header, and cookie-based sessions. Both
are recommended before handling real funds.

---

## 6. Operational

### OPS-01
**Live credentials remain in git history** — High, **OPEN, rotation required**

`.env.production` was removed from the working tree in commit `d9889fd`
("Remove sensitive data from .env.production"). **That does not remove it from
history.** The file is still retrievable at `60a242a`, `6b9c6c8` and `fa64679`.

Inspected without printing values, commit `60a242a` contains:

| Variable | Assessment |
|---|---|
| `DATABASE_URL` | 63 characters, not a placeholder — **contains live database credentials** |
| `ETHERSCAN_API_KEY` | 34 characters, not a placeholder — **live API key** |
| `JWT_SECRET` | 57 characters, contains a placeholder marker, but must be treated as compromised |

Wallet addresses were also present; those are public by nature and are not a secret,
though they do disclose the treasury.

**Required actions, none of which this work can perform:**

1. Rotate the database credentials and the Etherscan key **now**. Anyone who has
   ever cloned this repository has them.
2. Rotate `JWT_SECRET`. Every existing session token becomes invalid, which is the
   point.
3. Decide whether to purge history (`git filter-repo`, then force-push and have every
   collaborator re-clone) — note that this rewrites every commit hash and that
   GitHub may retain unreachable objects, so **rotation is mandatory regardless**.
4. Add secret scanning to CI.

`.gitignore` already covers `.env` and `.env.production`, and `.env.example` now
carries only `CHANGE_ME` placeholders, so the ongoing leak is closed — the
historical one is not.

### OPS-02 — Payment-proof uploads

`POST /api/payments/proof/{booking_id}` writes under
`uploads/payment_proofs/<booking_id>/`. Reviewed and found sound: the filename is a
server-generated UUID plus an extension chosen from a fixed allow-list (the
client-supplied filename is never used as a path component, so traversal is not
reachable), the booking id is a parsed `Uuid`, the content type must be `image/*`,
the size is capped at 5 MB with the partial file deleted on overflow, ownership is
checked before anything is written, and a settled payment refuses further proof.

*Residual:* the files are not virus-scanned and are served from the same origin if a
static handler is ever added — they should be served from a separate origin with
`Content-Disposition: attachment`.

---

## 7. What has NOT been verified

Stated plainly, because the value of the rest depends on this list being honest.

1. **The Anchor integration tests have not been run.** The Anchor CLI and Solana
   toolchain are not installed here (`anchor`, `solana` absent from `PATH`), and no
   local validator is available. The suite is written
   (`contracts/solana/tests/aeroledger-escrow.ts`) and covers quote forgery, account
   substitution, PDA seed mismatch, authorisation, state-machine violations, double
   completion, double refund, timeouts and pause. To run it:

   ```bash
   # install: https://www.anchor-lang.com/docs/installation
   cd contracts/solana
   yarn install
   anchor keys sync      # SOL-12: replace the placeholder program id first
   anchor test
   ```

   What **is** executed for the Solana program: 14 unit tests covering the entire
   quote-attestation parser and the account-size calculation. The constraint
   expressions themselves are type-checked by `cargo check` but their *runtime*
   behaviour is unproven here. The equivalent business rules are executed on the EVM
   side, which is the closest available evidence.

2. **Neither contract has been deployed, to any network.** No program keypair, no
   contract address, no gas measurements. The Solana program id is still the Anchor
   placeholder ([SOL-12](#sol-12)).

3. **The on-chain escrow is not yet wired into the backend.** This is the most
   important structural caveat. The backend currently implements a
   **direct-payment** architecture: quote → accept → deposit to a platform-controlled
   address → verify the transfer on chain → settle. The escrow contracts implement a
   **custodial escrow** architecture with a seller and a fee split. Both are
   coherent; they are not yet the same flow. What connects them today is the shared
   quote concept, and nothing else. See `ARCHITECTURE.md` §"Two payment paths" for
   exactly what works end to end and what the integration would involve. No claim is
   made that an order placed through the website is settled by the escrow program.

4. **No end-to-end browser run.** The frontend type-checks and builds for
   production, and the backend endpoints it calls are integration-tested against a
   real Postgres, but no browser session has been driven through the full flow with
   a live wallet.

5. **The wallet integration has not been tested against a real wallet extension.**
   The code paths are written against the EIP-1193 and Solana provider interfaces;
   no MetaMask or Phantom session has exercised them here.

6. **No live chain-provider verification.** The gateway logic is unit-tested against
   recorded response shapes, not against Etherscan, a Solana RPC or Blockstream.

7. **Third-party review.** None of this is a substitute for an independent audit
   before real funds are handled.

8. **Dependency CVEs were not scanned.** `cargo audit` and `npm audit` were not run
   in this environment; both should be CI gates.

---

## 8. Residual risks, ranked

| Risk | Why it remains | Suggested mitigation |
|---|---|---|
| `quote_authority` is a hot key | The signing design needs an online signer | HSM/KMS signer; per-period value caps; alerting on `OrderCreated` amounts |
| Admin can deactivate merchants and refund live orders | Operationally necessary | Multisig/timelock the admin; all actions emit events |
| Post-acceptance FX exposure | Deliberate: the alternative is re-charging a customer mid-payment | Cap the settlement window; hedge; or require faster settlement |
| Stablecoin de-peg between quote and settlement | USDT/USDC are quoted at the peg by policy | Quote stablecoins from the oracle too, or add a de-peg circuit breaker |
| JWT in `localStorage` ([FE-06](#fe-06)) | Cookie sessions need a backend redesign | `HttpOnly` cookies + CSRF; add a CSP |
| Credentials in git history ([OPS-01](#ops-01)) | Cannot be undone from here | Rotate now; then decide on history rewriting |
| Escrow contracts not wired to the backend | Architectural work beyond this round | See `ARCHITECTURE.md` |
| Anchor tests unexecuted | Toolchain unavailable here | Run `anchor test` on a machine with the toolchain |
