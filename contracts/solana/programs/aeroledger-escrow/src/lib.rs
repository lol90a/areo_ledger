//! # AeroLedger Escrow (Solana / Anchor)
//!
//! ## Commercial model
//!
//! AeroLedger is a **marketplace intermediary**, not the merchant of record. Every
//! order is a three-party relationship:
//!
//! | Party        | On-chain identity                 | Role                                                |
//! |--------------|-----------------------------------|-----------------------------------------------------|
//! | Buyer        | `Order.buyer` (signs `create_order`) | Funds the escrow vault, receives refunds           |
//! | Seller       | `Merchant.authority`              | Accepts the order, receives the net payout          |
//! | Payout wallet| `Merchant.payout_wallet`          | Where the seller's proceeds land (ATA of this key)  |
//! | Platform     | `Config.treasury`                 | Receives the quoted platform fee                    |
//! | Admin        | `Config.admin`                    | Governance only -- see "Privileged actions" below   |
//! | Quote signer | `Config.quote_authority`          | Ed25519 key that authorises the price of each order |
//!
//! The previous version of this program had no seller at all: `complete_order`
//! moved the buyer's tokens to an `admin_token_account`, so the escrow encoded a
//! buyer/admin relationship and the platform was implicitly the merchant for every
//! sale. That is the issue raised in review and it is fixed here: a seller must be
//! registered by governance before any order can name them, and the destination of
//! a payout is **snapshotted into the order at creation time** from the merchant
//! record. Nobody -- not the buyer, not the seller, not the admin -- can pass a
//! different token account at completion time and redirect the money.
//!
//! ## Privileged actions (explicit trust model)
//!
//! The admin key CAN:
//!   * register / deactivate merchants and supported mints,
//!   * rotate the quote authority and treasury (two-step for the admin itself),
//!   * pause new order creation,
//!   * complete a confirmed order (release to the **registered** seller wallet),
//!   * refund an order to the **recorded buyer** before the refund deadline.
//!
//! The admin CANNOT:
//!   * choose an arbitrary payout or refund destination (both are derived from
//!     state recorded at order creation),
//!   * change the amount of a live order,
//!   * confirm an order on a seller's behalf (only `Merchant.authority` can),
//!   * take funds out of a vault by any path other than the two above,
//!   * block a buyer's exit forever -- `timeout_refund` is a unilateral buyer
//!     escape hatch once the refund deadline passes.
//!
//! Residual risk: a compromised admin can deactivate merchants (denial of service)
//! and can refund live orders to their buyers (also DoS, not theft). A compromised
//! `quote_authority` can authorise under-priced orders; it is a hot key and must be
//! rotatable, which is why it is a separate role from `admin`.
//!
//! ## Token support
//!
//! SPL Token only. `Program<'info, Token>` plus `anchor_spl::token::{Mint,
//! TokenAccount}` pin every token account to the legacy SPL Token program, so
//! Token-2022 mints are rejected at deserialization. This is deliberate and
//! fail-closed: Token-2022 transfer fees, transfer hooks and confidential-transfer
//! extensions would each break the escrow's "what went in is what comes out"
//! accounting, and supporting them safely requires per-extension analysis that has
//! not been done. Mints are additionally gated by an admin-managed allow-list
//! (`SupportedMint`), so only reviewed mints can ever be escrowed.

use anchor_lang::prelude::*;
use anchor_lang::solana_program::keccak;
use anchor_lang::solana_program::sysvar::instructions::{
    load_current_index_checked, load_instruction_at_checked, ID as INSTRUCTIONS_SYSVAR_ID,
};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, CloseAccount, Mint, Token, TokenAccount, Transfer};

pub mod quote;
pub use quote::{quote_message, verify_quote_signature_data, QuoteCommitment, QuoteSignatureError};

// SECURITY / DEPLOYMENT NOTE:
// The program id below is the Anchor template placeholder. A REAL program keypair
// MUST be generated and this `declare_id!` updated to match
// `target/deploy/aeroledger_escrow-keypair.json` BEFORE any deploy to
// localnet/devnet/mainnet. The same id must match every entry in Anchor.toml.
// Use the sed one-liner in Anchor.toml, not `anchor keys sync`, which updates
// only one of the three [programs.*] entries per run (see TESTING.md).
// Deploying with this placeholder will fail id/authority checks.
declare_id!("Fg6PaFpoGXkYsidMpWxTWqK6W2BeZ7FEfcYkgqPLF6j");

/// Maximum accepted length (in bytes) of an `order_id` string.
///
/// A canonical UUID is 36 chars; 64 leaves room for other backend identifier
/// formats. The string is NOT used as a PDA seed (a 64-byte seed would exceed the
/// 32-byte limit); a fixed 32-byte keccak hash of it is the seed, and the full
/// string is stored in account data so off-chain indexers can correlate.
pub const MAX_ORDER_ID_LEN: usize = 64;

/// Hard ceiling on the admin-configurable refund window, so governance cannot set
/// a window so long that buyers can claw back settled business, nor so short that
/// buyers have no exit. 1 hour .. 30 days.
pub const MIN_REFUND_WINDOW_SECS: i64 = 3_600;
pub const MAX_REFUND_WINDOW_SECS: i64 = 30 * 86_400;

