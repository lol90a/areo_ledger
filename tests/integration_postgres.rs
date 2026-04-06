use std::env;
use uuid::Uuid;

use areo_ledger::application::dto::{
    AuthenticateUserInput, ConfirmPaymentInput, CreateBookingInput, CreateUserInput,
    InitPaymentInput,
};
use areo_ledger::application::use_cases::authenticate_user::AuthenticateUser;
use areo_ledger::application::use_cases::confirm_payment::ConfirmPayment;
use areo_ledger::application::use_cases::create_booking::CreateBooking;
use areo_ledger::application::use_cases::create_user::CreateUser;
use areo_ledger::application::use_cases::get_transactions::GetTransactions;
use areo_ledger::application::use_cases::init_payment::InitPayment;
use areo_ledger::domain::ports::blockchain_gateway::{BlockchainGateway, TxVerification};
use areo_ledger::infrastructure::db::{init_pool, run_migrations};
use areo_ledger::infrastructure::persistence::postgres::pg_booking_repository::PgBookingRepository;
use areo_ledger::infrastructure::persistence::postgres::pg_payment_repository::PgPaymentRepository;
use areo_ledger::infrastructure::persistence::postgres::pg_transaction_repository::PgTransactionRepository;
use areo_ledger::infrastructure::persistence::postgres::pg_user_repository::PgUserRepository;
use areo_ledger::shared::errors::DomainError;
use async_trait::async_trait;

#[derive(Clone)]
struct TestGateway;

#[async_trait]
impl BlockchainGateway for TestGateway {
    fn receiving_address(&self, token: &str) -> Result<String, DomainError> {
        Ok(format!("test-wallet-{}", token))
    }

    async fn verify_transaction(
        &self,
        _chain: &str,
        _tx_hash: &str,
        expected_to: &str,
        expected_amount_cents: i64,
    ) -> Result<TxVerification, DomainError> {
        Ok(TxVerification {
            verified: true,
            amount_cents: expected_amount_cents,
            confirmations: 99,
            from_address: "test-sender".to_string(),
            to_address: expected_to.to_string(),
        })
    }
}

async fn setup() -> Option<deadpool_postgres::Pool> {
    let url = env::var("TEST_DATABASE_URL").ok()?;
    let pool = init_pool(&url);
    run_migrations(&pool).await.ok()?;
    let client = pool.get().await.ok()?;
    let _ = client.batch_execute(
        "TRUNCATE TABLE audit_log, payments, bookings, transactions, users RESTART IDENTITY CASCADE"
    ).await;
    Some(pool)
}

#[tokio::test]
#[ignore]
async fn postgres_happy_path_signup_booking_payment_confirm() {
    let pool = setup()
        .await
        .expect("TEST_DATABASE_URL must be set for integration tests");

    let user = CreateUser::new(PgUserRepository::new(pool.clone()))
        .execute(CreateUserInput {
            email: format!("integration-{}@example.com", Uuid::new_v4()),
            name: "Integration User".to_string(),
            password: "Password123!".to_string(),
        })
        .await
        .unwrap();

    let authed = AuthenticateUser::new(PgUserRepository::new(pool.clone()))
        .execute(AuthenticateUserInput {
            email: user.email.clone(),
            password: "Password123!".to_string(),
        })
        .await
        .unwrap();
    assert_eq!(authed.id, user.id);

    let booking = CreateBooking::new(PgBookingRepository::new(pool.clone()))
        .execute(CreateBookingInput {
            user_id: user.id,
            flight_id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            base_price: 9500.0,
            payment_method: "eth".to_string(),
        })
        .await
        .unwrap();

    let init = InitPayment::new(
        PgBookingRepository::new(pool.clone()),
        PgPaymentRepository::new(pool.clone()),
        TestGateway,
    )
    .execute(InitPaymentInput {
        booking_id: booking.booking_id,
        method: "eth".to_string(),
    })
    .await
    .unwrap();
    assert!(!init.wallet_address.is_empty());

    ConfirmPayment::new(
        PgBookingRepository::new(pool.clone()),
        PgPaymentRepository::new(pool.clone()),
        PgTransactionRepository::new(pool.clone()),
        TestGateway,
    )
    .execute(ConfirmPaymentInput {
        booking_id: booking.booking_id,
        tx_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
    })
    .await
    .unwrap();

    let txs = GetTransactions::new(PgTransactionRepository::new(pool.clone()))
        .execute(user.id)
        .await
        .unwrap();
    assert!(!txs.is_empty());
}
