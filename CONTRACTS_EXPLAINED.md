# The Escrow Contracts, Explained Line by Line

Two implementations of the same escrow, one per chain:

| | Solana | EVM |
|---|---|---|
| Source | `contracts/solana/programs/aeroledger-escrow/src/{lib.rs,quote.rs}` | `contracts/ethereum/contracts/AeroLedgerEscrow.sol` |
| Size | 1,302 + 400 lines | 531 lines |
| Framework | Anchor | Solidity 0.8.24 + OpenZeppelin 5 |
| Tests | 14 unit (executed) + Anchor suite (**not** executed) | 52 (executed) |

**Status, stated first every time:** both are written and tested. **Neither is wired
into the backend.** The live payment path is direct payment with on-chain
verification. Don't let that come out as a surprise mid-interview.

---

## Part 0 — What both contracts are defending against

Before any code: there are exactly **four** things an escrow like this must get
right. Everything in both files is one of these four, and if you can name them you
can derive most of the code from first principles.

1. **The buyer must not be able to choose the price.** If `amount` is just an
   argument, anyone who can call the contract directly pays one wei for a jet. This
   is why both contracts carry a *signed quote*.
2. **No signer may choose where money goes.** Not the buyer, not the seller, not the
   admin. This is why payout destinations are **snapshotted into the order at
   creation** from governance-controlled state, and why settlement takes no
   destination argument at all.
3. **A buyer's funds must never be trappable.** If the counterparty goes silent, the
   buyer needs a unilateral exit. This is `timeout_refund` / `timeoutRefund`.
4. **What goes in must equal what comes out.** Per-asset accounting, exact integer
   arithmetic, no path that pays out more than was deposited for that asset.

The original version of this escrow failed (1) and (2) outright: it had no seller at
all, so `complete_order` paid the admin, and the buyer passed the amount as an
instruction argument.

---

# Part 1 — The Solana program

## 1.1 Module header and the trust model

The file opens with a table of the six roles. Learn this — it is the first question.

| Party | On-chain identity | Role |
|---|---|---|
| Buyer | `Order.buyer` (signs `create_order`) | Funds the vault, receives refunds |
| Seller | `Merchant.authority` | Accepts the order, receives net payout |
| Payout wallet | `Merchant.payout_wallet` | Where proceeds land (its ATA) |
| Platform | `Config.treasury` | Receives the quoted fee |
| Admin | `Config.admin` | **Governance only** |
| Quote signer | `Config.quote_authority` | Ed25519 key authorising each price |

The two lists that follow — what the admin **can** and **cannot** do — are the
honest statement of the trust model. The "cannot" list is the valuable half:

> The admin cannot choose a payout or refund destination, change a live order's
> amount, confirm an order on a seller's behalf, take funds by any other path, or
> block a buyer's exit forever.

And the residual risk is stated rather than hidden: a compromised admin can
deactivate merchants and refund live orders — **denial of service, not theft**. A
compromised `quote_authority` can authorise under-priced orders, which is exactly
why it is a separate, rotatable role.

### Token support — a fail-closed decision