/// Longest a signed quote may remain usable, measured from the moment the order is
/// created. Even if the backend signs a quote with a far-future expiry, the program
/// refuses anything beyond this, bounding the damage from a leaked quote.
pub const MAX_QUOTE_LIFETIME_SECS: i64 = 900; // 15 minutes

#[program]
pub mod aeroledger_escrow {
    use super::*;

    // ---------------------------------------------------------------------
    // Governance
    // ---------------------------------------------------------------------

    /// One-time program setup.
    ///
    /// Gated on the program's **upgrade authority**, not on "whoever calls first".
    /// Without this an attacker could front-run the deployer's `initialize` call
    /// and install themselves as admin of a freshly deployed program.
    pub fn initialize(
        ctx: Context<Initialize>,
        admin: Pubkey,
        quote_authority: Pubkey,
        treasury: Pubkey,
        refund_window_secs: i64,
    ) -> Result<()> {
        require!(admin != Pubkey::default(), EscrowError::InvalidAuthority);
        require!(
            quote_authority != Pubkey::default(),
            EscrowError::InvalidAuthority
        );
        require!(treasury != Pubkey::default(), EscrowError::InvalidAuthority);
        require!(
            (MIN_REFUND_WINDOW_SECS..=MAX_REFUND_WINDOW_SECS).contains(&refund_window_secs),
            EscrowError::InvalidRefundWindow
        );

        let config = &mut ctx.accounts.config;
        config.admin = admin;
        config.pending_admin = None;
        config.quote_authority = quote_authority;
        config.treasury = treasury;
        config.refund_window_secs = refund_window_secs;
        config.paused = false;
        config.bump = ctx.bumps.config;

        emit!(ConfigUpdated {
            admin,
            quote_authority,
            treasury,
            refund_window_secs,
            paused: false,
        });
        Ok(())
    }

    /// Rotate the hot quote-signing key, the treasury, and the refund window.
    /// Admin only. Does not affect orders already created -- each order snapshots
    /// its treasury and deadline at creation.
    pub fn set_config(
        ctx: Context<AdminOnly>,
        quote_authority: Option<Pubkey>,
        treasury: Option<Pubkey>,
        refund_window_secs: Option<i64>,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        if let Some(key) = quote_authority {
            require!(key != Pubkey::default(), EscrowError::InvalidAuthority);
            config.quote_authority = key;
        }
        if let Some(key) = treasury {
            require!(key != Pubkey::default(), EscrowError::InvalidAuthority);
            config.treasury = key;
        }
        if let Some(window) = refund_window_secs {
            require!(
                (MIN_REFUND_WINDOW_SECS..=MAX_REFUND_WINDOW_SECS).contains(&window),
                EscrowError::InvalidRefundWindow
            );
            config.refund_window_secs = window;
        }

        emit!(ConfigUpdated {
            admin: config.admin,
            quote_authority: config.quote_authority,
            treasury: config.treasury,
            refund_window_secs: config.refund_window_secs,
            paused: config.paused,
        });
        Ok(())
    }

