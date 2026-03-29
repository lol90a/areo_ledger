# AeroLedger Project Structure

## Project Organization

AeroLedger is a monorepo containing a Rust backend, Next.js frontend, and Ethereum/Solana smart contracts.

```
areo_ledger/
├── src/                      # Rust backend (Actix-web)
├── frontend-luxury/          # Next.js frontend (TypeScript)
├── contracts/                # Smart contracts
│   ├── ethereum/            # Solidity contracts (Hardhat)
│   └── solana/              # Rust contracts (Anchor)
├── migrations/               # Database schemas
└── target/                   # Rust build artifacts
```

## Backend Structure (Rust + Actix-web)

### Core Architecture
```
src/
├── routes/                   # API endpoint handlers
│   ├── admin.rs             # Admin dashboard endpoints
│   ├── assets.rs            # Tokenized asset management
│   ├── bookings.rs          # Flight booking creation
│   ├── calculator.rs        # Currency conversion
│   ├── flights.rs           # Flight search
│   ├── health.rs            # Health check endpoint
│   ├── order_tracking.rs    # Order status tracking
│   ├── orders.rs            # Order management
│   ├── payments.rs          # Crypto payment processing
│   ├── payments_new.rs      # Enhanced payment system
│   ├── portfolio.rs         # User portfolio tracking
│   ├── products.rs          # Product catalog
│   ├── profile.rs           # User profile management
│   ├── transactions.rs      # Transaction history
│   ├── users.rs             # Authentication & user management
│   ├── dto.rs               # Data transfer objects
│   └── mod.rs               # Route configuration
│
├── models/                   # Data structures
│   ├── booking.rs           # Booking entity
│   ├── flight.rs            # Flight entity
│   ├── payment.rs           # Payment entity
│   ├── user.rs              # User entity
│   └── mod.rs               # Model exports
│
├── services/                 # Business logic layer
│   ├── admin_service.rs     # Admin operations
│   ├── asset_service.rs     # Asset management
│   ├── blockchain.rs        # Blockchain integration
│   ├── booking_service.rs   # Booking logic
│   ├── calculator_service.rs # Currency calculations
│   ├── email.rs             # Email notifications
│   ├── flight_service.rs    # Flight operations
│   ├── order_service.rs     # Order processing
│   ├── payment_service.rs   # Payment processing
│   ├── portfolio_service.rs # Portfolio management
│   ├── product_service.rs   # Product operations
│   ├── profit_engine.rs     # Profit calculations
│   ├── transaction_service.rs # Transaction tracking
│   ├── user_service.rs      # User operations
│   └── mod.rs               # Service exports
│
├── middleware/               # Request middleware
│   ├── auth.rs              # JWT authentication
│   └── mod.rs               # Middleware exports
│
├── main.rs                   # Application entry point
├── config.rs                 # Environment configuration
├── db.rs                     # Database connection pool
├── auth.rs                   # Authentication utilities
└── errors.rs                 # Error handling
```

### API Endpoints (19 total)
- **Assets**: GET /api/assets, GET /api/assets/{id}
- **Products**: GET /api/products, GET /api/products/{id}
- **Orders**: POST /api/orders, GET /api/orders/{id}, GET /api/orders/user/{id}
- **Portfolio**: GET /api/portfolio/{id}
- **Transactions**: GET /api/transactions/{id}
- **Calculator**: POST /api/calculator/convert, GET /api/calculator/prices
- **Bookings**: POST /api/bookings
- **Payments**: POST /api/payments/init, POST /api/payments/confirm
- **Users**: POST /api/users/signup, POST /api/users/login
- **Admin**: GET /api/admin/stats, GET /api/admin/bookings, GET /api/admin/payments

## Frontend Structure (Next.js 14 + TypeScript)