```rust
// SPL Token only. `Program<'info, Token>` plus `anchor_spl::token::{Mint,
// TokenAccount}` pin every token account to the legacy SPL Token program, so
// Token-2022 mints are rejected at deserialization.
```

This is worth being able to defend. Token-2022's transfer fees, transfer hooks and
confidential transfers each break "what went in is what comes out". Supporting them
safely needs per-extension analysis that hasn't been done, so they are rejected at
the type level — not by a runtime check someone can forget.

## 1.2 Constants

```rust
declare_id!("Fg6PaFpoGXkYsidMpWxTWqK6W2BeZ7FEfcYkgqPLF6j");
```

**This is the Anchor template placeholder and it is a deployment blocker by design.**
No real keypair has been generated or committed — which is correct, because the
keypair must never be committed. Before any deploy: `anchor keys sync`. Logged as
SOL-12, open.

```rust
pub const MAX_ORDER_ID_LEN: usize = 64;
```

A UUID is 36 chars; 64 leaves room. **The string is not the PDA seed** — a seed is
capped at 32 bytes. The seed is `keccak256(order_id)`, a fixed 32 bytes, and the
full string is stored in account data so off-chain indexers can correlate. If you
are asked "why hash the order id?", that is the answer: *seed length*, not secrecy.

```rust
pub const MIN_REFUND_WINDOW_SECS: i64 = 3_600;        // 1 hour
pub const MAX_REFUND_WINDOW_SECS: i64 = 30 * 86_400;  // 30 days
pub const MAX_QUOTE_LIFETIME_SECS: i64 = 900;         // 15 minutes
```

Bounds on an admin-configurable value, so governance cannot set a window so long
that buyers claw back settled business, nor so short that buyers have no exit.
`MAX_QUOTE_LIFETIME_SECS` bounds the blast radius of a **leaked** signed quote: even
if the backend signs something with a far-future expiry, the program refuses it.

A unit test pins the relationship that actually matters:

```rust
assert!(MAX_QUOTE_LIFETIME_SECS < MIN_REFUND_WINDOW_SECS);
```

> A quote must die well before the refund window closes. Otherwise a quote could
> still be usable at a moment when the order it creates is already past refundable —
> a buyer could be locked into a fresh order with no exit.

## 1.3 Governance instructions

### `initialize` — gated on the upgrade authority

```rust
#[account(constraint = program_data.upgrade_authority_address == Some(payer.key()))]
pub program_data: Account<'info, ProgramData>,
```

The subtle part: **not** "whoever calls first". A freshly deployed program sits
un-initialised for a few blocks, and an attacker watching the mempool could call
`initialize` first and install themselves as admin. Tying it to the program's
upgrade authority — the key that deployed it — closes that window. (SOL-03.)

*Known consequence:* if the program is made immutable before `initialize` runs,
`upgrade_authority_address` is `None` and the program is permanently un-bootstrappable.
Initialize first, make immutable second.

Validation: `admin`, `quote_authority` and `treasury` all rejected if
`Pubkey::default()`; the refund window must be in bounds.

### `set_config` — rotate the hot key, treasury, refund window

Each field is `Option<Pubkey>` so callers patch selectively. **Does not affect live
orders** — each order snapshots its treasury and deadline at creation. That sentence
is the whole design, repeated in five places.

### `set_paused`

```rust
/// Settlement, refunds and the buyer's timeout exit stay available while paused --
/// a pause must never trap funds.
```

Only `create_order` reads `config.paused`. A pause that froze settlement would turn
an incident response into a fund-freeze.

### `transfer_admin` / `accept_admin` — two-step handover

Nominate, then the nominee proves key control by signing. Nominating a wrong or
uncontrolled key is recoverable because nothing changes until acceptance. This is
the Solana equivalent of OpenZeppelin's `Ownable2Step` on the EVM side. (SOL-08.)

### `register_mint` / `revoke_mint` — the allow-list

```rust
seeds = [b"mint", mint.key().as_ref()]
```

A PDA per allow-listed mint. Existence of the PDA *is* the allow-list entry — there
is no boolean to desynchronise. `revoke_mint` uses `close = admin`, which returns
rent and removes the entry in one step.

`revoke_mint` only blocks **new** orders; existing orders in that mint stay fully
completable and refundable. Revoking a mint must not strand the funds already in it.

### `register_merchant` / `update_merchant`

```rust
require!(ctx.accounts.merchant_authority.key() != Pubkey::default(), ...);
require!(payout_wallet != Pubkey::default(), ...);
```

The first of those two lines is **new** — see Part 4, SOL-13. `merchant_authority`
is an `UncheckedAccount` that never signs: onboarding is a governance action, and
the seller proves key control later by confirming their first order.

`update_merchant` can re-point a payout wallet. **This does not affect orders already
created** — and there is an executed EVM test proving the equivalent: *"keeps paying
the wallet recorded at creation even if the merchant is re-pointed"*, in which the
attacker receives exactly 0.

## 1.4 `create_order` — the heart of it

Guards in order, each one a question someone will ask:

```rust
require!(!config.paused, Paused);
require!(!order_id.is_empty(), InvalidOrderId);
require!(order_id.as_bytes().len() <= MAX_ORDER_ID_LEN, OrderIdTooLong);
require!(quote.amount > 0, InvalidAmount);
require!(quote.fee_amount < quote.amount, InvalidFeeAmount);
require!(merchant.active, MerchantInactive);
```

`fee_amount < amount` (strict) means `seller_amount = amount - fee_amount` is always
**greater than zero** — the seller can never be paid nothing. Note it is checked
before the signature, so a malformed quote fails cheaply.

Length is checked on **bytes**, not chars: `order_id.as_bytes().len()`. A multi-byte
UTF-8 string must not be able to overflow the 64-byte account field.

```rust
let now = Clock::get()?.unix_timestamp;
require!(quote.expiry > now, QuoteExpired);
require!(quote.expiry.checked_sub(now)? <= MAX_QUOTE_LIFETIME_SECS, QuoteLifetimeTooLong);
```

Two separate checks. The first rejects an expired quote; the second rejects one with
an unreasonably **long** life, so a leaked signature has a bounded window. `checked_sub`
because these are `i64` from an on-chain clock.

```rust
let order_id_hash = keccak::hash(order_id.as_bytes()).0;
let expected = quote_message(&order_id_hash, &buyer.key(), &merchant.authority,
                             &mint.key(), &quote);
