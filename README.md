# AeroLedger - Crypto Aviation Platform

## Project Overview
AeroLedger is a luxury flight booking and private jet sales platform that accepts cryptocurrency payments. Built with Rust (backend) and React (frontend), it provides a seamless experience for booking flights and purchasing aircraft using Bitcoin, Ethereum, USDT, USDC, Solana, and BNB.

## Architecture

### Backend (Rust + Actix-web)
```
src/
├── routes/          # API endpoints
│   ├── bookings.rs  # Booking creation with markup
│   ├── payments.rs  # Crypto payment processing
│   ├── users.rs     # Authentication
│   ├── admin.rs     # Admin dashboard
│   └── flights.rs   # Flight search
├── models/          # Data structures
├── services/        # Business logic (unused - direct DB access)
├── config.rs        # Environment configuration
├── db.rs           # Database connection
└── errors.rs       # Error handling
```

### Frontend (React + TypeScript)
```
src/
├── pages/          # Main views
├── components/     # Reusable UI components
├── context/        # State management (Auth, Theme)
└── services/       # API client
```

## Key Features

### 1. Crypto Payment System
- **6 Cryptocurrencies**: BTC, ETH, USDT, USDC, SOL, BNB
- **Multi-chain**: Ethereum, Bitcoin, Solana, BSC
- **Payment Flow**: Init → Send → Confirm with tx hash
- **Automatic Pricing**: 15% markup (10% profit + 5% service fee)

### 2. Dual Business Model
- **Flight Booking**: Commercial and private flights
- **Private Jet Sales**: 6 luxury aircraft ($9.5M - $90M)

### 3. Authentication
- User signup/login with auto-registration
- Admin role detection (email contains "admin")
- Protected routes

### 4. Admin Dashboard
- Real-time statistics
- Booking management
- Payment tracking

## Database Schema

### users
- Role-based access (user/admin)
- Auto-assigned based on email

### bookings
- Status tracking (pending/confirmed)
- Automatic markup calculation
- Payment method selection

### payments
- Multi-chain support
- Transaction hash storage
- Status management

## API Endpoints

### Authentication
- `POST /api/users/signup` - Register
- `POST /api/users/login` - Login (auto-creates if not exists)

### Bookings
- `POST /api/bookings` - Create booking with 15% markup

### Payments
- `POST /api/payments/init` - Get wallet address & amount
- `POST /api/payments/confirm` - Submit transaction hash

### Admin
- `GET /api/admin/stats` - Dashboard statistics
- `GET /api/admin/bookings` - Recent bookings
- `GET /api/admin/payments` - Payment history

## Running the Project

### Backend
```bash
cargo run
# Server: http://127.0.0.1:8080
```

### Frontend
```bash
cd frontend
npm install
npm start
# App: http://localhost:3000
```

## Technology Stack

### Backend
- **Rust** - Systems programming language
- **Actix-web** - Web framework
- **SQLx** - Database toolkit
- **SQLite** - Database

### Frontend
- **React 19** - UI library
- **TypeScript** - Type safety
- **Framer Motion** - Animations
- **Tailwind CSS** - Styling

## Smart Contract Consideration

### Current Implementation
This project currently uses **off-chain payment processing**:
- Users send crypto to a specified wallet address
- Transaction hash is stored in database
- No on-chain verification


---

**Note**: This is a demonstration project. For production use, implement proper security measures, smart contracts, and compliance features.