    /// Pause or resume new order creation. Settlement, refunds and the buyer's
    /// timeout exit stay available while paused -- a pause must never trap funds.
    pub fn set_paused(ctx: Context<AdminOnly>, paused: bool) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.paused = paused;
        emit!(ConfigUpdated {
            admin: config.admin,
            quote_authority: config.quote_authority,
            treasury: config.treasury,
            refund_window_secs: config.refund_window_secs,
            paused,
        });
        Ok(())
    }

    /// Step 1 of a two-step admin handover. Nominating a wrong or uncontrolled key
    /// is recoverable: nothing changes until the nominee accepts.
    pub fn transfer_admin(ctx: Context<AdminOnly>, new_admin: Pubkey) -> Result<()> {
        require!(
            new_admin != Pubkey::default(),
            EscrowError::InvalidAuthority
        );
        ctx.accounts.config.pending_admin = Some(new_admin);
        Ok(())
    }

    /// Step 2 of the handover: the nominee proves key control by signing.
    pub fn accept_admin(ctx: Context<AcceptAdmin>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        let pending = config.pending_admin.ok_or(EscrowError::NoPendingAdmin)?;
        require_keys_eq!(
            pending,
            ctx.accounts.new_admin.key(),
            EscrowError::Unauthorized
        );
        config.admin = pending;
        config.pending_admin = None;

        emit!(ConfigUpdated {
            admin: config.admin,
            quote_authority: config.quote_authority,
            treasury: config.treasury,
            refund_window_secs: config.refund_window_secs,
            paused: config.paused,
        });
        Ok(())
    }

    /// Add a mint to the escrow allow-list. Only allow-listed mints can be escrowed,
    /// which keeps unreviewed / malicious / wrong-decimal tokens out of the system.
    pub fn register_mint(ctx: Context<RegisterMint>) -> Result<()> {
        let entry = &mut ctx.accounts.supported_mint;
        entry.mint = ctx.accounts.mint.key();
        entry.decimals = ctx.accounts.mint.decimals;
        entry.bump = ctx.bumps.supported_mint;
        emit!(MintRegistered {
            mint: entry.mint,
            decimals: entry.decimals,
        });
        Ok(())
    }

    /// Remove a mint from the allow-list. Existing orders in that mint stay fully
    /// completable and refundable -- this only blocks NEW orders.
    pub fn revoke_mint(ctx: Context<RevokeMint>) -> Result<()> {
        emit!(MintRevoked {
            mint: ctx.accounts.supported_mint.mint,
        });
        Ok(())
    }

    /// Register a seller and the wallet their proceeds must land in.
    ///
    /// `payout_wallet` is recorded here, under governance control, and copied into
    /// every order the merchant is party to. Payout destinations are therefore never
    /// chosen at settlement time by whoever happens to be signing.
    pub fn register_merchant(ctx: Context<RegisterMerchant>, payout_wallet: Pubkey) -> Result<()> {
        // `merchant_authority` is an UncheckedAccount that never signs here, so it is
        // whatever key governance passed. The default pubkey has no private key, so
        // registering it would create a seller who can never sign `confirm_order` --
        // every order against them would be stuck until refunded. Rejected for the
        // same reason, and in the same way, as every other authority in this program.
        require!(
            ctx.accounts.merchant_authority.key() != Pubkey::default(),
            EscrowError::InvalidAuthority
        );
        require!(
            payout_wallet != Pubkey::default(),
            EscrowError::InvalidAuthority
        );
        let merchant = &mut ctx.accounts.merchant;
        merchant.authority = ctx.accounts.merchant_authority.key();
        merchant.payout_wallet = payout_wallet;
        merchant.active = true;
        merchant.bump = ctx.bumps.merchant;

        emit!(MerchantUpdated {
            merchant: merchant.authority,
            payout_wallet,
            active: true,
        });
        Ok(())
    }

    /// Activate / deactivate a merchant, or point them at a new payout wallet.
    /// Changing the payout wallet does NOT affect orders already created.
    pub fn update_merchant(
        ctx: Context<UpdateMerchant>,
        payout_wallet: Option<Pubkey>,
        active: Option<bool>,
    ) -> Result<()> {
        let merchant = &mut ctx.accounts.merchant;
        if let Some(wallet) = payout_wallet {
            require!(wallet != Pubkey::default(), EscrowError::InvalidAuthority);
            merchant.payout_wallet = wallet;
        }
        if let Some(flag) = active {
            merchant.active = flag;
        }
        emit!(MerchantUpdated {
            merchant: merchant.authority,
            payout_wallet: merchant.payout_wallet,
            active: merchant.active,
        });
        Ok(())
    }

    // ---------------------------------------------------------------------
    // Order lifecycle
    // ---------------------------------------------------------------------

    /// Buyer creates an order and funds the escrow vault.
    ///
    /// The price is NOT chosen by the buyer. `quote` must be covered by an Ed25519
    /// signature from `config.quote_authority` over a canonical message binding
    /// (order id, buyer, seller, mint, amount, fee, expiry) -- see [`quote`]. The
    /// signature is verified by the native Ed25519 program in the same transaction
    /// and this instruction proves, by introspecting the instruction list, that the
    /// verified bytes are exactly the ones this order commits to.
    pub fn create_order(
        ctx: Context<CreateOrder>,
        order_id: String,
        quote: QuoteCommitment,
    ) -> Result<()> {
        require!(!ctx.accounts.config.paused, EscrowError::Paused);
        require!(!order_id.is_empty(), EscrowError::InvalidOrderId);
        require!(
            order_id.as_bytes().len() <= MAX_ORDER_ID_LEN,
            EscrowError::OrderIdTooLong
        );
        require!(quote.amount > 0, EscrowError::InvalidAmount);
        // The fee is carved out of the escrowed amount, so it can never exceed it.
        require!(
            quote.fee_amount < quote.amount,
            EscrowError::InvalidFeeAmount
        );
        require!(ctx.accounts.merchant.active, EscrowError::MerchantInactive);

        let now = Clock::get()?.unix_timestamp;
        require!(quote.expiry > now, EscrowError::QuoteExpired);
        require!(
            quote
                .expiry
                .checked_sub(now)
                .ok_or(EscrowError::MathOverflow)?
                <= MAX_QUOTE_LIFETIME_SECS,
            EscrowError::QuoteLifetimeTooLong
        );

        let order_id_hash = keccak::hash(order_id.as_bytes()).0;
        let expected = quote_message(
            &order_id_hash,
            &ctx.accounts.buyer.key(),
            &ctx.accounts.merchant.authority,
            &ctx.accounts.mint.key(),
            &quote,
        );
        verify_quote_attestation(
            &ctx.accounts.instructions_sysvar,
            &ctx.accounts.config.quote_authority,
            &expected,
        )?;

        let refund_deadline = now
            .checked_add(ctx.accounts.config.refund_window_secs)
            .ok_or(EscrowError::MathOverflow)?;

        let order = &mut ctx.accounts.order;
        order.buyer = ctx.accounts.buyer.key();
        order.seller = ctx.accounts.merchant.authority;
        order.mint = ctx.accounts.mint.key();
        // Settlement destinations are fixed HERE, from governance-controlled state.
        order.payout_wallet = ctx.accounts.merchant.payout_wallet;
        order.treasury = ctx.accounts.config.treasury;
        order.amount = quote.amount;
        order.fee_amount = quote.fee_amount;
        order.status = OrderStatus::Pending;
        order.created_at = now;
        order.refund_deadline = refund_deadline;
        order.quote_expiry = quote.expiry;
        order.order_id_hash = order_id_hash;
        order.order_id = order_id;
        order.bump = ctx.bumps.order;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.buyer_token_account.to_account_info(),
                    to: ctx.accounts.escrow_token_account.to_account_info(),
                    authority: ctx.accounts.buyer.to_account_info(),
                },
            ),
            quote.amount,
        )?;

        emit!(OrderCreated {
            order_id_hash,
            buyer: order.buyer,
            seller: order.seller,
            mint: order.mint,
            amount: order.amount,
            fee_amount: order.fee_amount,
            refund_deadline,
        });
        Ok(())
    }

    /// The **seller** accepts the order: Pending -> Confirmed.
    ///
    /// Only `Merchant.authority` may do this. The admin deliberately cannot confirm
    /// on a seller's behalf, because confirmation is the seller's statement that
    /// they will fulfil the service; letting the platform forge it would recreate
    /// the buyer/admin-only relationship this redesign removes.
    pub fn confirm_order(ctx: Context<ConfirmOrder>) -> Result<()> {
        let order = &mut ctx.accounts.order;
        require!(
            order.status == OrderStatus::Pending,
            EscrowError::InvalidStatus
        );
        order.status = OrderStatus::Confirmed;
        emit!(OrderConfirmed {
            order_id_hash: order.order_id_hash,
            seller: order.seller,
        });
        Ok(())
    }

    /// Release a confirmed order: net proceeds to the seller's registered wallet,
    /// platform fee to the treasury recorded on the order.
    ///
    /// Callable by the seller or the admin. Both destinations come from the order
    /// account, and the token accounts are constrained to be the associated token
    /// accounts of those recorded keys for the order's mint -- so neither signer can
    /// substitute a destination they control.
    pub fn complete_order(ctx: Context<CompleteOrder>) -> Result<()> {
        require!(
            ctx.accounts.order.status == OrderStatus::Confirmed,
            EscrowError::InvalidStatus
        );

        let authority = ctx.accounts.authority.key();
        require!(
            authority == ctx.accounts.order.seller || authority == ctx.accounts.config.admin,
            EscrowError::Unauthorized
        );

        let vault_balance = ctx.accounts.escrow_token_account.amount;
        let amount = ctx.accounts.order.amount;
        let fee_amount = ctx.accounts.order.fee_amount;
        // Solvency: the vault must still hold at least what the order promised.
        require!(vault_balance >= amount, EscrowError::InsufficientEscrow);

        let seller_amount = amount
            .checked_sub(fee_amount)
            .ok_or(EscrowError::MathOverflow)?;
        // Anything above the quoted amount (an unsolicited donation into the vault)
        // sweeps to the treasury, so the vault always lands on exactly zero and can
        // be closed. The seller is paid precisely what was quoted, never less.
        let treasury_amount = vault_balance
            .checked_sub(seller_amount)
            .ok_or(EscrowError::MathOverflow)?;

        // Terminal state BEFORE any CPI (checks-effects-interactions).
        ctx.accounts.order.status = OrderStatus::Completed;

        let order_id_hash = ctx.accounts.order.order_id_hash;
        let bump = ctx.accounts.order.bump;
        let seeds: &[&[u8]] = &[b"order", order_id_hash.as_ref(), &[bump]];
        let signer: &[&[&[u8]]] = &[seeds];

        if seller_amount > 0 {
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.escrow_token_account.to_account_info(),
                        to: ctx.accounts.seller_token_account.to_account_info(),
                        authority: ctx.accounts.order.to_account_info(),
                    },
                    signer,
                ),
                seller_amount,
            )?;
        }
        if treasury_amount > 0 {
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.escrow_token_account.to_account_info(),
                        to: ctx.accounts.treasury_token_account.to_account_info(),
                        authority: ctx.accounts.order.to_account_info(),
                    },
                    signer,
                ),
                treasury_amount,
            )?;
        }

        close_vault(
            &ctx.accounts.token_program,
            &ctx.accounts.escrow_token_account,
            &ctx.accounts.buyer,
            &ctx.accounts.order,
            signer,
        )?;

        emit!(OrderCompleted {
            order_id_hash,
            seller: ctx.accounts.order.seller,
            seller_amount,
            treasury_amount,
        });
        Ok(())
    }

    /// Return escrowed funds to the recorded buyer.
    ///
    /// Authorisation by status:
    ///   * `Pending`   -- buyer, seller or admin (the seller has not accepted yet),
    ///   * `Confirmed` -- seller or admin only (the buyer cannot unilaterally cancel
    ///     work the seller has already committed to; they use [`timeout_refund`]).
    ///
    /// Always within the refund deadline, always to the buyer's ATA for the order's
    /// mint. A `Completed` or already-`Refunded` order is rejected by the status
    /// guard, which is what blocks double-refund and refund-after-payout.
    pub fn refund_order(mut ctx: Context<RefundOrder>) -> Result<()> {
        let status = ctx.accounts.order.status;
        let authority = ctx.accounts.authority.key();
        let is_buyer = authority == ctx.accounts.order.buyer;
        let is_seller = authority == ctx.accounts.order.seller;
        let is_admin = authority == ctx.accounts.config.admin;

        match status {
            OrderStatus::Pending => {
                require!(is_buyer || is_seller || is_admin, EscrowError::Unauthorized)
            }
            OrderStatus::Confirmed => require!(is_seller || is_admin, EscrowError::Unauthorized),
            _ => return Err(EscrowError::CannotRefund.into()),
        }

        let now = Clock::get()?.unix_timestamp;
        require!(
            now <= ctx.accounts.order.refund_deadline,
            EscrowError::RefundWindowExpired
        );

        settle_refund(&mut ctx, RefundKind::Refund)
    }

    /// Unilateral buyer exit for a stuck order.
    ///
    /// If the seller never confirms, or confirms and never completes, the buyer can
    /// always reclaim their funds once the refund deadline passes. Without this the
    /// seller or admin could hold a buyer's money hostage simply by doing nothing.
    pub fn timeout_refund(mut ctx: Context<RefundOrder>) -> Result<()> {
        require!(
            matches!(
                ctx.accounts.order.status,
                OrderStatus::Pending | OrderStatus::Confirmed
            ),
            EscrowError::CannotRefund
        );
        require_keys_eq!(
            ctx.accounts.authority.key(),
            ctx.accounts.order.buyer,
            EscrowError::Unauthorized
        );

        let now = Clock::get()?.unix_timestamp;
        require!(
            now > ctx.accounts.order.refund_deadline,
            EscrowError::TimeoutNotReached
        );

        settle_refund(&mut ctx, RefundKind::Timeout)
    }
}

