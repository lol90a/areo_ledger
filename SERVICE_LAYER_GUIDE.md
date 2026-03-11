# Service Layer Refactoring - Complete Guide

## 🎯 What Was Moved to Services

### BEFORE (Routes had business logic):
```
routes/
├── payments.rs      ❌ Had payment processing logic
├── bookings.rs      ❌ Had booking calculations
├── orders.rs        ❌ Had order creation logic
├── portfolio.rs     ❌ Had portfolio calculations
├── admin.rs         ❌ Had statistics logic
└── transactions.rs  ❌ Had transaction queries
```

### AFTER (Clean separation):
```
routes/                          services/
├── payments.rs    →  HTTP only  ├── payment_service.rs    ✅ Payment logic
├── bookings.rs    →  HTTP only  ├── booking_service.rs    ✅ Booking logic
├── orders.rs      →  HTTP only  ├── order_service.rs      ✅ Order logic
├── portfolio.rs   →  HTTP only  ├── portfolio_service.rs  ✅ Portfolio logic
├── admin.rs       →  HTTP only  ├── admin_service.rs      ✅ Admin logic
└── transactions.rs →  HTTP only  ├── blockchain.rs         ✅ Verification
                                  └── email.rs              ✅ Notifications
```

---

## 📦 Services Created

### 1. PaymentService (`services/payment_service.rs`)

**Responsibilities**:
- Initialize payments with wallet address selection
- Confirm payments with blockchain verification
- Get payment status
- Get booking payments

**Key Methods**:
```rust
pub async fn init_payment(&self, pool: &PgPool, booking_id: Uuid, payment_method: &str) -> Result<PaymentInitResponse, String>
pub async fn confirm_payment(&self, pool: &PgPool, booking_id: Uuid, tx_hash: &str, sender_address: Option<String>) -> Result<PaymentConfirmResponse, String>
pub async fn get_payment_status(&self, pool: &PgPool, payment_id: Uuid) -> Result<serde_json::Value, String>
pub async fn get_booking_payments(&self, pool: &PgPool, booking_id: Uuid) -> Result<Vec<serde_json::Value>, String>
```

**Features**:
- ✅ Blockchain transaction verification
- ✅ Email notifications on confirmation
- ✅ Multi-chain support (ETH, BTC, SOL, BSC)
- ✅ Environment-based wallet addresses

---

### 2. BookingService (`services/booking_service.rs`)

**Responsibilities**:
- Create bookings with automatic markup calculation
- Get booking details
- Get user bookings
- Update booking status
- Cancel bookings

**Key Methods**:
```rust
pub async fn create_booking(&self, pool: &PgPool, user_id: Uuid, flight_id: &str, base_price: f64, payment_method: &str) -> Result<BookingResponse, String>
pub async fn get_booking(&self, pool: &PgPool, booking_id: Uuid) -> Result<serde_json::Value, String>
pub async fn get_user_bookings(&self, pool: &PgPool, user_id: Uuid) -> Result<Vec<serde_json::Value>, String>
pub async fn update_booking_status(&self, pool: &PgPool, booking_id: Uuid, status: &str) -> Result<(), String>
pub async fn cancel_booking(&self, pool: &PgPool, booking_id: Uuid, user_id: Uuid) -> Result<(), String>
```

**Features**:
- ✅ Automatic 15% markup (10% profit + 5% service fee)
- ✅ Email confirmation on booking
- ✅ Authorization checks for cancellation

---

### 3. OrderService (`services/order_service.rs`)

**Responsibilities**:
- Create orders with items
- Calculate service fees
- Convert to crypto amounts
- Update orders with transaction details
- Get order details

**Key Methods**:
```rust
pub async fn create_order(&self, pool: &PgPool, user_id: Uuid, items: Vec<OrderItem>, total: f64, crypto_method: &str, sender_wallet: &str, tx_hash: Option<String>) -> Result<OrderResponse, String>
pub async fn update_order(&self, pool: &PgPool, order_id: Uuid, sender_wallet: &str, tx_hash: &str) -> Result<(), String>
pub async fn get_order(&self, pool: &PgPool, order_id: Uuid) -> Result<serde_json::Value, String>
pub async fn get_user_orders(&self, pool: &PgPool, user_id: Uuid) -> Result<Vec<serde_json::Value>, String>
```

**Features**:
- ✅ 5% service fee calculation
- ✅ Crypto amount conversion
- ✅ Order items management
- ✅ Email confirmation

---

### 4. PortfolioService (`services/portfolio_service.rs`)

**Responsibilities**:
- Calculate portfolio value
- Track profit/loss
- Manage assets
- Update asset values

**Key Methods**:
```rust
pub async fn get_user_portfolio(&self, pool: &PgPool, user_id: Uuid) -> Result<PortfolioResponse, String>
pub async fn add_asset(&self, pool: &PgPool, user_id: Uuid, asset_name: &str, asset_type: &str, quantity: i32, purchase_price: f64) -> Result<Uuid, String>
pub async fn update_asset_value(&self, pool: &PgPool, asset_id: Uuid, current_value: f64) -> Result<(), String>
```

