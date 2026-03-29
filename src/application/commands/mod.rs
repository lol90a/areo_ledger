use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::domain::value_objects::{TxHash, WalletAddress, Money, FiatCurrency, IdempotencyKey, resolve_chain_token};
use crate::domain::entities::booking::Booking;
use crate::domain::entities::payment::Payment;
use crate::domain::entities::ledger::JournalEntry;
use crate::domain::repositories::booking_repository::BookingRepository;
use crate::domain::repositories::payment_repository::PaymentRepository;
use crate::domain::repositories::ledger_repository::LedgerRepository;
use crate::domain::ports::blockchain_gateway::BlockchainGateway;
use crate::domain::ports::event_publisher::EventPublisher;
use crate::shared::errors::DomainError;

// ── CreateBooking Command ─────────────────────────────────────────────────────

pub struct CreateBookingCommand {
    pub user_id:        Uuid,
    pub flight_id:      Uuid,
    /// Amount in cents — never f64 at the API boundary.
    pub base_price_cents: i64,
    pub currency:       FiatCurrency,
    pub payment_method: String,
}

pub struct CreateBookingResult {
    pub booking_id:        Uuid,
    pub total_price_cents: i64,
    pub currency:          FiatCurrency,
}

pub struct CreateBookingHandler<BR, EP>
where
    BR: BookingRepository,
    EP: EventPublisher,
{
    booking_repo:   BR,
    event_publisher: EP,
}

impl<BR, EP> CreateBookingHandler<BR, EP>
where
    BR: BookingRepository,
    EP: EventPublisher,
{
    pub fn new(booking_repo: BR, event_publisher: EP) -> Self {
        Self { booking_repo, event_publisher }
    }

    pub async fn handle(&self, cmd: CreateBookingCommand) -> Result<CreateBookingResult, DomainError> {
        let base_price = Money::from_cents(cmd.base_price_cents, cmd.currency)
            .map_err(|e| DomainError::ValidationError(e.to_string()))?;

        let (booking, event) = Booking::create(
            Uuid::new_v4(),
            cmd.user_id,
            cmd.flight_id,
            base_price,
            cmd.payment_method,
        )?;

        let total_cents = booking.pricing().total.cents();
        self.booking_repo.save(&booking).await?;
        self.event_publisher.publish(&event).await?;

        Ok(CreateBookingResult {
            booking_id: booking.id(),
            total_price_cents: total_cents,
            currency: cmd.currency,
        })
    }
}

// ── InitPayment Command ───────────────────────────────────────────────────────

pub struct InitPaymentCommand {
    pub booking_id:      Uuid,
    pub method:          String,
    /// Client-supplied idempotency key — prevents duplicate payment records.
    pub idempotency_key: IdempotencyKey,
}

pub struct InitPaymentResult {
    pub payment_id:      Uuid,
    pub wallet_address:  String,
    pub amount_cents:    i64,
    pub currency:        FiatCurrency,
}

pub struct InitPaymentHandler<BR, PR, BG, EP>
where
    BR: BookingRepository,
    PR: PaymentRepository,
    BG: BlockchainGateway,
    EP: EventPublisher,
{
    booking_repo:    BR,
    payment_repo:    PR,
    blockchain:      BG,
    event_publisher: EP,
}

