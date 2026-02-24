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

## Environment Setup

Create `.env` file:
```
DATABASE_URL=sqlite:test.db
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

### Should You Add Smart Contracts?

**YES, if you want:**
1. **Trustless Payments** - Automatic escrow and release
2. **On-chain Verification** - Verify payments programmatically
3. **Refund Logic** - Automated refunds for cancellations
4. **Transparency** - All transactions on blockchain
5. **DeFi Integration** - Staking, yield, governance

**NO, if you prefer:**
1. **Simplicity** - Current system works fine
2. **Lower Costs** - No gas fees for users
3. **Flexibility** - Easy to change payment logic
4. **Speed** - Instant confirmation without waiting for blocks

### Recommended Approach
For a production system handling real payments, **implement smart contracts** for:
- **Payment Escrow**: Hold funds until flight/jet delivery
- **Refund Mechanism**: Automatic refunds for cancellations
- **Multi-sig Wallet**: Require multiple approvals for large transactions
- **Oracle Integration**: Verify flight status from external APIs

### Smart Contract Stack
If implementing:
- **Solidity** for Ethereum/BSC
- **Anchor** for Solana
- **Hardhat/Foundry** for development
- **Chainlink** for oracles

## Security Notes

### Current Limitations
1. **No password hashing** - Passwords not stored (demo only)
2. **Hardcoded wallet** - Replace with environment variable
3. **No tx verification** - Trust user-provided hash
4. **No rate limiting** - Add in production

### Production Checklist
- [ ] Implement proper password hashing (bcrypt/argon2)
- [ ] Add JWT authentication
- [ ] Verify transaction hashes on-chain
- [ ] Use environment variables for wallet addresses
- [ ] Add rate limiting
- [ ] Implement HTTPS
- [ ] Add input sanitization
- [ ] Set up monitoring/logging

## File Structure Notes

### Keep
- `src/` - All backend code
- `frontend/src/` - All frontend code
- `migrations/` - Database schema
- `Cargo.toml` - Rust dependencies
- `BACKEND_API.md` - API documentation

### Remove Before Deployment
- `target/` - Build artifacts (add to .gitignore)
- `test.db` - Development database
- `node_modules/` - Frontend dependencies (add to .gitignore)

## Deployment Recommendations

### Backend
- Use PostgreSQL instead of SQLite
- Deploy on AWS/DigitalOcean/Heroku
- Set up CI/CD pipeline
- Configure environment variables

### Frontend
- Build: `npm run build`
- Deploy on Vercel/Netlify
- Configure API endpoint

### Database
- Migrate to PostgreSQL for production
- Set up backups
- Use connection pooling

## Future Enhancements

1. **Web3 Integration** - MetaMask, WalletConnect
2. **Smart Contracts** - Escrow, refunds
3. **Real-time Verification** - Check tx on blockchain
4. **Email Notifications** - Booking confirmations
5. **KYC/AML** - Compliance for large transactions
6. **Multi-currency** - More crypto options
7. **NFT Tickets** - Blockchain-based boarding passes

## Support

For questions or issues, contact the development team.

---

**Note**: This is a demonstration project. For production use, implement proper security measures, smart contracts, and compliance features.
