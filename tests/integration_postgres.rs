//! End-to-end tests against a real PostgreSQL instance.
//!
//! They are `#[ignore]`d by default because they need a disposable database. Run:
//!
//! ```text
//! docker run --rm -d --name aeroledger-test -e POSTGRES_PASSWORD=postgres -p 55432:5432 postgres:16
//! TEST_DATABASE_URL=postgres://postgres:postgres@127.0.0.1:55432/postgres \
//!   cargo test --test integration_postgres -- --ignored --test-threads=1
//! ```
//!
//! What they cover that unit tests cannot: the quote state machine under real
//! unique indexes, the atomicity of settlement, and replay protection enforced by
//! the database rather than by application code.

use std::env;
use std::sync::Arc;
use uuid::Uuid;

use areo_ledger::application::dto::{
    AcceptQuoteInput, AuthenticateUserInput, ConfirmPaymentInput, CreateBookingInput,
    CreateQuoteInput, CreateUserInput, InitPaymentInput,
};
use areo_ledger::application::use_cases::accept_quote::AcceptQuote;
use areo_ledger::application::use_cases::authenticate_user::AuthenticateUser;
use areo_ledger::application::use_cases::confirm_payment::ConfirmPayment;
use areo_ledger::application::use_cases::create_booking::CreateBooking;
use areo_ledger::application::use_cases::create_quote::CreateQuote;
use areo_ledger::application::use_cases::create_user::CreateUser;
use areo_ledger::application::use_cases::get_transactions::GetTransactions;
use areo_ledger::application::use_cases::init_payment::InitPayment;
use areo_ledger::domain::ports::blockchain_gateway::{
    BlockchainGateway, TxVerification, VerifyRequest,
};
use areo_ledger::domain::ports::price_oracle::{PriceOracle, TokenRate};
use areo_ledger::domain::value_objects::CryptoToken;
use areo_ledger::infrastructure::db::{init_pool, run_migrations};
use areo_ledger::infrastructure::persistence::postgres::pg_booking_repository::PgBookingRepository;
use areo_ledger::infrastructure::persistence::postgres::pg_catalog_repository::PgCatalogRepository;
use areo_ledger::infrastructure::persistence::postgres::pg_payment_repository::PgPaymentRepository;
use areo_ledger::infrastructure::persistence::postgres::pg_quote_repository::PgQuoteRepository;
use areo_ledger::infrastructure::persistence::postgres::pg_settlement_repository::PgSettlementRepository;
use areo_ledger::infrastructure::persistence::postgres::pg_transaction_repository::PgTransactionRepository;
use areo_ledger::infrastructure::persistence::postgres::pg_user_repository::PgUserRepository;
use areo_ledger::shared::errors::DomainError;
use async_trait::async_trait;
use rust_decimal::Decimal;

const FLIGHT_ID: &str = "00000000-0000-0000-0000-000000000001";

/// Gateway that reports exactly the expected amount landing on the platform wallet.
#[derive(Clone)]
struct TestGateway;

#[async_trait]
impl BlockchainGateway for TestGateway {
    fn receiving_address(&self, token: &str) -> Result<String, DomainError> {
        Ok(format!("test-wallet-{}", token))
    }

    async fn verify_transaction(
        &self,
        request: &VerifyRequest,
    ) -> Result<TxVerification, DomainError> {
        Ok(TxVerification {
            // This stub only ever reports settled transactions, so what it sees
            // is final by construction.
            observed: true,
            verified: true,
            base_units: request.expected_base_units,
            confirmations: 99,
            from_address: "test-sender".to_string(),
            to_address: request.expected_to.clone(),
        })
    }
}

struct TestOracle;

#[async_trait]
impl PriceOracle for TestOracle {
    async fn rate(&self, token: CryptoToken) -> Result<TokenRate, DomainError> {
        Ok(TokenRate {
            unit_price_usd: if token.is_stablecoin() {
                Decimal::ONE
            } else {
                Decimal::from(2500)
            },
            source: "test".to_string(),
            observed_at: chrono::Utc::now(),
        })
    }
}

