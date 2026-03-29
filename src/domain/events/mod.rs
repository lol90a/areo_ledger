use uuid::Uuid;
use chrono::{DateTime, Utc};

pub trait DomainEvent: Send + Sync {
    fn event_id(&self)     -> Uuid;
    fn occurred_at(&self)  -> DateTime<Utc>;
    fn aggregate_id(&self) -> Uuid;
    fn event_type(&self)   -> &'static str;
}

#[derive(Debug, Clone)]
pub struct UserRegistered {
    pub event_id:    Uuid,
    pub occurred_at: DateTime<Utc>,
    pub user_id:     Uuid,
    pub email:       String,
    pub role:        String,
}

impl DomainEvent for UserRegistered {
    fn event_id(&self)     -> Uuid          { self.event_id }
    fn occurred_at(&self)  -> DateTime<Utc> { self.occurred_at }
    fn aggregate_id(&self) -> Uuid          { self.user_id }
    fn event_type(&self)   -> &'static str  { "user.registered" }
}

#[derive(Debug, Clone)]
pub struct BookingCreated {
    pub event_id:       Uuid,
    pub occurred_at:    DateTime<Utc>,
    pub booking_id:     Uuid,
    pub user_id:        Uuid,
    pub flight_id:      Uuid,
    pub total_cents:    i64,
    pub payment_method: String,
}

impl DomainEvent for BookingCreated {
    fn event_id(&self)     -> Uuid          { self.event_id }
    fn occurred_at(&self)  -> DateTime<Utc> { self.occurred_at }
    fn aggregate_id(&self) -> Uuid          { self.booking_id }
    fn event_type(&self)   -> &'static str  { "booking.created" }
}

#[derive(Debug, Clone)]
pub struct PaymentConfirmed {
    pub event_id:    Uuid,
    pub occurred_at: DateTime<Utc>,
    pub payment_id:  Uuid,
    pub booking_id:  Uuid,
    pub tx_hash:     String,
}

impl DomainEvent for PaymentConfirmed {
    fn event_id(&self)     -> Uuid          { self.event_id }
    fn occurred_at(&self)  -> DateTime<Utc> { self.occurred_at }
    fn aggregate_id(&self) -> Uuid          { self.payment_id }
    fn event_type(&self)   -> &'static str  { "payment.confirmed" }
}