// =========================================================================
// Shared handler helpers
// =========================================================================

enum RefundKind {
    Refund,
    Timeout,
}

/// Move the whole vault back to the buyer and close it. Shared by `refund_order`
/// and `timeout_refund` so the two paths cannot drift apart.
fn settle_refund(ctx: &mut Context<RefundOrder>, kind: RefundKind) -> Result<()> {
    let vault_balance = ctx.accounts.escrow_token_account.amount;
    require!(
        vault_balance >= ctx.accounts.order.amount,
        EscrowError::InsufficientEscrow
    );

    // Terminal state before the CPI (checks-effects-interactions).
    ctx.accounts.order.status = OrderStatus::Refunded;

    let order_id_hash = ctx.accounts.order.order_id_hash;
    let bump = ctx.accounts.order.bump;
    let seeds: &[&[u8]] = &[b"order", order_id_hash.as_ref(), &[bump]];
    let signer: &[&[&[u8]]] = &[seeds];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow_token_account.to_account_info(),
                to: ctx.accounts.buyer_token_account.to_account_info(),
                authority: ctx.accounts.order.to_account_info(),
            },
            signer,
        ),
        vault_balance,
    )?;

    close_vault(
        &ctx.accounts.token_program,
        &ctx.accounts.escrow_token_account,
        &ctx.accounts.buyer,
        &ctx.accounts.order,
        signer,
    )?;

    emit!(OrderRefunded {
        order_id_hash,
        buyer: ctx.accounts.order.buyer,
        amount: vault_balance,
        timed_out: matches!(kind, RefundKind::Timeout),
    });
    Ok(())
}