**Features**:
- ✅ Total purchase value calculation
- ✅ Current value tracking
- ✅ Profit/loss percentage
- ✅ Asset categorization

---

### 5. AdminService (`services/admin_service.rs`)

**Responsibilities**:
- Dashboard statistics
- Revenue reporting
- User analytics
- Recent activity tracking

**Key Methods**:
```rust
pub async fn get_stats(&self, pool: &PgPool) -> Result<AdminStats, String>
pub async fn get_recent_bookings(&self, pool: &PgPool, limit: i64) -> Result<Vec<BookingRecord>, String>
pub async fn get_recent_payments(&self, pool: &PgPool, limit: i64) -> Result<Vec<PaymentRecord>, String>
pub async fn get_revenue_by_period(&self, pool: &PgPool, period: &str) -> Result<Vec<serde_json::Value>, String>
pub async fn get_top_users(&self, pool: &PgPool, limit: i64) -> Result<Vec<serde_json::Value>, String>
```

**Features**:
- ✅ Real-time statistics
- ✅ Revenue analytics by period
- ✅ Top users by spending
- ✅ Payment status tracking

---

## 🔄 How to Use Services in Routes

### Example: Refactored Payments Route

**OLD (routes/payments.rs)**:
```rust
async fn init(pool: web::Data<PgPool>, payload: web::Json<InitPaymentRequest>) -> Result<web::Json<serde_json::Value>, AppError> {
    // ❌ Business logic in route
    let booking = sqlx::query!("SELECT total_price FROM bookings WHERE id = $1", payload.booking_id).fetch_one(&**pool).await?;
    let (chain, token) = match payload.method.as_str() { ... };
    let receiver_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb";
    sqlx::query!("INSERT INTO payments ...").execute(&**pool).await?;
    Ok(web::Json(serde_json::json!({ ... })))
}
```

**NEW (routes/payments_new.rs)**:
```rust
async fn init_payment(pool: web::Data<PgPool>, payload: web::Json<InitPaymentRequest>) -> impl Responder {
    let payment_service = PaymentService::new();
    
    // ✅ Delegate to service
    match payment_service.init_payment(&pool, payload.booking_id, &payload.payment_method).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({"error": e})),
    }
}
```

---

## ✅ Benefits of Service Layer

### 1. Separation of Concerns
- **Routes**: Handle HTTP (request/response)
- **Services**: Handle business logic
- **Database**: Handle data persistence

### 2. Testability
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_booking() {
        let service = BookingService::new();
        let result = service.create_booking(&pool, user_id, "FL123", 1000.0, "BTC").await;
        assert!(result.is_ok());
    }
}
```

### 3. Reusability
```rust
// Use same service in multiple routes
let payment_service = PaymentService::new();

// In payments route
payment_service.init_payment(...).await;

// In admin route
payment_service.get_payment_status(...).await;

// In webhook handler
payment_service.confirm_payment(...).await;
```

### 4. Maintainability
- Change business logic in ONE place
- Routes stay thin and focused
- Easy to add new features

---

## 🚀 Migration Steps

### Step 1: Update Routes to Use Services

Replace old routes with new service-based routes:

```bash
# Backup old routes
mv src/routes/payments.rs src/routes/payments_old.rs
mv src/routes/payments_new.rs src/routes/payments.rs

# Do same for bookings, orders, etc.
```

### Step 2: Update Route Imports

In each route file:
```rust
use crate::services::{PaymentService, BookingService, OrderService};
```

### Step 3: Test Each Service

```bash
cargo test --lib services
```

---

## 📊 What Stays in Routes vs Services

### ✅ ROUTES (HTTP Layer)
- Request validation
- Response formatting
- HTTP status codes
- Error handling (HTTP errors)
- Authentication checks
- CORS handling

### ✅ SERVICES (Business Logic)
- Database queries
- Calculations (markup, fees, totals)
- Business rules
- External API calls (blockchain, email)
- Data transformations
- Validation (business rules)

---

## 🔧 Next Steps

1. **Replace Old Routes**:
   ```bash
   # Remove old route files
   rm src/routes/payments_old.rs
   rm src/routes/bookings_old.rs
   ```

2. **Update mod.rs**:
   ```rust
   // Ensure all services are exported
   pub use services::*;
   ```

3. **Add Tests**:
   ```bash
   # Create test files
   touch src/services/tests/payment_service_test.rs
   touch src/services/tests/booking_service_test.rs
   ```

4. **Update Documentation**:
   - Document each service method
   - Add usage examples
   - Create API documentation

---

## 📝 Summary

| Component | Before | After |
|-----------|--------|-------|
| **Payments** | Logic in routes | ✅ PaymentService |
| **Bookings** | Logic in routes | ✅ BookingService |
| **Orders** | Logic in routes | ✅ OrderService |
| **Portfolio** | Logic in routes | ✅ PortfolioService |
| **Admin** | Logic in routes | ✅ AdminService |
| **Blockchain** | N/A | ✅ BlockchainVerifier |
| **Email** | N/A | ✅ EmailService |

**Result**: Clean, maintainable, testable code with proper separation of concerns! 🎉