verify_quote_attestation(&instructions_sysvar, &config.quote_authority, &expected)?;
```

The message binds **everything that could be swapped**: which order, which buyer,
which seller, which mint, the amount, the fee, the expiry. §1.8 covers the
verification itself.

Then the snapshot — the single most important block in the program:

```rust
order.buyer         = ctx.accounts.buyer.key();
order.seller        = ctx.accounts.merchant.authority;
order.mint          = ctx.accounts.mint.key();
order.payout_wallet = ctx.accounts.merchant.payout_wallet;  // from governance state
order.treasury      = ctx.accounts.config.treasury;         // from governance state
order.amount        = quote.amount;
order.fee_amount    = quote.fee_amount;
order.status        = OrderStatus::Pending;
order.refund_deadline = now.checked_add(config.refund_window_secs)?;
```

Destinations are fixed **here**, from state only governance can write, at a moment
the buyer cannot influence. Everything in §1.6 follows from this.

Finally the funds move, buyer → vault, with the buyer as signing authority:

```rust
token::transfer(CpiContext::new(..., Transfer {
    from: buyer_token_account, to: escrow_token_account, authority: buyer,
}), quote.amount)?;
```

### Replay protection — know where it lives

The order PDA is `seeds = [b"order", keccak(order_id)]` with `init`. `init` **fails if
the account already exists**, so a given `order_id` can be created exactly once, ever.
The `Order` account is never closed — not even after completion — so that guard is
permanent.

*Cost of that choice:* the order account's rent (~0.002 SOL) is never reclaimed. That
is a deliberate trade: permanent replay safety for permanent rent. The **vault's**
rent *is* returned to the buyer (§1.7).

## 1.5 `confirm_order` — the seller's commitment

```rust
require!(order.status == OrderStatus::Pending, InvalidStatus);
order.status = OrderStatus::Confirmed;
```

Authorisation lives entirely in the accounts struct:

```rust
#[account(mut, seeds = [...], bump = order.bump, has_one = seller @ Unauthorized)]
pub order: Account<'info, Order>,
pub seller: Signer<'info>,
```

`has_one = seller` forces the passed `seller` account to equal `order.seller`; the
`Signer` type forces it to have signed. Together: only the recorded seller confirms.

**The admin deliberately cannot confirm on a seller's behalf.** Confirmation is the
seller's statement that they will fulfil the service; letting the platform forge it
would recreate the buyer/admin-only relationship this redesign exists to remove.

*(Observation: the `merchant` account in `ConfirmOrder` is loaded and constrained but
never read — `has_one` + `Signer` already do the work. Harmless, slightly redundant.
Mentioning it yourself reads well.)*

## 1.6 `complete_order` — the two-check composition

This is the **set-piece answer**. Two constraints compose to make redirection
impossible:

```rust
#[account(mut, seeds = [b"order", order.order_id_hash.as_ref()], bump = order.bump,
          has_one = mint, has_one = buyer,
          has_one = payout_wallet, has_one = treasury)]
pub order: Account<'info, Order>,

#[account(mut, associated_token::mint = mint, associated_token::authority = payout_wallet)]
pub seller_token_account: Account<'info, TokenAccount>,
```

1. `has_one = payout_wallet` → the passed account **must equal the key recorded on the
   order**.
2. `associated_token::authority = payout_wallet` → the token account **must be that
   key's canonical ATA for the order's mint**.

No instruction argument anywhere in this path names a destination. There is nothing
for a signer to manipulate. The same pair pins the treasury.

Handler body:

```rust
require!(order.status == OrderStatus::Confirmed, InvalidStatus);
require!(authority == order.seller || authority == config.admin, Unauthorized);

let vault_balance = escrow_token_account.amount;
require!(vault_balance >= order.amount, InsufficientEscrow);      // solvency

