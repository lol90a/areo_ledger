# AeroLedger Technology Stack

## Programming Languages

### Backend
- **Rust** (Edition 2021)
  - Systems programming language
  - Memory safety without garbage collection
  - High performance and concurrency

### Frontend
- **TypeScript** (v5.3.3)
  - Type-safe JavaScript
  - Enhanced IDE support
  - Compile-time error checking

### Smart Contracts
- **Solidity** (Ethereum contracts)
- **Rust** (Solana programs via Anchor framework)

## Backend Technologies

### Web Framework
- **actix-web** (v4.12.1) - High-performance async web framework
- **actix-cors** (v0.6) - CORS middleware
- **actix-governor** (v0.5) - Rate limiting middleware

### Database
- **postgres** (v0.19) - PostgreSQL client
- **r2d2** (v0.8) - Connection pooling
- **r2d2_postgres** (v0.18) - PostgreSQL connection pool

### Authentication & Security
- **jsonwebtoken** (v9.2) - JWT token generation/validation
- **bcrypt** (v0.15) - Password hashing
- **uuid** (v1.0) - Unique identifier generation

### Async Runtime
- **tokio** (v1.0) - Async runtime with full features

### Serialization
- **serde** (v1.0) - Serialization framework
- **serde_json** (v1.0) - JSON support
- **serde_urlencoded** (v0.7) - URL encoding

### Utilities
- **chrono** (v0.4) - Date and time handling
- **dotenvy** (v0.15) - Environment variable loading
- **reqwest** (v0.11) - HTTP client
- **lettre** (v0.11) - Email sending
- **log** (v0.4) - Logging facade
- **env_logger** (v0.11) - Logger implementation

## Frontend Technologies

### Framework & Runtime
- **Next.js** (v14.1.0) - React framework with App Router
- **React** (v18.2.0) - UI library
- **React DOM** (v18.2.0) - React renderer

### UI Components
- **@radix-ui/react-avatar** (v1.0.4)
- **@radix-ui/react-dialog** (v1.0.5)
- **@radix-ui/react-dropdown-menu** (v2.0.6)
- **@radix-ui/react-select** (v2.0.0)
- **@radix-ui/react-separator** (v1.0.3)
- **@radix-ui/react-slot** (v1.0.2)
- **@radix-ui/react-tabs** (v1.0.4)

### 3D Graphics
- **three** (v0.161.0) - 3D library
- **@react-three/fiber** (v8.15.16) - React renderer for Three.js
- **@react-three/drei** (v9.96.1) - Three.js helpers

### State Management & Data Fetching
- **@tanstack/react-query** (v5.17.19) - Server state management
- **axios** (v1.6.7) - HTTP client

### Styling & Animation
- **tailwindcss** (v3.4.1) - Utility-first CSS framework
- **tailwindcss-animate** (v1.0.7) - Animation utilities
- **framer-motion** (v11.0.3) - Animation library
- **autoprefixer** (v10.4.17) - CSS vendor prefixing
- **postcss** (v8.4.33) - CSS transformation

### UI Utilities
- **class-variance-authority** (v0.7.0) - Component variants
- **clsx** (v2.1.0) - Conditional classnames
- **tailwind-merge** (v2.2.1) - Tailwind class merging
- **lucide-react** (v0.323.0) - Icon library

### Data Visualization
- **recharts** (v2.10.3) - Chart library

### Utilities
- **uuid** (v13.0.0) - Unique identifier generation

### Development Tools
- **TypeScript** (v5.3.3) - Type checking
- **ESLint** (v8.56.0) - Code linting
- **eslint-config-next** (v14.1.0) - Next.js ESLint config
- **@types/node** (v20.11.5) - Node.js type definitions
- **@types/react** (v18.2.48) - React type definitions
- **@types/react-dom** (v18.2.18) - React DOM type definitions
- **@types/three** (v0.161.2) - Three.js type definitions

## Smart Contract Technologies

### Ethereum
- **Hardhat** - Development environment
- **Solidity** - Smart contract language
- **OpenZeppelin** - Contract libraries

### Solana
- **Anchor** - Solana development framework
- **Rust** - Program language

## Database

### Primary Database
- **PostgreSQL** - Production database
- **SQLite** - Development/testing database

### Features Used
- Connection pooling (r2d2)
- Async queries
- Transaction support
- Migration management

## Development Commands

### Backend (Rust)
```bash
# Development
cargo run                    # Run server (http://127.0.0.1:8080)
cargo build                  # Build project
cargo build --release        # Production build
cargo test                   # Run tests
cargo check                  # Check compilation

# Database
# Migrations run automatically on startup
```

### Frontend (Next.js)
```bash
# Development
npm install                  # Install dependencies
npm run dev                  # Dev server (http://localhost:3001)
npm run build                # Production build
npm start                    # Start production server
npm run lint                 # Run ESLint

# Environment
# Copy .env.local.example to .env.local
```

### Smart Contracts (Ethereum)
```bash
cd contracts/ethereum
npm install                  # Install dependencies
npx hardhat compile          # Compile contracts
npx hardhat run scripts/deploy.js  # Deploy contracts
npx hardhat test             # Run tests
```

### Smart Contracts (Solana)
```bash
cd contracts/solana
anchor build                 # Build programs
anchor deploy                # Deploy programs
anchor test                  # Run tests
```

## Build Systems

### Backend
- **Cargo** - Rust package manager and build tool
- Incremental compilation
- Dependency caching
- Release optimization

### Frontend
- **npm** - Node package manager
- **Next.js** - Build and bundling
- **Turbopack** - Fast bundler (Next.js 14)
- **SWC** - Fast TypeScript/JavaScript compiler

### Smart Contracts
- **Hardhat** - Ethereum development environment
- **Anchor** - Solana framework with built-in build system

## Environment Configuration

### Backend (.env)
```
DATABASE_URL=postgres://user:pass@localhost/aeroledger
JWT_SECRET=your-secret-key
RUST_LOG=info
```

### Frontend (.env.local)
```
NEXT_PUBLIC_API_URL=http://127.0.0.1:8080
```

## API Communication

### Protocol
- **REST** - HTTP/JSON API
- **WebSocket** - Real-time updates (future)

### Authentication
- **JWT** - Bearer token authentication
- Token stored in localStorage
- Included in Authorization header

### CORS Configuration
- Allowed origin: http://localhost:3001
- Allowed methods: GET, POST, PUT, DELETE
- Allowed headers: Content-Type, Authorization

### Rate Limiting
- 100 requests per minute per IP
- Configured via actix-governor

## Performance Features

### Backend
- Async/await throughout
- Connection pooling
- Efficient database queries
- JSON serialization optimization

### Frontend
- Server-side rendering (SSR)
- Static generation where possible
- Code splitting
- Image optimization
- Font optimization

## Security Features

### Backend
- CORS protection
- Rate limiting
- JWT authentication
- Input validation
- SQL injection prevention (parameterized queries)
- Password hashing (bcrypt)

### Frontend
- XSS protection (React escaping)
- CSRF protection (Next.js)
- Secure token storage
- Environment variable protection

## Deployment Targets

### Backend
- AWS EC2 / DigitalOcean
- Docker containers
- Systemd service

### Frontend
- Vercel (recommended)
- Netlify
- AWS Amplify

### Database
- AWS RDS (PostgreSQL)
- DigitalOcean Managed Database
- Self-hosted PostgreSQL

## Version Requirements

- **Rust**: 1.70+ (Edition 2021)
- **Node.js**: 18.0+
- **npm**: 9.0+
- **PostgreSQL**: 13+
- **Solidity**: 0.8+