async fn setup() -> Option<deadpool_postgres::Pool> {
    let url = env::var("TEST_DATABASE_URL").ok()?;
    let pool = init_pool(&url);
    run_migrations(&pool).await.ok()?;
    let client = pool.get().await.ok()?;
    // Preserve the seeded flights/assets catalogue; only clear per-user state.
    let _ = client
        .batch_execute(
            "TRUNCATE TABLE audit_log, payment_quotes, payments, bookings, transactions, users \
             RESTART IDENTITY CASCADE",
        )
        .await;
    Some(pool)
}

async fn make_user(pool: &deadpool_postgres::Pool, tag: &str) -> Uuid {
    CreateUser::new(PgUserRepository::new(pool.clone()))
        .execute(CreateUserInput {
            email: format!("{}-{}@example.com", tag, Uuid::new_v4()),
            name: "Integration User".to_string(),
            password: "Password123!".to_string(),
        })
        .await
        .unwrap()
        .id
}

async fn make_booking(pool: &deadpool_postgres::Pool, user_id: Uuid) -> Uuid {
    CreateBooking::new(
        PgBookingRepository::new(pool.clone()),
        PgCatalogRepository::new(pool.clone()),
    )
    .execute(CreateBookingInput {
        user_id,
        flight_id: Some(Uuid::parse_str(FLIGHT_ID).unwrap()),
        asset_id: None,
        payment_method: "eth".to_string(),
    })
    .await
    .unwrap()
    .booking_id
}

async fn quote_for(
    pool: &deadpool_postgres::Pool,
    booking_id: Uuid,
    user_id: Uuid,
) -> areo_ledger::application::dto::QuoteView {
    CreateQuote::new(
        PgBookingRepository::new(pool.clone()),
        PgQuoteRepository::new(pool.clone()),
        PgPaymentRepository::new(pool.clone()),
        PgCatalogRepository::new(pool.clone()),
        TestGateway,
        Arc::new(TestOracle),
    )
    .execute(CreateQuoteInput {
        booking_id,
        user_id,
        method: "eth".to_string(),
    })
    .await
    .unwrap()
}

async fn set_flight_price(pool: &deadpool_postgres::Pool, dollars: &str) {
    let client = pool.get().await.unwrap();
    client
        .execute(
            "UPDATE flights SET price_usd = ($1::TEXT)::DECIMAL(15,2) WHERE id = $2",
            &[&dollars, &Uuid::parse_str(FLIGHT_ID).unwrap()],
        )
        .await
        .unwrap();
}