let seller_amount   = amount.checked_sub(fee_amount)?;
let treasury_amount = vault_balance.checked_sub(seller_amount)?;
```

The arithmetic is provably safe: `vault_balance >= amount >= seller_amount`, so
neither `checked_sub` can underflow. `treasury_amount` is therefore
`fee_amount + (anything donated into the vault)`.

> **Why sweep donations to the treasury?** Anyone can transfer tokens into the
> vault's ATA. Paying out exactly `amount` would leave a non-zero balance and the
> vault could not be closed, stranding its rent forever. Sweeping the excess makes
> the vault land on exactly zero every time. The seller is paid precisely what was
> quoted — never less, and never more.

Then checks-effects-interactions, done properly:

```rust
ctx.accounts.order.status = OrderStatus::Completed;   // terminal state BEFORE any CPI
```

Solana has no reentrancy guard; the discipline *is* the defence. The status is
terminal before the first `token::transfer`, so a re-entrant call would hit
`require!(status == Confirmed)` and fail.

The vault signs its own transfers as a PDA:

```rust
let seeds: &[&[u8]] = &[b"order", order_id_hash.as_ref(), &[bump]];
let signer: &[&[&[u8]]] = &[seeds];
token::transfer(CpiContext::new_with_signer(..., signer), seller_amount)?;
```

`new_with_signer` with the order's own seeds — the program proves it controls the PDA
without any private key existing.

## 1.7 Refunds and the buyer's escape hatch

**`refund_order`** — authorisation varies by status:

| Status | Who may refund |
|---|---|
| `Pending` | buyer, seller **or** admin (the seller hasn't accepted yet) |
| `Confirmed` | seller or admin **only** |
| anything else | rejected (`CannotRefund`) |

The buyer loses unilateral cancellation once the seller has confirmed — otherwise a
buyer could cancel work a seller has already committed to. Always within
`refund_deadline`.

`Completed` / `Refunded` fall into the `_ =>` arm, which is what blocks **double
refund** and **refund-after-payout**.

**`timeout_refund`** — the buyer's unilateral exit:

```rust
require!(matches!(status, Pending | Confirmed), CannotRefund);
require_keys_eq!(authority.key(), order.buyer, Unauthorized);
require!(now > order.refund_deadline, TimeoutNotReached);
```

> Without this, a seller or admin could hold a buyer's money hostage simply by doing
> nothing. Note the boundary: `refund_order` needs `now <= deadline`, `timeout_refund`
> needs `now > deadline`. No gap, no overlap — at every instant exactly one path is
> open.

Both delegate to **`settle_refund`**, shared so the two paths cannot drift apart:
status to `Refunded` before the CPI, the **entire** `vault_balance` (donations
included) back to the buyer, then `close_vault`.

**`close_vault`** returns the vault ATA's rent lamports to the buyer, who paid for it
at creation. Leaving it open would strand rent on every order. (SOL-06.)

## 1.8 `quote.rs` — Ed25519 by instruction introspection

The deepest part of the program. Pure functions (`&[u8]` in, `Result` out), so all 11
of its tests run with plain `cargo test` — no validator.

### The canonical message

```rust
pub const QUOTE_DOMAIN: &[u8] = b"AEROLEDGER_ORDER_QUOTE_V1";   // 25 bytes
pub const QUOTE_MESSAGE_LEN: usize = 25 + 32 + 32 + 32 + 32 + 8 + 8 + 8;  // 177
```

Layout: `domain ‖ order_id_hash ‖ buyer ‖ seller ‖ mint ‖ amount ‖ fee ‖ expiry`,
every integer little-endian, every width fixed.

Two properties worth naming:

- **Domain separation.** Changing the layout must change `QUOTE_DOMAIN`, so a
  signature made for an older layout can never be replayed against a newer one.
- **Fixed widths, no length prefixes.** Every field is a constant size, so there is
  no way to shift bytes between fields and produce a different-but-equal message.
  The test `every_field_changes_the_message` asserts each field independently
  changes the output.

### The security argument — say this *before* they ask

```rust
/// This does NOT verify the signature cryptographically -- the Ed25519 native
/// program already did that, and the runtime fails the whole transaction if it
/// did not. What this function proves is that the verification the runtime
/// performed was over OUR message and OUR signer.
```

Solana's Ed25519 program is a real instruction in the same transaction. If the
signature doesn't check out, **the runtime aborts the entire transaction** — this
program's instruction never executes. So this code doesn't need to redo the maths.
What it needs to prove is *which* verification happened, because the attacker writes
the transaction and can put any Ed25519 instruction in it they like.

### Finding the instruction — position pinning

```rust
let current_index = load_current_index_checked(&sysvar_info)?;
require!(current_index > 0, MissingQuoteSignature);
let ed25519_ix = load_instruction_at_checked((current_index - 1) as usize, &sysvar_info)?;
require_keys_eq!(ed25519_ix.program_id, ed25519_program::ID, MissingQuoteSignature);
```

It must be **immediately before** this instruction, and it must be the native Ed25519
program.

> If instead you scanned the instruction list for the *first* Ed25519 instruction you
> could find, an attacker would put a harmless one they control at index 0 and the
> real check would never look past it. Pinning the position removes the search, and
> therefore removes the ambiguity.

The `current_index > 0` check also handles the trivial case where `create_order` is
the first instruction and there is nothing before it.

### Parsing the payload — four classes of attack

The native instruction's layout:

```text
offset  size  field
     0     1  num_signatures
     1     1  padding
     2     2  signature_offset
     4     2  signature_instruction_index
     6     2  public_key_offset
     8     2  public_key_instruction_index
    10     2  message_data_offset
    12     2  message_data_size
    14     2  message_instruction_index
```

**(a) Burying the real signature among others**

```rust
if ed25519_ix_data[0] != 1 { return Err(NotExactlyOneSignature); }
```

Exactly one. A multi-signature instruction would let an attacker bundle our signature
with others and reuse offsets ambiguously.

**(b) Pointing at another instruction's bytes** — the one unique to introspection

```rust
const SELF_INSTRUCTION_INDEX: u16 = u16::MAX;

