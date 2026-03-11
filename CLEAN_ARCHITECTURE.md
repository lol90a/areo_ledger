# AeroLedger - Complete Clean Architecture

## ✅ All Routes Now Have Services

Every route in the `routes/` folder now has a corresponding service in the `services/` folder.

### Complete Service Layer

| Route | Service | Description |
|-------|---------|-------------|
| `users.rs` | `UserService` | User registration, authentication, management |
| `bookings.rs` | `BookingService` | Flight booking creation and management |
| `flights.rs` | `FlightService` | Flight search and filtering |
| `payments.rs` | `PaymentService` | Payment initialization and confirmation |
| `admin.rs` | `AdminService` | Dashboard statistics aggregation |
| `assets.rs` | `AssetService` | Tokenized asset management |
| `calculator.rs` | `CalculatorService` | Currency conversion (crypto & fiat) |
| `products.rs` | `ProductService` | Marketplace products (jets, yachts, etc.) |
| `orders.rs` | `OrderService` | Order creation and tracking |
| `portfolio.rs` | `PortfolioService` | User portfolio management |
| `transactions.rs` | `TransactionService` | Transaction history |
| `health.rs` | *(No service needed)* | Simple health check |

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    HTTP Requests                         │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                  Routes (Controllers)                    │
│  - Handle HTTP logic                                     │
│  - Validate input                                        │
│  - Call services                                         │
│  - Return responses                                      │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                Services (Business Logic)                 │
│  - Core business rules                                   │
│  - Data processing                                       │
│  - Database operations                                   │
│  - Calculations                                          │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│              Database (PostgreSQL + r2d2)                │
│  - Connection pooling                                    │
│  - Data persistence                                      │
└─────────────────────────────────────────────────────────┘
```

## Service Details

### 1. UserService
**Location**: `src/services/user_service.rs`

**Methods**:
- `create_user()` - Register new user with bcrypt password hashing
- `authenticate_user()` - Login with password verification
- `get_user_by_id()` - Fetch user details
- `count_users()` - Count total users (admin stats)

**Features**:
- Auto-detect admin role (email contains "admin")
- Secure password hashing with bcrypt
- UUID-based user IDs

---

### 2. BookingService
**Location**: `src/services/booking_service.rs`

**Methods**:
- `create_booking()` - Create flight booking with automatic markup
- `get_all_bookings()` - Fetch all bookings (admin)
- `count_bookings()` - Count total bookings
- `get_total_revenue()` - Calculate confirmed revenue

**Features**:
- 15% automatic markup (10% profit + 5% service fee)
- Status tracking (pending/confirmed)
- Payment method validation

---

### 3. FlightService
**Location**: `src/services/flight_service.rs`

**Methods**:
- `get_all_flights()` - Returns 6 luxury flights
- `search_flights()` - Filter by origin, destination, passengers

**Features**:
- 6 hardcoded luxury flights (Gulfstream, Bombardier, etc.)
- Smart filtering with partial matching
- Passenger capacity filtering

---

### 4. PaymentService
**Location**: `src/services/payment_service.rs`

**Methods**:
- `init_payment()` - Initialize payment with wallet address
- `confirm_payment()` - Confirm with transaction hash
- `get_all_payments()` - Fetch all payments (admin)
- `get_chain_and_token()` - Map payment method to blockchain

**Features**:
- Multi-chain support (BTC, ETH, SOL, BSC)
- Transaction hash storage
- Status management (pending/confirmed)

---

### 5. AdminService
**Location**: `src/services/admin_service.rs`

**Methods**:
- `get_stats()` - Aggregate dashboard statistics

**Features**:
- Total users count
- Total bookings count
- Total revenue calculation
- Active flights count

---

### 6. AssetService
**Location**: `src/services/asset_service.rs`

**Methods**:
- `get_all_assets()` - Returns 6 tokenized assets
- `get_asset_by_id()` - Get specific asset details

**Features**:
- Tokenized luxury assets (jets, yachts, real estate, cars)
- ROI tracking
- Token availability
- Detailed specifications

---

### 7. CalculatorService
**Location**: `src/services/calculator_service.rs`

**Methods**:
- `convert()` - Convert between currencies
- `get_crypto_prices()` - Get 15 crypto prices
- `get_fiat_rates()` - Get 15 fiat exchange rates

**Features**:
- 15 cryptocurrencies (BTC, ETH, USDT, etc.)
- 15 fiat currencies (USD, EUR, GBP, etc.)
- Real-time conversion calculations

---

### 8. ProductService
**Location**: `src/services/product_service.rs`

**Methods**:
- `get_all_products()` - Returns 12 luxury products
- `get_product_by_id()` - Get specific product

**Features**:
- 12 luxury items (3 jets, 3 yachts, 3 real estate, 3 cars)
- Detailed specifications
- Feature lists
- High-quality images

---

### 9. OrderService
**Location**: `src/services/order_service.rs`

**Methods**:
- `create_order()` - Create marketplace order
- `update_order()` - Update with transaction details
- `get_order_by_id()` - Fetch order details
- `get_user_orders()` - Get user's order history
- `get_crypto_prices()` - Calculate crypto amounts

**Features**:
- 5% service fee calculation
- Crypto amount calculation
- Order status tracking
- Multiple items per order

---

### 10. PortfolioService
**Location**: `src/services/portfolio_service.rs`

**Methods**:
- `get_user_portfolio()` - Get user's asset portfolio

**Features**:
- Total purchase value
- Current value tracking
- Asset quantity
- Portfolio statistics

---

### 11. TransactionService
**Location**: `src/services/transaction_service.rs`

**Methods**:
- `get_user_transactions()` - Get transaction history

**Features**:
- Transaction type tracking (buy/sell)
- Amount and token tracking
- Status monitoring
- Transaction hash display

---

## Database Schema (PostgreSQL)

### Tables Created

1. **users** - User accounts
2. **bookings** - Flight bookings
3. **payments** - Payment records
4. **orders** - Marketplace orders
5. **order_items** - Order line items

All tables use:
- UUID for primary keys
- DECIMAL for monetary values
- TIMESTAMP for dates
- Proper foreign key relationships

## API Endpoints

### Authentication
- `POST /api/users/signup`
- `POST /api/users/login`

### Flights
- `GET /api/flights?from=&to=&passengers=`

### Bookings
- `POST /api/bookings`

### Payments
- `POST /api/payments/init`
- `POST /api/payments/confirm`

### Admin
- `GET /api/admin/stats`
- `GET /api/admin/bookings`
- `GET /api/admin/payments`

### Assets
- `GET /api/assets`
- `GET /api/assets/{id}`

### Products
- `GET /api/products`
- `GET /api/products/{id}`

### Orders
- `POST /api/orders`
- `GET /api/orders/{id}`
- `PUT /api/orders/{id}`
- `GET /api/orders/user/{user_id}`

### Portfolio
- `GET /api/portfolio/{user_id}`

### Transactions
- `GET /api/transactions/{user_id}`

### Calculator
- `POST /api/calculator/convert`
- `GET /api/calculator/prices`

### Health
- `GET /api/health`

## Benefits of This Architecture

### 1. Separation of Concerns
- **Routes**: HTTP handling only
- **Services**: Business logic only
- **Database**: Data persistence only

### 2. Testability
- Services can be unit tested independently
- Mock database connections for testing
- Test business logic without HTTP layer

### 3. Maintainability
- Clear structure, easy to navigate
- Changes isolated to specific layers
- Easy to add new features

### 4. Reusability
- Services can be called from multiple routes
- Shared business logic in one place
- No code duplication

### 5. Type Safety
- PostgreSQL with rust-postgres
- Compile-time type checking
- No runtime type errors

### 6. Performance
- r2d2 connection pooling
- Efficient database access
- Minimal overhead

## Running the Application

1. **Install PostgreSQL**
   ```bash
   # Windows: Download from postgresql.org
   # Mac: brew install postgresql
   # Linux: sudo apt install postgresql
   ```

2. **Create Database**
   ```bash
   createdb aeroledger
   ```

3. **Configure Environment**
   Create `.env`:
   ```
   DATABASE_URL=postgresql://username:password@localhost/aeroledger
   JWT_SECRET=your-secret-key-here
   WALLET_ADDRESS=0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb
   ```

4. **Build & Run**
   ```bash
   cargo build --release
   cargo run --release
   ```

5. **Server starts on**
   ```
   http://127.0.0.1:8080
   ```

## Next Steps

1. ✅ All routes have services
2. ✅ Clean architecture implemented
3. ✅ PostgreSQL with r2d2
4. ✅ Proper error handling
5. ⏳ Add authentication middleware
6. ⏳ Add input validation
7. ⏳ Write unit tests
8. ⏳ Add database migrations
9. ⏳ Add logging
10. ⏳ Add caching (Redis)

## File Structure

```
src/
├── main.rs                      # Application entry
├── db.rs                        # PostgreSQL pool
├── config.rs                    # Environment config
├── errors.rs                    # Error handling
├── auth.rs                      # JWT authentication
├── models/                      # Data structures
│   ├── user.rs
│   ├── booking.rs
│   ├── payment.rs
│   └── mod.rs
├── services/                    # ✅ Business Logic
│   ├── user_service.rs
│   ├── booking_service.rs
│   ├── flight_service.rs
│   ├── payment_service.rs
│   ├── admin_service.rs
│   ├── asset_service.rs
│   ├── calculator_service.rs
│   ├── product_service.rs
│   ├── order_service.rs
│   ├── portfolio_service.rs
│   ├── transaction_service.rs
│   └── mod.rs
└── routes/                      # ✅ HTTP Handlers
    ├── users.rs
    ├── bookings.rs
    ├── flights.rs
    ├── payments.rs
    ├── admin.rs
    ├── assets.rs
    ├── calculator.rs
    ├── products.rs
    ├── orders.rs
    ├── portfolio.rs
    ├── transactions.rs
    ├── health.rs
    └── mod.rs
```

---

**🎉 Clean Architecture Complete!**

Every route now has a corresponding service. All business logic is in the service layer. Routes are thin controllers that only handle HTTP.
