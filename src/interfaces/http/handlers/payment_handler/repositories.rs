use actix_web::web;

use crate::infrastructure::db::DbPool;
use crate::infrastructure::persistence::postgres::pg_audit_log_repository::PgAuditLogRepository;
use crate::infrastructure::persistence::postgres::pg_booking_repository::PgBookingRepository;
use crate::infrastructure::persistence::postgres::pg_catalog_repository::PgCatalogRepository;
use crate::infrastructure::persistence::postgres::pg_payment_repository::PgPaymentRepository;
use crate::infrastructure::persistence::postgres::pg_quote_repository::PgQuoteRepository;
use crate::infrastructure::persistence::postgres::pg_settlement_repository::PgSettlementRepository;

/// Builds the Postgres repositories the payment endpoints depend on.
///
/// Before the split, every handler repeated `PgXRepository::new(pool.get_ref()
/// .clone())` inline -- nineteen times across six endpoints, each one importing
/// the concrete type. Separating the handlers into modules would have carried
/// those imports into all of them; collecting the construction here means each
/// handler module imports this one type and asks for what it needs by name.
///
/// Every accessor returns a fresh value. A repository is a handle over the
/// shared pool, so that costs an `Arc` clone, not a connection -- the same thing
/// the inline construction was doing.
pub(super) struct Repositories {
    pool: DbPool,
}

impl Repositories {
    pub(super) fn new(pool: &web::Data<DbPool>) -> Self {
        Self {
            pool: pool.get_ref().clone(),
        }
    }

    pub(super) fn bookings(&self) -> PgBookingRepository {
        PgBookingRepository::new(self.pool.clone())
    }

    pub(super) fn quotes(&self) -> PgQuoteRepository {
        PgQuoteRepository::new(self.pool.clone())
    }

    pub(super) fn payments(&self) -> PgPaymentRepository {
        PgPaymentRepository::new(self.pool.clone())
    }

    pub(super) fn catalog(&self) -> PgCatalogRepository {
        PgCatalogRepository::new(self.pool.clone())
    }

    pub(super) fn settlements(&self) -> PgSettlementRepository {
        PgSettlementRepository::new(self.pool.clone())
    }

    pub(super) fn audit_log(&self) -> PgAuditLogRepository {
        PgAuditLogRepository::new(self.pool.clone())
    }
}