if signature_ix_index != SELF_INSTRUCTION_INDEX
    || pubkey_ix_index != SELF_INSTRUCTION_INDEX
    || message_ix_index != SELF_INSTRUCTION_INDEX {
    return Err(IndirectData);
}
```

The Ed25519 program can be told its signature, key and message live in a **different**
instruction of the transaction. If we allowed that, we would be reading bytes from
one instruction while the runtime verified bytes from another — the exact
confused-deputy the whole design is avoiding. Only self-contained instructions are
accepted.

**(c) Malformed offsets / out-of-bounds slicing**

```rust
let sig_end = sig_start.checked_add(ED25519_SIGNATURE_LEN).ok_or(Malformed)?;
let key = ed25519_ix_data.get(key_start..key_end).ok_or(Malformed)?;
```

`checked_add` then `.get()` — every bound checked before any slice. No indexing that
could panic. A panic in a Solana program is a failed transaction, not a vulnerability,
but the discipline is right.

**(d) Wrong signer or wrong message**

```rust
if key != expected_signer.as_ref() { return Err(WrongSigner); }
if msg != expected_message        { return Err(MessageMismatch); }
```

`expected_message` is exactly 177 bytes. If `message_data_size` differs, the slice
comparison fails — so a truncated or extended message is caught by the same check.

### The attacks this defeats, end to end

| Attack | Stopped by |
|---|---|
| Pass `amount = 1` with no signature | No Ed25519 ix at `index-1` → `MissingQuoteSignature` |
| Sign the cheap quote with my own key | `key != expected_signer` → `WrongSigner` |
| Reuse a real signature from a cheaper order | Message binds `order_id_hash`, `mint`, `seller` → `MessageMismatch` |
| Reuse the signature for *this* order twice | Order PDA `init` → account already exists |
| Use an old, legitimately-signed quote | `quote.expiry > now` → `QuoteExpired` |
| Sign a quote with a 10-year expiry | `expiry - now <= 900` → `QuoteLifetimeTooLong` |
| Put a decoy Ed25519 ix at index 0 | Position pinned to `current_index - 1` |
| Bundle many signatures in one ix | `num_signatures != 1` → `NotExactlyOneSignature` |
| Point offsets at another instruction | `!= SELF_INSTRUCTION_INDEX` → `IndirectData` |
| Overflow the offsets | `checked_add` + `.get()` → `Malformed` |

## 1.9 State accounts, events, errors

**`Config`**: admin, `pending_admin: Option<Pubkey>`, quote_authority, treasury,
refund_window_secs, paused, bump. **`SupportedMint`**: mint, decimals, bump — decimals
snapshotted so off-chain consumers needn't guess. **`Merchant`**: authority,
payout_wallet, active, bump. **`Order`**: 5 pubkeys, amount, fee, status, 3
timestamps, `order_id_hash`, `order_id` string, bump.

```rust
impl anchor_lang::Space for OrderStatus { const INIT_SPACE: usize = 1; }
```

Spelled out rather than derived, so the account's size calculation doesn't depend on a
macro's treatment of enums.

**Account sizing is security-relevant** and pinned by an executed test:

```rust
assert_eq!(Order::INIT_SPACE,
    32 * 5 + 8 + 8 + 1 + 24 + 32 + (4 + MAX_ORDER_ID_LEN) + 1);
```

Too small and `init` fails or data truncates; too large and every order overpays rent.
The test means adding a field later cannot silently break allocation.

Eight events (`ConfigUpdated`, `MintRegistered`, `MintRevoked`, `MerchantUpdated`,
`OrderCreated`, `OrderConfirmed`, `OrderCompleted`, `OrderRefunded`) — without these
an indexer has to diff account state to learn anything (SOL-09). 26 named error
variants, each with a `#[msg]`.

---

# Part 2 — The EVM contract

```solidity
contract AeroLedgerEscrow is ReentrancyGuard, Pausable, Ownable2Step, EIP712
```

Four OpenZeppelin bases, each doing one job: reentrancy guard on every value-moving
path, pause on creation only, two-step ownership, typed-data signing.

## 2.1 Types and storage

```solidity
enum OrderStatus { None, Pending, Confirmed, Completed, Refunded }
```

`None` is the zero value — an unwritten mapping slot reads as `None`, which is how
`_openOrder` detects a free slot.

```solidity
address private constant NATIVE = address(0);

bytes32 private constant QUOTE_TYPEHASH = keccak256(
  "Quote(bytes32 orderHash,address buyer,address seller,address token,uint256 amount,uint256 fee,uint256 expiry)"
);

uint256 public constant MAX_QUOTE_LIFETIME = 15 minutes;
uint256 public constant MIN_REFUND_WINDOW  = 1 hours;
uint256 public constant MAX_REFUND_WINDOW  = 30 days;
```

The same three bounds as Solana, same reasoning.

```solidity
mapping(address => uint256) public escrowBalance;
/// Invariant: this contract's balance of each asset >= escrowBalance[asset].
```

**Per-asset accounting.** This is what makes "a tiny native deposit cannot drain
pooled USDC" true — and there's a test by that name.

### `QuoteInput` — why a struct

```solidity
struct QuoteInput { string orderId; address seller; address token;
                    uint256 amount; uint256 fee; uint256 expiry; bytes signature; }
```