// =====================================================================
// Happy path
// =====================================================================

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing at a disposable Postgres"]
async fn quote_accept_pay_confirm_end_to_end() {
    let pool = setup()
        .await
        .expect("TEST_DATABASE_URL must be set for integration tests");
    set_flight_price(&pool, "9500.00").await;

    let user_id = make_user(&pool, "happy").await;
    let authed = AuthenticateUser::new(PgUserRepository::new(pool.clone()))
        .execute(AuthenticateUserInput {
            email: {
                let client = pool.get().await.unwrap();
                let row = client
                    .query_one("SELECT email FROM users WHERE id = $1", &[&user_id])
                    .await
                    .unwrap();
                row.get::<_, String>(0)
            },
            password: "Password123!".to_string(),
        })
        .await
        .unwrap();
    assert_eq!(authed.id, user_id);

    let booking_id = make_booking(&pool, user_id).await;

    // $9,500 base + 10% markup + 5% fee = $10,925.00.
    let quote = quote_for(&pool, booking_id, user_id).await;
    assert_eq!(quote.total_cents, 1_092_500);
    assert_eq!(quote.token, "ETH");
    assert!(!quote.price_changed);
    // $10,925 / $2,500 = 4.37 ETH = 4_370_000_000_000_000_000 wei.
    assert_eq!(quote.crypto_amount, "4.37");
    assert_eq!(quote.crypto_base_units, "4370000000000000000");
    assert_eq!(quote.status, "active");

    // Paying before approving the price must be refused.
    let err = InitPayment::new(
        PgBookingRepository::new(pool.clone()),
        PgPaymentRepository::new(pool.clone()),
        PgQuoteRepository::new(pool.clone()),
    )
    .execute(InitPaymentInput { booking_id })
    .await
    .unwrap_err();
    assert!(matches!(err, DomainError::Conflict(_)));

    let accepted = AcceptQuote::new(PgQuoteRepository::new(pool.clone()))
        .execute(AcceptQuoteInput {
            quote_id: quote.quote_id,
            booking_id,
            user_id,
            approved_total_cents: quote.total_cents,
            approved_crypto_base_units: quote.crypto_base_units.clone(),
        })
        .await
        .unwrap();
    assert_eq!(accepted.status, "accepted");

    let init = InitPayment::new(
        PgBookingRepository::new(pool.clone()),
        PgPaymentRepository::new(pool.clone()),
        PgQuoteRepository::new(pool.clone()),
    )
    .execute(InitPaymentInput { booking_id })
    .await
    .unwrap();
    assert_eq!(init.crypto_base_units, quote.crypto_base_units);
    assert_eq!(init.total_cents, quote.total_cents);
    assert_eq!(init.quote_id, quote.quote_id);

    ConfirmPayment::new(
        PgPaymentRepository::new(pool.clone()),
        PgSettlementRepository::new(pool.clone()),
        TestGateway,
    )
    .execute(ConfirmPaymentInput {
        booking_id,
        tx_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
    })
    .await
    .unwrap();

    let txs = GetTransactions::new(PgTransactionRepository::new(pool.clone()))
        .execute(user_id)
        .await
        .unwrap();
    assert!(!txs.is_empty(), "settlement must write a ledger row");
}