/// Close the (now empty) escrow ATA, returning its rent lamports to the buyer who
/// paid for it. Leaving it open would strand rent permanently on every order.
fn close_vault<'info>(
    token_program: &Program<'info, Token>,
    vault: &Account<'info, TokenAccount>,
    rent_destination: &UncheckedAccount<'info>,
    order: &Account<'info, Order>,
    signer: &[&[&[u8]]],
) -> Result<()> {
    token::close_account(CpiContext::new_with_signer(
        token_program.to_account_info(),
        CloseAccount {
            account: vault.to_account_info(),
            destination: rent_destination.to_account_info(),
            authority: order.to_account_info(),
        },
        signer,
    ))
}

/// Prove that the transaction contains a native Ed25519 verification of
/// `expected_message` by `expected_signer`, immediately before this instruction.
///
/// Pinning the position to `current_index - 1` means an attacker cannot append a
/// harmless Ed25519 instruction somewhere else in the transaction and have it
/// mistaken for the quote attestation.
fn verify_quote_attestation(
    instructions_sysvar: &UncheckedAccount,
    expected_signer: &Pubkey,
    expected_message: &[u8],
) -> Result<()> {
    let sysvar_info = instructions_sysvar.to_account_info();
    let current_index = load_current_index_checked(&sysvar_info)
        .map_err(|_| error!(EscrowError::MissingQuoteSignature))?;
    require!(current_index > 0, EscrowError::MissingQuoteSignature);

    let ed25519_ix = load_instruction_at_checked((current_index - 1) as usize, &sysvar_info)
        .map_err(|_| error!(EscrowError::MissingQuoteSignature))?;
    require_keys_eq!(
        ed25519_ix.program_id,
        anchor_lang::solana_program::ed25519_program::ID,
        EscrowError::MissingQuoteSignature
    );

    verify_quote_signature_data(&ed25519_ix.data, expected_signer, expected_message).map_err(|e| {
        match e {
            QuoteSignatureError::WrongSigner => error!(EscrowError::UnauthorizedQuoteSigner),
            QuoteSignatureError::MessageMismatch => error!(EscrowError::QuoteMismatch),
            _ => error!(EscrowError::MalformedQuoteSignature),
        }
    })
}

// =========================================================================
// Accounts
// =========================================================================

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + Config::INIT_SPACE,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(mut)]
    pub payer: Signer<'info>,

    /// This program, used to reach its ProgramData account.
    #[account(
        constraint = program.programdata_address()? == Some(program_data.key())
            @ EscrowError::Unauthorized
    )]
    pub program: Program<'info, program::AeroledgerEscrow>,

    /// Only the key that can upgrade this program may bootstrap its config.
    #[account(
        constraint = program_data.upgrade_authority_address == Some(payer.key())
            @ EscrowError::Unauthorized
    )]
    pub program_data: Account<'info, ProgramData>,

    pub system_program: Program<'info, System>,
}