> Two reasons. It keeps the signing payload and the call payload obviously
> identical — you can see at a glance that nothing signed is missing from the call.
> And seven loose arguments plus locals blows the EVM's 16-slot stack limit; a
> calldata struct is one slot.

## 2.2 Constructor and governance

```solidity
constructor(...) Ownable(initialOwner) EIP712("AeroLedgerEscrow", "2")
```

The EIP-712 name and **version "2"** — bumping the version invalidates every
signature made against version 1. Useful if the quote layout ever changes.

`supportedTokens[NATIVE] = true` in the constructor, and both `addSupportedToken` and
`removeSupportedToken` revert on `NATIVE` — so native coin is permanently escrowable
and the only way to stop it is `pause()`.

Governance setters: `setQuoteSigner`, `setTreasury`, `setRefundWindow`,
`pause`/`unpause`, `setMerchant`, `addSupportedToken`/`removeSupportedToken`. All
`onlyOwner`, all zero-address checked, all emit. `setTreasury`/`setMerchant` only
affect **future** orders.

## 2.3 Creating an order — two strictly segregated paths

```solidity
function createOrder(QuoteInput calldata q) external payable nonReentrant whenNotPaused {
    if (q.token != NATIVE) revert NotNative();
    if (msg.value != q.amount) revert AmountMismatch();
    ...
}

function createOrderERC20(QuoteInput calldata q) external nonReentrant whenNotPaused {
    if (q.token == NATIVE) revert NotERC20();
    if (!supportedTokens[q.token]) revert TokenNotSupported();
    ...
}
```

`msg.value != q.amount` — **exactly**, not `>=`. The price is the backend's to set.
And the two paths cannot be confused: there are tests named *"rejects a native order
that names an ERC20"* and the converse.

### `_pullTokens` — measure, don't trust

```solidity
uint256 balBefore = erc20.balanceOf(address(this));
erc20.safeTransferFrom(msg.sender, address(this), amount);
received = erc20.balanceOf(address(this)) - balBefore;
if (received < amount) revert AmountMismatch();
```

The contract credits what it **actually gained**, not what it asked for. A token that
taxes the inbound transfer delivers less than `amount` and the order reverts rather
than silently under-funding a signed quote. (EVM-07; there's a `FeeOnTransferToken`
mock and a test.)

Be precise about the limit — see Part 4, EVM-09: this covers an **inbound** tax only.

### `_openOrder` — shared validation

```solidity
if (q.amount == 0) revert AmountMismatch();
if (q.fee >= q.amount) revert InvalidFee();
if (block.timestamp > q.expiry) revert QuoteExpired();
if (q.expiry - block.timestamp > MAX_QUOTE_LIFETIME) revert QuoteLifetimeTooLong();
```

Note the ordering: `QuoteExpired` is checked first, which guarantees
`q.expiry >= block.timestamp`, so the subtraction on the next line cannot underflow.
In 0.8.x an underflow reverts rather than wrapping, but relying on the panic for
control flow would be sloppy — the explicit ordering is the right habit.

```solidity
orderHash = _computeOrderHash(msg.sender, q.orderId);
if (_orders[orderHash].status != OrderStatus.None) revert OrderExists();
```

> **The order slot itself is the replay guard.** A signed quote can be used at most
> once, because the second attempt collides with an existing order — and since a
> Completed or Refunded order still isn't `None`, that holds forever.

```solidity
function _computeOrderHash(address buyer, string calldata orderId) private pure returns (bytes32) {
    return keccak256(abi.encode(buyer, orderId));
}
```

**Deliberately excludes `block.timestamp`**, which the original mixed in and which
made the hash unpredictable off-chain — the backend could not compute the id of the
order it had just authorised. `abi.encode` (not `encodePacked`) so a dynamic `string`
can't be shifted against the address to collide two different inputs.

Then the snapshot, identical in spirit to Solana:

```solidity
payoutWallet: merchant.payoutWallet,   // from governance state
treasury:     treasury,                // from governance state
refundDeadline: block.timestamp + refundWindow
```

### `_verifyQuote`

```solidity
bytes32 digest = _hashTypedDataV4(keccak256(abi.encode(
    QUOTE_TYPEHASH, orderHash, msg.sender, q.seller, q.token, q.amount, q.fee, q.expiry)));
(address recovered, ECDSA.RecoverError err, ) = ECDSA.tryRecover(digest, q.signature);
if (err != ECDSA.RecoverError.NoError || recovered != quoteSigner) revert InvalidQuoteSignature();
```

Four things worth being able to say:

- **`_hashTypedDataV4` binds `chainId` and `verifyingContract`.** A signature harvested
  from a testnet deployment is worthless on mainnet, and one from a previous
  deployment is worthless against this one. That is the reason for EIP-712 over a raw
  `eth_sign` hash.
- **`msg.sender` is in the digest**, not a parameter. The buyer cannot be spoofed.
- **`tryRecover`, not `recover`** — returns an error instead of reverting, so the
  contract emits its own named `InvalidQuoteSignature`. OZ v5's implementation also
  rejects malleable `s` values and bad `v`, so the classic signature-malleability
  trick doesn't apply.