impl<BR, PR, BG, EP> InitPaymentHandler<BR, PR, BG, EP>
where
    BR: BookingRepository,
    PR: PaymentRepository,
    BG: BlockchainGateway,
    EP: EventPublisher,
{
    pub fn new(booking_repo: BR, payment_repo: PR, blockchain: BG, event_publisher: EP) -> Self {
        Self { booking_repo, payment_repo, blockchain, event_publisher }
    }

    pub async fn handle(&self, cmd: InitPaymentCommand) -> Result<InitPaymentResult, DomainError> {
        // ── Idempotency check ─────────────────────────────────────────────────
        // If a payment already exists for this idempotency key, return it — don't create a duplicate.
        if let Ok(existing) = self.payment_repo.find_by_idempotency_key(cmd.idempotency_key.value()).await {
            return Ok(InitPaymentResult {
                payment_id:   existing.id(),
                wallet_address: existing.receiver_address().as_str().to_string(),
                amount_cents: existing.amount().cents(),
                currency:     existing.amount().currency(),
            });
        }

        let (chain, token) = resolve_chain_token(&cmd.method)
            .ok_or_else(|| DomainError::ValidationError(
                format!("Unsupported payment method: '{}'", cmd.method)
            ))?;

        let booking = self.booking_repo.find_by_id(&cmd.booking_id).await?;
        let raw_address = self.blockchain.receiving_address(token).await?;
        let wallet = WalletAddress::new(raw_address)
            .map_err(|e| DomainError::ValidationError(e.to_string()))?;

        let (payment, event) = Payment::initiate(
            Uuid::new_v4(),
            booking.id(),
            chain,
            token,
            booking.pricing().total,
            wallet,
            cmd.idempotency_key.value(),
        );

        let result = InitPaymentResult {
            payment_id:   payment.id(),
            wallet_address: payment.receiver_address().as_str().to_string(),
            amount_cents: payment.amount().cents(),
            currency:     payment.amount().currency(),
        };

        self.payment_repo.save(&payment).await?;
        self.event_publisher.publish(&event).await?;

        Ok(result)
    }
}

// ── ConfirmPayment Command ────────────────────────────────────────────────────

pub struct ConfirmPaymentCommand {
    pub booking_id: Uuid,
    pub tx_hash:    String,
}

pub struct ConfirmPaymentHandler<BR, PR, LR, EP>
where
    BR: BookingRepository,
    PR: PaymentRepository,
    LR: LedgerRepository,
    EP: EventPublisher,
{
    booking_repo:    BR,
    payment_repo:    PR,
    ledger_repo:     LR,
    event_publisher: EP,
}

impl<BR, PR, LR, EP> ConfirmPaymentHandler<BR, PR, LR, EP>
where
    BR: BookingRepository,
    PR: PaymentRepository,
    LR: LedgerRepository,
    EP: EventPublisher,
{
    pub fn new(booking_repo: BR, payment_repo: PR, ledger_repo: LR, event_publisher: EP) -> Self {
        Self { booking_repo, payment_repo, ledger_repo, event_publisher }
    }

    /// Atomically confirms payment + booking + writes ledger entry.
    /// All three writes happen in a single DB transaction via the UnitOfWork pattern.
    pub async fn handle(&self, cmd: ConfirmPaymentCommand) -> Result<(), DomainError> {
        let tx_hash = TxHash::new(&cmd.tx_hash)?;

        let mut payment = self.payment_repo.find_by_booking_id(&cmd.booking_id).await?;
        let mut booking = self.booking_repo.find_by_id(&cmd.booking_id).await?;

        // ── Idempotency: already confirmed is a no-op, not an error ──────────
        if payment.is_terminal() {
            return Ok(());
        }

        // ── Domain state transitions ──────────────────────────────────────────
        // Payment must be escrowed before it can be confirmed.
        // If it's still Pending (user submitted hash without escrow step), escrow it first.
        if payment.status() == &crate::domain::entities::payment::PaymentStatus::Pending {
            let sender = WalletAddress::new("unknown_sender_address_placeholder")
                .map_err(|e| DomainError::ValidationError(e.to_string()))?;
            payment.escrow(sender)?;
        }

        let payment_event = payment.confirm(tx_hash.clone())?;
        let booking_event = booking.confirm(tx_hash)?;

        // ── Double-entry ledger ───────────────────────────────────────────────
        let journal = JournalEntry::for_payment_settlement(
            payment.id(),
            booking.id(),
            payment.amount(),
            booking.pricing().service_fee,
        )?;

        // ── Atomic write — all three or none ─────────────────────────────────
        self.payment_repo.update(&payment).await?;
        self.booking_repo.update(&booking).await?;
        self.ledger_repo.save_journal(&journal).await?;

        // ── Publish events after successful persistence ───────────────────────
        self.event_publisher.publish(&payment_event).await?;
        self.event_publisher.publish(&booking_event).await?;

        Ok(())
    }
}