/// Any instruction whose sole authorisation requirement is "signed by the admin".
#[derive(Accounts)]
pub struct AdminOnly<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
        has_one = admin @ EscrowError::Unauthorized
    )]
    pub config: Account<'info, Config>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct AcceptAdmin<'info> {
    #[account(mut, seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,
    /// Must match `config.pending_admin` (checked in the handler).
    pub new_admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct RegisterMint<'info> {
    #[account(
        seeds = [b"config"],
        bump = config.bump,
        has_one = admin @ EscrowError::Unauthorized
    )]
    pub config: Account<'info, Config>,

    #[account(
        init,
        payer = admin,
        space = 8 + SupportedMint::INIT_SPACE,
        seeds = [b"mint", mint.key().as_ref()],
        bump
    )]
    pub supported_mint: Account<'info, SupportedMint>,

    /// Typed as `anchor_spl::token::Mint`, which pins the owner to the legacy SPL
    /// Token program -- a Token-2022 mint cannot be allow-listed.
    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RevokeMint<'info> {
    #[account(
        seeds = [b"config"],
        bump = config.bump,
        has_one = admin @ EscrowError::Unauthorized
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        close = admin,
        seeds = [b"mint", supported_mint.mint.as_ref()],
        bump = supported_mint.bump
    )]
    pub supported_mint: Account<'info, SupportedMint>,

    #[account(mut)]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct RegisterMerchant<'info> {
    #[account(
        seeds = [b"config"],
        bump = config.bump,
        has_one = admin @ EscrowError::Unauthorized
    )]
    pub config: Account<'info, Config>,

    #[account(
        init,
        payer = admin,
        space = 8 + Merchant::INIT_SPACE,
        seeds = [b"merchant", merchant_authority.key().as_ref()],
        bump
    )]
    pub merchant: Account<'info, Merchant>,

    /// The seller's identity key. Not a signer: onboarding is a governance action,
    /// and the seller proves key control later by confirming their first order.
    /// CHECK: used only as a PDA seed and stored as `Merchant.authority`.
    pub merchant_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateMerchant<'info> {
    #[account(
        seeds = [b"config"],
        bump = config.bump,
        has_one = admin @ EscrowError::Unauthorized
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [b"merchant", merchant.authority.as_ref()],
        bump = merchant.bump
    )]
    pub merchant: Account<'info, Merchant>,

    pub admin: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(order_id: String)]
pub struct CreateOrder<'info> {
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Box<Account<'info, Config>>,

    /// Proves the mint is on the governance allow-list, and ties the PDA to it.
    #[account(
        seeds = [b"mint", mint.key().as_ref()],
        bump = supported_mint.bump,
        constraint = supported_mint.mint == mint.key() @ EscrowError::UnsupportedMint
    )]
    pub supported_mint: Box<Account<'info, SupportedMint>>,

    /// The seller this order is with. `active` is enforced in the handler.
    #[account(
        seeds = [b"merchant", merchant.authority.as_ref()],
        bump = merchant.bump
    )]
    pub merchant: Box<Account<'info, Merchant>>,

    #[account(
        init,
        payer = buyer,
        space = 8 + Order::INIT_SPACE,
        seeds = [b"order", keccak::hash(order_id.as_bytes()).0.as_ref()],
        bump
    )]
    pub order: Box<Account<'info, Order>>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    pub mint: Box<Account<'info, Mint>>,

    /// Buyer's source token account: must be theirs and hold the order's mint.
    #[account(
        mut,
        constraint = buyer_token_account.owner == buyer.key() @ EscrowError::InvalidTokenAccountOwner,
        constraint = buyer_token_account.mint == mint.key() @ EscrowError::InvalidMint
    )]
    pub buyer_token_account: Box<Account<'info, TokenAccount>>,

    /// Escrow vault: the associated token account of the order PDA for `mint`.
    /// `init` guarantees a fresh, program-controlled vault with the right authority
    /// and mint, so no pre-existing attacker-owned account can be passed instead.
    #[account(
        init,
        payer = buyer,
        associated_token::mint = mint,
        associated_token::authority = order
    )]
    pub escrow_token_account: Box<Account<'info, TokenAccount>>,

    /// Validated at creation time so an order can never be funded into a state where
    /// the seller has no place to be paid. Read-only here.
    #[account(
        associated_token::mint = mint,
        associated_token::authority = payout_wallet
    )]
    pub seller_token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: constrained to equal `merchant.payout_wallet`, and only used as the
    /// ATA authority for `seller_token_account`.
    #[account(constraint = payout_wallet.key() == merchant.payout_wallet @ EscrowError::InvalidPayoutDestination)]
    pub payout_wallet: UncheckedAccount<'info>,

    #[account(
        associated_token::mint = mint,
        associated_token::authority = treasury
    )]
    pub treasury_token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: constrained to equal `config.treasury`.
    #[account(constraint = treasury.key() == config.treasury @ EscrowError::InvalidPayoutDestination)]
    pub treasury: UncheckedAccount<'info>,

    /// CHECK: the Instructions sysvar, read to locate the Ed25519 quote attestation.
    #[account(address = INSTRUCTIONS_SYSVAR_ID @ EscrowError::MissingQuoteSignature)]
    pub instructions_sysvar: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ConfirmOrder<'info> {
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [b"order", order.order_id_hash.as_ref()],
        bump = order.bump,
        has_one = seller @ EscrowError::Unauthorized
    )]
    pub order: Account<'info, Order>,

    #[account(
        seeds = [b"merchant", order.seller.as_ref()],
        bump = merchant.bump,
        constraint = merchant.authority == order.seller @ EscrowError::Unauthorized
    )]
    pub merchant: Account<'info, Merchant>,

    /// The seller themself. Must match `order.seller` via `has_one` above.
    pub seller: Signer<'info>,
}