- **Both halves checked.** `err != NoError` *and* `recovered != quoteSigner`. On
  failure `tryRecover` returns `address(0)`, and `quoteSigner` can never be
  `address(0)` — checked in the constructor and the setter — so even a single check
  would hold. Both is correct anyway.

`domainSeparator()` and `quoteDigest(...)` are exposed as views so the backend can
verify it is signing for the chain and contract it believes it is.

## 2.4 Lifecycle

**`confirmOrder`** — `Pending → Confirmed`, `msg.sender == order.seller` only. Not
`nonReentrant`, correctly: it moves no value. The owner cannot confirm, same reasoning
as Solana.

**`completeOrder`**:

```solidity
if (order.status != OrderStatus.Confirmed) revert InvalidStatus();
if (msg.sender != order.seller && msg.sender != owner()) revert NotAuthorized();

uint256 sellerAmount = amount - feeAmount;   // fee < amount enforced at creation

order.status = OrderStatus.Completed;        // Effects
_debitEscrow(token, amount);                 // Effects
if (sellerAmount > 0) _payout(token, payoutWallet, sellerAmount);   // Interactions
if (feeAmount > 0)    _payout(token, orderTreasury, feeAmount);
```

Textbook checks-effects-interactions, **plus** `nonReentrant`. Destinations are read
from `order`, never from arguments. `amount - feeAmount` is unchecked-safe because
`fee < amount` was enforced at creation — and the comment says so, which is the right
way to justify an implicit invariant.

**`refundOrder`** — the same status-dependent authorisation table as Solana, within
`refundDeadline`.

**`timeoutRefund`** — `msg.sender == order.buyer`, `block.timestamp > refundDeadline`.
Again the boundary is exact: `refundOrder` wants `<=`, `timeoutRefund` wants `>`.

**`_settleRefund`** — status to `Refunded`, `_debitEscrow`, pay the recorded buyer.

## 2.5 Helpers and recovery

```solidity
function _debitEscrow(address token, uint256 amount) private {
    uint256 bal = escrowBalance[token];
    if (bal < amount) revert EscrowUnderflow();
    escrowBalance[token] = bal - amount;
}

function _payout(address token, address to, uint256 amount) private {
    if (token == NATIVE) {
        (bool ok, ) = payable(to).call{value: amount}("");
        if (!ok) revert NativeTransferFailed();
    } else {
        IERC20(token).safeTransfer(to, amount);
    }
}
```

`_payout` uses a **checked low-level call**, not `transfer`. `transfer` forwards only
2300 gas, which is not enough for a multisig's `receive()` — using it would brick
payouts to exactly the kind of wallet a serious seller uses. There are two tests:
one paying a gas-hungry multisig, one asserting a failed native payout **reverts**
rather than silently losing the funds.

```solidity
function unaccountedBalance(address token) public view returns (uint256) {
    uint256 held = token == NATIVE ? address(this).balance : IERC20(token).balanceOf(address(this));
    uint256 owed = escrowBalance[token];
    return held > owed ? held - owed : 0;
}

function sweepUnaccounted(address token, address to) external onlyOwner nonReentrant { ... }
```

> Recovery for assets sent here by mistake, **capped by `unaccountedBalance`** — it
> subtracts every live order's deposit, so the owner cannot touch escrowed funds. The
> tests are *"can sweep only funds that are not backing a live order"* and *"only the
> owner can sweep"*. (EVM-06.)

Note the contract has **no `receive()` or `fallback()`**, so a stray `send` reverts
instead of silently accumulating. `selfdestruct`-forced ether still lands as
unaccounted and is sweepable.

---

# Part 3 — Side by side

| Concern | Solana | EVM |
|---|---|---|
| Price authority | Ed25519 + instruction introspection | EIP-712 + `ECDSA.tryRecover` |
| Replay guard | Order PDA `init` (account must not exist) | `_orders[hash].status != None` |
| Destination pinning | `has_one` + `associated_token::authority` | Snapshot fields read in the handler |
| Reentrancy | Checks-effects-interactions only | CEI **+** `nonReentrant` |
| Asset segregation | One vault ATA per order | `escrowBalance` per asset |
| Surplus/donations | Swept to treasury at completion | Credited to the order → paid to the **seller** |
| Ownership handover | `transfer_admin` / `accept_admin` | `Ownable2Step` |
| Token restriction | `SupportedMint` PDA + SPL-only types | `supportedTokens` mapping |
| Rent / storage | Vault rent returned; order rent permanent | Storage never freed |
| Tests | 14 unit executed; Anchor suite **not** executed | 52 executed |

**The one real semantic divergence** is the surplus row — worth volunteering, because
an interviewer comparing the two files will spot it.

---

# Part 4 — The audit

What was checked on this pass, and what came out of it. Two findings, both logged in
`SECURITY_AUDIT.md`.