### App Router Architecture
```
frontend-luxury/
├── app/                      # Next.js App Router
│   ├── admin/               # Admin dashboard
│   ├── assets/              # Tokenized assets page
│   ├── calculator/          # Currency calculator
│   ├── cart/                # Shopping cart
│   ├── checkout/            # Checkout flow
│   ├── dashboard/           # User dashboard
│   ├── flights/             # Flight search
│   ├── marketplace/         # Product marketplace
│   ├── order-tracking/      # Order status
│   ├── portfolio/           # Asset portfolio
│   ├── profile/             # User profile
│   ├── signin/              # Authentication
│   ├── transactions/        # Transaction history
│   ├── layout.tsx           # Root layout
│   ├── page.tsx             # Homepage
│   └── providers.tsx        # Context providers
│
├── components/               # Reusable UI components
│   ├── calculator/          # Calculator components
│   ├── cards/               # Card components
│   ├── charts/              # Chart visualizations
│   ├── dashboard/           # Dashboard widgets
│   ├── hero/                # Hero sections
│   ├── layout/              # Layout components
│   ├── marketplace/         # Marketplace components
│   ├── navbar/              # Navigation
│   ├── theme/               # Theme components
│   └── ui/                  # Base UI components (Radix)
│
├── context/                  # React Context
│   ├── CartContext.tsx      # Shopping cart state
│   └── ThemeContext.tsx     # Theme state
│
├── lib/                      # Utilities
│   ├── api.ts               # API client (Axios)
│   ├── hooks.ts             # Custom React hooks
│   └── utils.ts             # Helper functions
│
├── styles/
│   └── globals.css          # Global styles
│
├── middleware.ts             # Next.js middleware
├── next.config.js            # Next.js configuration
├── tailwind.config.js        # Tailwind CSS config
└── tsconfig.json             # TypeScript config
```

## Smart Contracts Structure

### Ethereum (Hardhat)
```
contracts/ethereum/
├── contracts/                # Solidity contracts
├── scripts/                  # Deployment scripts
│   └── deploy.js            # Contract deployment
├── artifacts/                # Compiled contracts
├── cache/                    # Build cache
├── hardhat.config.js         # Hardhat configuration
└── package.json              # Dependencies
```

### Solana (Anchor)
```
contracts/solana/
├── programs/                 # Rust programs
└── Anchor.toml               # Anchor configuration
```

## Database Structure

### Migrations
```
migrations/
├── 01_create_users.sql       # User table
├── 02_create_flights.sql     # Flight table
├── 03_create_bookings.sql    # Booking table
├── 04_create_payments.sql    # Payment table
├── 05_create_profit_ledger.sql # Profit tracking
├── 02_order_status_history.sql # Order history
└── complete_schema.sql       # Full schema
```

### Database Tables
- **users**: Authentication and role management
- **bookings**: Flight bookings with status tracking
- **payments**: Multi-chain payment records
- **orders**: Product orders
- **order_items**: Order line items
- **profit_ledger**: Revenue tracking

## Architectural Patterns

### Backend Patterns
- **Layered Architecture**: Routes → Services → Database
- **Repository Pattern**: Database access abstraction
- **Middleware Chain**: CORS → Rate Limiting → Auth → Routes
- **Error Handling**: Centralized error types and responses
- **Connection Pooling**: r2d2 for database connections

### Frontend Patterns
- **App Router**: File-based routing with Next.js 14
- **Context API**: Global state management
- **Component Composition**: Reusable UI components
- **API Client**: Centralized Axios instance
- **Server Components**: Default server-side rendering

### Communication
- **REST API**: JSON over HTTP
- **JWT Authentication**: Bearer token in headers
- **CORS**: Configured for localhost:3001
- **Rate Limiting**: 100 requests/minute

## Key Relationships

### Data Flow
```
Frontend (Next.js) 
    ↓ HTTP/REST
Backend Routes (Actix)
    ↓ Business Logic
Services Layer
    ↓ SQL Queries
Database (PostgreSQL/SQLite)
```

### Authentication Flow
```
User Login → JWT Token → localStorage → Authorization Header → Protected Routes
```

### Payment Flow
```
Init Payment → Wallet Address → User Sends Crypto → Submit TX Hash → Confirm Payment
```

## Configuration Files

### Backend
- **Cargo.toml**: Rust dependencies and project metadata
- **.env**: Database URL, JWT secret, API keys

### Frontend
- **package.json**: Node dependencies and scripts
- **.env.local**: Backend API URL
- **next.config.js**: Next.js configuration
- **tailwind.config.js**: Tailwind CSS customization

### Smart Contracts
- **hardhat.config.js**: Ethereum network configuration
- **Anchor.toml**: Solana program configuration