#[derive(Accounts)]
pub struct CompleteOrder<'info> {
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [b"order", order.order_id_hash.as_ref()],
        bump = order.bump,
        has_one = mint @ EscrowError::InvalidMint,
        has_one = buyer @ EscrowError::Unauthorized,
        has_one = payout_wallet @ EscrowError::InvalidPayoutDestination,
        has_one = treasury @ EscrowError::InvalidPayoutDestination
    )]
    pub order: Account<'info, Order>,

    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = order
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    /// Seller proceeds land here: the ATA of the payout wallet recorded on the order.
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = payout_wallet
    )]
    pub seller_token_account: Account<'info, TokenAccount>,

    /// CHECK: pinned to `order.payout_wallet` by `has_one`.
    pub payout_wallet: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = treasury
    )]
    pub treasury_token_account: Account<'info, TokenAccount>,

    /// CHECK: pinned to `order.treasury` by `has_one`.
    pub treasury: UncheckedAccount<'info>,

    /// Receives the escrow vault's rent when it is closed. Pinned to `order.buyer`.
    /// CHECK: validated by `has_one = buyer`.
    #[account(mut)]
    pub buyer: UncheckedAccount<'info>,

    /// Must be `order.seller` or `config.admin` (checked in the handler).
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct RefundOrder<'info> {
    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [b"order", order.order_id_hash.as_ref()],
        bump = order.bump,
        has_one = mint @ EscrowError::InvalidMint,
        has_one = buyer @ EscrowError::Unauthorized
    )]
    pub order: Account<'info, Order>,

    pub mint: Account<'info, Mint>,

    /// The order's recorded buyer. Does not sign (a seller or admin may refund on
    /// their behalf), but every refund destination is derived from this key.
    /// CHECK: validated by `has_one = buyer`.
    #[account(mut)]
    pub buyer: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = order
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    /// Refunds always go to the buyer's ATA for the order's mint -- never to an
    /// account chosen by whoever is signing.
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = buyer
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,

    /// Authorisation depends on the instruction and the order status; see handlers.
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

// =========================================================================
// State
// =========================================================================