## Verified sound (no change needed)

- **Arithmetic.** Every `checked_sub`/`checked_add` on Solana was traced for
  reachability of underflow. `complete_order` is provably safe:
  `vault_balance >= amount >= seller_amount`. EVM relies on 0.8.x checked arithmetic
  plus the explicit `fee < amount` and expiry-ordering invariants.
- **Reentrancy.** Solana sets the terminal status before every CPI on all three
  value-moving paths. EVM is CEI *and* guarded. During `_pullTokens` the guard is
  already held, so `sweepUnaccounted` cannot observe the window where tokens have
  arrived but `escrowBalance` has not yet been credited.
- **Replay.** Both guards are permanent, because neither an `Order` PDA nor an EVM
  order slot is ever cleared.
- **Quote binding.** Every field an attacker would want to change — order, buyer,
  seller, mint/token, amount, fee, expiry — is inside the signed payload on both
  chains.
- **Refund boundary.** `<=` versus `>` on both chains: no instant where both paths or
  neither path is open.
- **Pause semantics.** Only creation is pausable on both. A pause cannot strand funds.
- **Account sizing.** Pinned by an executed test.

## SOL-13 — `register_merchant` accepted the default pubkey — Low, **fixed**

Every other authority in the program is checked against `Pubkey::default()` — admin,
quote_authority, treasury, the `transfer_admin` nominee, and both `payout_wallet`
sites. `register_merchant` checked `payout_wallet` but not `merchant_authority`,
which arrives as an `UncheckedAccount` and never signs.

Nobody holds the private key for the all-zero pubkey, so such a merchant can never
sign `confirm_order`; orders against them sit `Pending` until refunded or timed out.
**Self-inflicted availability fault by a misconfiguring admin, not a theft path** —
hence Low. The EVM twin was already stricter (`setMerchant` rejects a zero `seller`),
and that inconsistency between the two programs is the real smell.

Fixed with the same one-line `require!` the other eight sites use. Honest caveat: the
guard sits inside an Anchor instruction context, so it is **not** unit-tested — it
belongs to the unexecuted Anchor suite.

## EVM-09 — NatSpec misdescribed the surplus and overstated coverage — Info, **fixed**

1. `createOrderERC20` claimed a rebase-up surplus *"settles to the treasury"*. It does
   not: `order.amount` becomes `received` while `fee` stays at the signed value, so
   `sellerAmount = received - fee` and the surplus goes to the **seller**. The Solana
   twin genuinely does sweep to the treasury. The comment made two different
   behaviours look identical.
2. *"A fee-on-transfer token reverts"* is true only for a tax on the **inbound**
   transfer. A token taxing only the outbound transfer passes creation, and at payout
   the recipients receive less than the quote while `escrowBalance` is debited in
   full.

   That second case was analysed, not waved away: it **cannot** break the per-asset
   solvency invariant or reach another order's funds, because the contract's balance
   falls by at least the debited amount. It underpays that one order's two recipients.
   The control is the owner-managed allow-list — so the comment must not claim code
   does it.

Documentation only; no behaviour change. Both comments now state what the code does.

## Post-fix verification

```text
contracts/solana/programs/aeroledger-escrow $ cargo test --lib   → 14 passed, 0 failed
contracts/ethereum                          $ npx hardhat test   → 52 passed, 0 failed
                                            $ npx hardhat compile → clean
```

---

# Part 5 — Residual risks. Say these before you're asked.

1. **Neither contract is wired into the backend.** The live path is direct payment
   with on-chain verification.
2. **The Anchor integration suite has never been executed** — 30+ adversarial cases,
   written and committed, no Solana toolchain available. The EVM equivalents *are*
   executed.
3. **The Solana program id is the Anchor placeholder** (SOL-12). Deployment blocker by
   design; `anchor keys sync` before any deploy.
4. **Single-key governance on both.** `Config.admin` and `owner()` should be a
   multisig behind a timelock before real funds move.
5. **The quote authority is a hot key** on both chains. It belongs in an HSM/KMS
   signer. It is deliberately a *separate*, rotatable role from admin precisely
   because it is the one most likely to leak — but separation limits the damage, it
   doesn't prevent it. A leaked quote key authorises under-priced orders.
6. **Compromised governance means denial of service.** An admin can deactivate
   merchants and refund live orders. That is DoS, not theft — the destination
   constraints hold even against the admin — but it is real.
7. **Token-2022 is rejected, not supported** (Solana). Deliberate and fail-closed.
8. **An outbound-taxing ERC20 would underpay** (EVM, EVM-09). Excluded by the
   allow-list, not by code.
9. **Order account rent is never reclaimed on Solana.** The price of permanent replay
   protection.
10. **Oracle/price risk is off-chain.** Both contracts trust the quote signer
    completely. If the backend prices something wrongly, both chains will faithfully
    enforce the wrong price. The contracts guarantee *"the amount the backend signed"*,
    never *"a fair amount"*.