// =====================================================================
// Price changes (interviewer question 2)
// =====================================================================

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing at a disposable Postgres"]
async fn a_price_change_between_booking_and_checkout_is_surfaced_and_must_be_re_approved() {
    let pool = setup().await.expect("TEST_DATABASE_URL must be set");
    set_flight_price(&pool, "5000.00").await;

    let user_id = make_user(&pool, "reprice").await;
    let booking_id = make_booking(&pool, user_id).await;

    let first = quote_for(&pool, booking_id, user_id).await;
    assert_eq!(first.total_cents, 575_000); // $5,000 + 15%
    assert!(!first.price_changed);

    // The provider re-prices the flight while the user sits on the checkout page.
    set_flight_price(&pool, "5500.00").await;

    let second = quote_for(&pool, booking_id, user_id).await;
    assert!(second.price_changed, "the UI must be told the price moved");
    assert_eq!(second.previous_total_cents, Some(575_000));
    assert_eq!(second.total_cents, 632_500); // $5,500 + 15%

    // Accepting the OLD figures must fail: the user never saw the new price.
    let err = AcceptQuote::new(PgQuoteRepository::new(pool.clone()))
        .execute(AcceptQuoteInput {
            quote_id: second.quote_id,
            booking_id,
            user_id,
            approved_total_cents: first.total_cents,
            approved_crypto_base_units: second.crypto_base_units.clone(),
        })
        .await
        .unwrap_err();
    assert!(matches!(err, DomainError::Conflict(_)));

    // The superseded quote cannot be accepted either, so the cheaper price is gone.
    let err = AcceptQuote::new(PgQuoteRepository::new(pool.clone()))
        .execute(AcceptQuoteInput {
            quote_id: first.quote_id,
            booking_id,
            user_id,
            approved_total_cents: first.total_cents,
            approved_crypto_base_units: first.crypto_base_units.clone(),
        })
        .await
        .unwrap_err();
    assert!(matches!(err, DomainError::Conflict(_)));

    // Approving the NEW figures works, and locks them.
    let accepted = AcceptQuote::new(PgQuoteRepository::new(pool.clone()))
        .execute(AcceptQuoteInput {
            quote_id: second.quote_id,
            booking_id,
            user_id,
            approved_total_cents: second.total_cents,
            approved_crypto_base_units: second.crypto_base_units.clone(),
        })
        .await
        .unwrap();
    assert_eq!(accepted.status, "accepted");

    let init = InitPayment::new(
        PgBookingRepository::new(pool.clone()),
        PgPaymentRepository::new(pool.clone()),
        PgQuoteRepository::new(pool.clone()),
    )
    .execute(InitPaymentInput { booking_id })
    .await
    .unwrap();
    assert_eq!(init.total_cents, 632_500);
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing at a disposable Postgres"]
async fn an_expired_quote_cannot_be_accepted() {
    let pool = setup().await.expect("TEST_DATABASE_URL must be set");
    set_flight_price(&pool, "1000.00").await;

    let user_id = make_user(&pool, "expiry").await;
    let booking_id = make_booking(&pool, user_id).await;
    let quote = quote_for(&pool, booking_id, user_id).await;

    // Age the whole row past its TTL rather than sleeping ten minutes. Both
    // timestamps move, because the table enforces `expires_at > created_at`.
    let client = pool.get().await.unwrap();
    client
        .execute(
            "UPDATE payment_quotes
             SET created_at = NOW() - INTERVAL '20 minutes',
                 expires_at = NOW() - INTERVAL '10 minutes'
             WHERE id = $1",
            &[&quote.quote_id],
        )
        .await
        .unwrap();

    let err = AcceptQuote::new(PgQuoteRepository::new(pool.clone()))
        .execute(AcceptQuoteInput {
            quote_id: quote.quote_id,
            booking_id,
            user_id,
            approved_total_cents: quote.total_cents,
            approved_crypto_base_units: quote.crypto_base_units.clone(),
        })
        .await
        .unwrap_err();
    assert!(matches!(err, DomainError::Conflict(_)));

    // And no payment can be created from it.
    let err = InitPayment::new(
        PgBookingRepository::new(pool.clone()),
        PgPaymentRepository::new(pool.clone()),
        PgQuoteRepository::new(pool.clone()),
    )
    .execute(InitPaymentInput { booking_id })
    .await
    .unwrap_err();
    assert!(matches!(err, DomainError::Conflict(_)));
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing at a disposable Postgres"]
async fn only_one_quote_per_booking_is_ever_active() {
    let pool = setup().await.expect("TEST_DATABASE_URL must be set");
    set_flight_price(&pool, "1000.00").await;

    let user_id = make_user(&pool, "single").await;
    let booking_id = make_booking(&pool, user_id).await;

    for _ in 0..3 {
        quote_for(&pool, booking_id, user_id).await;
    }

    let client = pool.get().await.unwrap();
    let row = client
        .query_one(
            "SELECT COUNT(*) FROM payment_quotes WHERE booking_id = $1 AND status = 'active'",
            &[&booking_id],
        )
        .await
        .unwrap();
    assert_eq!(row.get::<_, i64>(0), 1);

    let row = client
        .query_one(
            "SELECT COUNT(*) FROM payment_quotes WHERE booking_id = $1 AND status = 'superseded'",
            &[&booking_id],
        )
        .await
        .unwrap();
    assert_eq!(row.get::<_, i64>(0), 2);
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing at a disposable Postgres"]
async fn a_booking_cannot_be_re_quoted_after_the_price_is_locked() {
    let pool = setup().await.expect("TEST_DATABASE_URL must be set");
    set_flight_price(&pool, "1000.00").await;

    let user_id = make_user(&pool, "locked-quote").await;
    let booking_id = make_booking(&pool, user_id).await;
    let quote = quote_for(&pool, booking_id, user_id).await;

    AcceptQuote::new(PgQuoteRepository::new(pool.clone()))
        .execute(AcceptQuoteInput {
            quote_id: quote.quote_id,
            booking_id,
            user_id,
            approved_total_cents: quote.total_cents,
            approved_crypto_base_units: quote.crypto_base_units.clone(),
        })
        .await
        .unwrap();

    // Acceptance is the lock. A second quote would leave two prices for one booking
    // and collide with `idx_payment_quotes_one_accepted`; the use case must turn
    // that into a clean conflict rather than a raw database error.
    let err = CreateQuote::new(
        PgBookingRepository::new(pool.clone()),
        PgQuoteRepository::new(pool.clone()),
        PgPaymentRepository::new(pool.clone()),
        PgCatalogRepository::new(pool.clone()),
        TestGateway,
        Arc::new(TestOracle),
    )
    .execute(CreateQuoteInput {
        booking_id,
        user_id,
        method: "eth".to_string(),
    })
    .await
    .unwrap_err();
    assert!(matches!(err, DomainError::Conflict(_)));

    // The locked price is still settleable afterwards.
    let init = InitPayment::new(
        PgBookingRepository::new(pool.clone()),
        PgPaymentRepository::new(pool.clone()),
        PgQuoteRepository::new(pool.clone()),
    )
    .execute(InitPaymentInput { booking_id })
    .await
    .unwrap();
    assert_eq!(init.crypto_base_units, quote.crypto_base_units);
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing at a disposable Postgres"]
async fn a_quote_cannot_be_accepted_by_another_user() {
    let pool = setup().await.expect("TEST_DATABASE_URL must be set");
    set_flight_price(&pool, "1000.00").await;

    let owner = make_user(&pool, "owner").await;
    let intruder = make_user(&pool, "intruder").await;
    let booking_id = make_booking(&pool, owner).await;
    let quote = quote_for(&pool, booking_id, owner).await;

    let err = AcceptQuote::new(PgQuoteRepository::new(pool.clone()))
        .execute(AcceptQuoteInput {
            quote_id: quote.quote_id,
            booking_id,
            user_id: intruder,
            approved_total_cents: quote.total_cents,
            approved_crypto_base_units: quote.crypto_base_units.clone(),
        })
        .await
        .unwrap_err();
    assert!(matches!(err, DomainError::Unauthorized));
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing at a disposable Postgres"]
async fn another_user_cannot_quote_someone_elses_booking() {
    let pool = setup().await.expect("TEST_DATABASE_URL must be set");
    set_flight_price(&pool, "1000.00").await;

    let owner = make_user(&pool, "owner2").await;
    let intruder = make_user(&pool, "intruder2").await;
    let booking_id = make_booking(&pool, owner).await;

    let err = CreateQuote::new(
        PgBookingRepository::new(pool.clone()),
        PgQuoteRepository::new(pool.clone()),
        PgPaymentRepository::new(pool.clone()),
        PgCatalogRepository::new(pool.clone()),
        TestGateway,
        Arc::new(TestOracle),
    )
    .execute(CreateQuoteInput {
        booking_id,
        user_id: intruder,
        method: "eth".to_string(),
    })
    .await
    .unwrap_err();
    assert!(matches!(err, DomainError::Unauthorized));
}

// =====================================================================
// Replay / idempotency
// =====================================================================

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing at a disposable Postgres"]
async fn a_transaction_hash_cannot_settle_two_bookings() {
    let pool = setup().await.expect("TEST_DATABASE_URL must be set");
    set_flight_price(&pool, "1000.00").await;

    let user_id = make_user(&pool, "replay").await;

    let mut bookings = vec![];
    for _ in 0..2 {
        let booking_id = make_booking(&pool, user_id).await;
        let quote = quote_for(&pool, booking_id, user_id).await;
        AcceptQuote::new(PgQuoteRepository::new(pool.clone()))
            .execute(AcceptQuoteInput {
                quote_id: quote.quote_id,
                booking_id,
                user_id,
                approved_total_cents: quote.total_cents,
                approved_crypto_base_units: quote.crypto_base_units.clone(),
            })
            .await
            .unwrap();
        InitPayment::new(
            PgBookingRepository::new(pool.clone()),
            PgPaymentRepository::new(pool.clone()),
            PgQuoteRepository::new(pool.clone()),
        )
        .execute(InitPaymentInput { booking_id })
        .await
        .unwrap();
        bookings.push(booking_id);
    }

    let tx = "0xabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcab";

    ConfirmPayment::new(
        PgPaymentRepository::new(pool.clone()),
        PgSettlementRepository::new(pool.clone()),
        TestGateway,
    )
    .execute(ConfirmPaymentInput {
        booking_id: bookings[0],
        tx_hash: tx.to_string(),
    })
    .await
    .unwrap();

    let err = ConfirmPayment::new(
        PgPaymentRepository::new(pool.clone()),
        PgSettlementRepository::new(pool.clone()),
        TestGateway,
    )
    .execute(ConfirmPaymentInput {
        booking_id: bookings[1],
        tx_hash: tx.to_string(),
    })
    .await
    .unwrap_err();
    assert!(matches!(err, DomainError::Conflict(_)));
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing at a disposable Postgres"]
async fn confirming_the_same_payment_twice_settles_once() {
    let pool = setup().await.expect("TEST_DATABASE_URL must be set");
    set_flight_price(&pool, "1000.00").await;

    let user_id = make_user(&pool, "idem").await;
    let booking_id = make_booking(&pool, user_id).await;
    let quote = quote_for(&pool, booking_id, user_id).await;
    AcceptQuote::new(PgQuoteRepository::new(pool.clone()))
        .execute(AcceptQuoteInput {
            quote_id: quote.quote_id,
            booking_id,
            user_id,
            approved_total_cents: quote.total_cents,
            approved_crypto_base_units: quote.crypto_base_units.clone(),
        })
        .await
        .unwrap();
    InitPayment::new(
        PgBookingRepository::new(pool.clone()),
        PgPaymentRepository::new(pool.clone()),
        PgQuoteRepository::new(pool.clone()),
    )
    .execute(InitPaymentInput { booking_id })
    .await
    .unwrap();

    let tx = "0xfeedfeedfeedfeedfeedfeedfeedfeedfeedfeedfeedfeedfeedfeedfeedfeed";
    for _ in 0..3 {
        ConfirmPayment::new(
            PgPaymentRepository::new(pool.clone()),
            PgSettlementRepository::new(pool.clone()),
            TestGateway,
        )
        .execute(ConfirmPaymentInput {
            booking_id,
            tx_hash: tx.to_string(),
        })
        .await
        .unwrap();
    }

    // Exactly one ledger row, no matter how many times the client retried.
    let client = pool.get().await.unwrap();
    let row = client
        .query_one(
            "SELECT COUNT(*) FROM transactions WHERE user_id = $1 AND tx_hash = $2",
            &[&user_id, &tx],
        )
        .await
        .unwrap();
    assert_eq!(row.get::<_, i64>(0), 1);
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing at a disposable Postgres"]
async fn a_booking_cannot_be_re_quoted_once_payment_is_initialised() {
    let pool = setup().await.expect("TEST_DATABASE_URL must be set");
    set_flight_price(&pool, "1000.00").await;

    let user_id = make_user(&pool, "locked").await;
    let booking_id = make_booking(&pool, user_id).await;
    let quote = quote_for(&pool, booking_id, user_id).await;
    AcceptQuote::new(PgQuoteRepository::new(pool.clone()))
        .execute(AcceptQuoteInput {
            quote_id: quote.quote_id,
            booking_id,
            user_id,
            approved_total_cents: quote.total_cents,
            approved_crypto_base_units: quote.crypto_base_units.clone(),
        })
        .await
        .unwrap();
    InitPayment::new(
        PgBookingRepository::new(pool.clone()),
        PgPaymentRepository::new(pool.clone()),
        PgQuoteRepository::new(pool.clone()),
    )
    .execute(InitPaymentInput { booking_id })
    .await
    .unwrap();

    // Re-pricing after the amount is locked would create two conflicting prices.
    set_flight_price(&pool, "2000.00").await;
    let err = CreateQuote::new(
        PgBookingRepository::new(pool.clone()),
        PgQuoteRepository::new(pool.clone()),
        PgPaymentRepository::new(pool.clone()),
        PgCatalogRepository::new(pool.clone()),
        TestGateway,
        Arc::new(TestOracle),
    )
    .execute(CreateQuoteInput {
        booking_id,
        user_id,
        method: "eth".to_string(),
    })
    .await
    .unwrap_err();
    assert!(matches!(err, DomainError::Conflict(_)));
}
