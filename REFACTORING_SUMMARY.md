# AeroLedger Refactoring Summary

## Changes Made

### 1. Database Layer (Removed SQLx, Added PostgreSQL with r2d2)

**Cargo.toml**
- Removed: `sqlx`
- Added: `postgres`, `r2d2`, `r2d2_postgres`

**db.rs**
- Replaced SQLx async pool with r2d2 synchronous connection pool
- Using `postgres` crate with NoTLS
- Connection pool size: 15
- All tables created with proper PostgreSQL types (UUID, DECIMAL, TIMESTAMP)

### 2. Clean Architecture - Service Layer

All business logic moved to services:

**services/user_service.rs**
- `create_user()` - User registration with bcrypt password hashing
- `authenticate_user()` - Login with password verification
- `get_user_by_id()` - Fetch user details
- `count_users()` - Admin statistics

**services/booking_service.rs**
- `create_booking()` - Create booking with 15% markup (10% profit + 5% service fee)
- `get_all_bookings()` - Fetch all bookings for admin
- `count_bookings()` - Count total bookings
- `get_total_revenue()` - Calculate total confirmed revenue

**services/flight_service.rs**
- `get_all_flights()` - Returns 6 hardcoded luxury flights
- `search_flights()` - Filter flights by origin, destination, and passenger count

**services/payment_service.rs**
- `init_payment()` - Initialize payment with wallet address
- `confirm_payment()` - Confirm payment with transaction hash
- `get_all_payments()` - Fetch all payments for admin
- Helper: `get_chain_and_token()` - Map payment method to blockchain

**services/admin_service.rs**
- `get_stats()` - Aggregate statistics (users, bookings, revenue, flights)

### 3. Clean Routes (Controllers)

All routes now only handle HTTP logic and delegate to services:

**routes/users.rs**
- `POST /api/users/signup` - Calls UserService::create_user()
- `POST /api/users/login` - Calls UserService::authenticate_user()
- Returns JWT token with user data

**routes/bookings.rs**
- `POST /api/bookings` - Calls BookingService::create_booking()
- Validates payment method and base price
- Returns booking_id and total_price

**routes/flights.rs**
- `GET /api/flights` - Calls FlightService::search_flights()
- Query params: from, to, date, passengers

**routes/payments.rs**
- `POST /api/payments/init` - Calls PaymentService::init_payment()
- `POST /api/payments/confirm` - Calls PaymentService::confirm_payment()

**routes/admin.rs**
- `GET /api/admin/stats` - Calls AdminService::get_stats()
- `GET /api/admin/bookings` - Calls BookingService::get_all_bookings()
- `GET /api/admin/payments` - Calls PaymentService::get_all_payments()

**routes/products.rs**
- Already clean with hardcoded 12 luxury products
- No database needed for products (static data)

### 4. Main.rs

- Clean and minimal
- Initializes database pool
- Creates tables on startup
- Configures CORS for frontend
- Rate limiting with Governor
- Logging middleware

## Project Structure

```
src/
├── main.rs                 # Application entry point
├── db.rs                   # PostgreSQL connection pool
├── config.rs               # Environment configuration
├── errors.rs               # Error handling
├── auth.rs                 # JWT authentication
├── models/                 # Data structures
│   ├── user.rs
│   ├── booking.rs
│   ├── payment.rs
│   └── mod.rs
├── services/               # Business logic layer
│   ├── user_service.rs
│   ├── booking_service.rs
│   ├── flight_service.rs
│   ├── payment_service.rs
│   ├── admin_service.rs
│   └── mod.rs
└── routes/                 # HTTP handlers
    ├── users.rs
    ├── bookings.rs
    ├── flights.rs
    ├── payments.rs
    ├── admin.rs
    ├── products.rs
    └── mod.rs
```

## Database Schema (PostgreSQL)

### users
- id: UUID PRIMARY KEY
- email: VARCHAR(255) UNIQUE
- name: VARCHAR(255)
- role: VARCHAR(50) DEFAULT 'user'
- password_hash: VARCHAR(255)
- wallet_address: VARCHAR(255)
- created_at: TIMESTAMP

### bookings
- id: UUID PRIMARY KEY
- user_id: UUID
- flight_id: UUID
- status: VARCHAR(50)
- base_price: DECIMAL(15, 2)
- markup: DECIMAL(15, 2)
- service_fee: DECIMAL(15, 2)
- total_price: DECIMAL(15, 2)
- payment_method: VARCHAR(50)
- payment_address: VARCHAR(255)
- tx_hash: VARCHAR(255)
- created_at: TIMESTAMP

### payments
- id: UUID PRIMARY KEY
- booking_id: UUID
- chain: VARCHAR(50)
- token: VARCHAR(50)
- amount: DECIMAL(15, 8)
- sender_address: VARCHAR(255)
- receiver_address: VARCHAR(255)
- tx_hash: VARCHAR(255)
- status: VARCHAR(50)
- created_at: TIMESTAMP

### orders
- id: UUID PRIMARY KEY
- user_id: UUID
- total: DECIMAL(15, 2)
- service_fee: DECIMAL(15, 2)
- grand_total: DECIMAL(15, 2)
- crypto_method: VARCHAR(50)
- sender_wallet: VARCHAR(255)
- tx_hash: VARCHAR(255)
- wallet_address: VARCHAR(255)
- crypto_amount: DECIMAL(15, 8)
- status: VARCHAR(50)
- created_at: TIMESTAMP

### order_items
- id: SERIAL PRIMARY KEY
- order_id: UUID
- product_id: INTEGER
- name: VARCHAR(255)
- price: DECIMAL(15, 2)
- quantity: INTEGER

## Environment Variables

Create `.env` file:
```
DATABASE_URL=postgresql://username:password@localhost/aeroledger
JWT_SECRET=your-secret-key-here
WALLET_ADDRESS=0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb
```

## Running the Application

1. Install PostgreSQL
2. Create database: `createdb aeroledger`
3. Update `.env` with your database credentials
4. Build: `cargo build --release`
5. Run: `cargo run --release`

## Benefits of This Architecture

1. **Separation of Concerns**: Routes handle HTTP, Services handle business logic
2. **Testability**: Services can be unit tested independently
3. **Maintainability**: Clear structure, easy to find and modify code
4. **Scalability**: Easy to add new features without touching existing code
5. **Type Safety**: PostgreSQL with rust-postgres provides compile-time guarantees
6. **Performance**: r2d2 connection pooling for efficient database access

## Next Steps

1. Add proper error handling and logging
2. Implement authentication middleware
3. Add input validation with validator crate
4. Write unit tests for services
5. Add integration tests for routes
6. Implement proper transaction handling
7. Add database migrations with refinery or diesel
8. Consider adding caching layer (Redis)