#[account]
#[derive(InitSpace)]
pub struct Config {
    /// Governance key. See the module docs for exactly what it can and cannot do.
    pub admin: Pubkey,
    /// Nominee for a two-step admin handover.
    pub pending_admin: Option<Pubkey>,
    /// Ed25519 key whose signature authorises an order's price. Hot key; rotatable.
    pub quote_authority: Pubkey,
    /// Wallet whose ATA receives platform fees.
    pub treasury: Pubkey,
    /// How long after creation an order stays refundable / before a buyer may
    /// unilaterally time out.
    pub refund_window_secs: i64,
    /// When true, no new orders may be created. Settlement and refunds continue.
    pub paused: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct SupportedMint {
    pub mint: Pubkey,
    /// Snapshot of the mint's decimals, so off-chain consumers do not have to guess.
    pub decimals: u8,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Merchant {
    /// The seller's identity key: signs `confirm_order` and may complete/refund.
    pub authority: Pubkey,
    /// Wallet whose ATA receives this seller's proceeds.
    pub payout_wallet: Pubkey,
    pub active: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Order {
    /// Funded the escrow; the only valid refund recipient.
    pub buyer: Pubkey,
    /// `Merchant.authority` this order is with.
    pub seller: Pubkey,
    /// SPL mint held in escrow.
    pub mint: Pubkey,
    /// Snapshot of `Merchant.payout_wallet` -- the only valid payout recipient.
    pub payout_wallet: Pubkey,
    /// Snapshot of `Config.treasury` -- the only valid fee recipient.
    pub treasury: Pubkey,
    /// Total escrowed base units (quoted, signed, never buyer-chosen).
    pub amount: u64,
    /// Platform fee in base units, carved out of `amount` at payout.
    pub fee_amount: u64,
    pub status: OrderStatus,
    pub created_at: i64,
    /// Refunds allowed up to here; the buyer's timeout exit opens after it.
    pub refund_deadline: i64,
    /// Expiry of the quote that authorised this order (recorded for audit).
    pub quote_expiry: i64,
    /// keccak-256 of `order_id`; the PDA seed component.
    pub order_id_hash: [u8; 32],
    /// Full off-chain order id (UUID etc.); stored, never used as a seed.
    /// Kept in sync with [`MAX_ORDER_ID_LEN`] by `account_sizes_match_their_field_layout`.
    #[max_len(64)]
    pub order_id: String,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Completed,
    Refunded,
}

/// Borsh serialises a fieldless enum as a single discriminant byte. Spelled out
/// explicitly rather than derived, so the `Order` account's size calculation does
/// not depend on a derive macro's treatment of enums.
impl anchor_lang::Space for OrderStatus {
    const INIT_SPACE: usize = 1;
}

// =========================================================================
// Events
// =========================================================================

#[event]
pub struct ConfigUpdated {
    pub admin: Pubkey,
    pub quote_authority: Pubkey,
    pub treasury: Pubkey,
    pub refund_window_secs: i64,
    pub paused: bool,
}

#[event]
pub struct MintRegistered {
    pub mint: Pubkey,
    pub decimals: u8,
}

#[event]
pub struct MintRevoked {
    pub mint: Pubkey,
}

#[event]
pub struct MerchantUpdated {
    pub merchant: Pubkey,
    pub payout_wallet: Pubkey,
    pub active: bool,
}

#[event]
pub struct OrderCreated {
    pub order_id_hash: [u8; 32],
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub fee_amount: u64,
    pub refund_deadline: i64,
}

#[event]
pub struct OrderConfirmed {
    pub order_id_hash: [u8; 32],
    pub seller: Pubkey,
}

#[event]
pub struct OrderCompleted {
    pub order_id_hash: [u8; 32],
    pub seller: Pubkey,
    pub seller_amount: u64,
    pub treasury_amount: u64,
}

#[event]
pub struct OrderRefunded {
    pub order_id_hash: [u8; 32],
    pub buyer: Pubkey,
    pub amount: u64,
    /// True when the refund came from the buyer's unilateral timeout exit.
    pub timed_out: bool,
}

// =========================================================================
// Errors
// =========================================================================

#[error_code]
pub enum EscrowError {
    #[msg("Invalid order status for this operation")]
    InvalidStatus,
    #[msg("Cannot refund this order")]
    CannotRefund,
    #[msg("Refund window has expired")]
    RefundWindowExpired,
    #[msg("The refund deadline has not been reached yet")]
    TimeoutNotReached,
    #[msg("Unauthorized: signer is not the required authority")]
    Unauthorized,
    #[msg("Order id must not be empty")]
    InvalidOrderId,
    #[msg("Order id exceeds the maximum allowed length")]
    OrderIdTooLong,
    #[msg("Amount must be greater than zero")]
    InvalidAmount,
    #[msg("Fee must be smaller than the escrowed amount")]
    InvalidFeeAmount,
    #[msg("Token account owner does not match")]
    InvalidTokenAccountOwner,
    #[msg("Token account mint does not match the order mint")]
    InvalidMint,
    #[msg("This mint is not on the escrow allow-list")]
    UnsupportedMint,
    #[msg("Payout or refund destination does not match the one recorded on the order")]
    InvalidPayoutDestination,
    #[msg("Merchant is not active")]
    MerchantInactive,
    #[msg("New order creation is paused")]
    Paused,
    #[msg("The signed quote has expired")]
    QuoteExpired,
    #[msg("The signed quote is valid for too long")]
    QuoteLifetimeTooLong,
    #[msg("No Ed25519 quote attestation precedes this instruction")]
    MissingQuoteSignature,
    #[msg("The Ed25519 quote attestation is malformed")]
    MalformedQuoteSignature,
    #[msg("The quote was signed by a key other than the configured quote authority")]
    UnauthorizedQuoteSigner,
    #[msg("The signed quote does not match this order")]
    QuoteMismatch,
    #[msg("Escrow vault holds less than the order amount")]
    InsufficientEscrow,
    #[msg("Refund window is outside the permitted bounds")]
    InvalidRefundWindow,
    #[msg("No pending admin has been nominated")]
    NoPendingAdmin,
    #[msg("Authority key cannot be the default pubkey")]
    InvalidAuthority,
    #[msg("Arithmetic overflow")]
    MathOverflow,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Account sizes are security-relevant: too small and `init` fails or data is
    /// truncated, too large and every order overpays rent. These assertions pin the
    /// layout so a field added later cannot silently break allocation.
    #[test]
    fn account_sizes_match_their_field_layout() {
        // admin(32) + pending_admin Option<Pubkey>(1+32) + quote_authority(32)
        // + treasury(32) + refund_window_secs(8) + paused(1) + bump(1)
        assert_eq!(Config::INIT_SPACE, 32 + 33 + 32 + 32 + 8 + 1 + 1);

        // mint(32) + decimals(1) + bump(1)
        assert_eq!(SupportedMint::INIT_SPACE, 32 + 1 + 1);

        // authority(32) + payout_wallet(32) + active(1) + bump(1)
        assert_eq!(Merchant::INIT_SPACE, 32 + 32 + 1 + 1);

        // 5 pubkeys + amount(8) + fee(8) + status(1) + 3 timestamps(24)
        // + order_id_hash(32) + String(4 + 64) + bump(1)
        assert_eq!(
            Order::INIT_SPACE,
            32 * 5 + 8 + 8 + 1 + 24 + 32 + (4 + MAX_ORDER_ID_LEN) + 1
        );
    }

    #[test]
    fn refund_window_bounds_are_sane() {
        assert!(MIN_REFUND_WINDOW_SECS < MAX_REFUND_WINDOW_SECS);
        assert!(MAX_QUOTE_LIFETIME_SECS < MIN_REFUND_WINDOW_SECS);
    }
}
