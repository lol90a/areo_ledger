# AeroLedger Development Guidelines

## Code Quality Standards

### Rust Backend Standards

#### Module Organization
- Use explicit module declarations at the top of main.rs
- Group related functionality into modules (routes, services, models, middleware)
- Export public interfaces through mod.rs files
- Keep route handlers thin - delegate to service layer

#### Naming Conventions
- **Files**: snake_case (e.g., `payment_service.rs`, `user_service.rs`)
- **Structs**: PascalCase (e.g., `PaymentService`, `AuthResponse`)
- **Functions**: snake_case (e.g., `init_payment`, `confirm_payment`)
- **Constants**: SCREAMING_SNAKE_CASE
- **Request/Response DTOs**: Suffix with `Request` or `Response` (e.g., `LoginRequest`, `AuthResponse`)

#### Error Handling
- Use custom `AppError` type for all error handling
- Convert database errors to `AppError::DatabaseError`
- Use `AppError::NotFound` for missing resources
- Use `AppError::ValidationError` for input validation failures
- Return `Result<HttpResponse, AppError>` from route handlers

#### Database Access
- Use connection pooling (r2d2) for all database operations
- Get connection with `pool.get().map_err(|e| AppError::DatabaseError(e.to_string()))?`
- Use parameterized queries to prevent SQL injection
- Handle database errors explicitly with proper error messages

#### Service Layer Pattern
```rust
pub struct PaymentService;

impl PaymentService {
    pub fn init_payment(
        pool: &DbPool,
        booking_id: Uuid,
        method: &str,
    ) -> Result<(String, f64), AppError> {
        // Implementation
    }
}
```
- Services are stateless structs with static methods
- All business logic resides in services, not routes
- Services accept `&DbPool` as first parameter
- Return `Result<T, AppError>` from service methods

#### Route Configuration
```rust
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .route("/signup", web::post().to(signup))
            .route("/login", web::post().to(login))
    );
}
```
- Each route module exports a `config` function
- Use `web::scope()` to group related endpoints
- Register all routes in `routes/mod.rs`

#### Validation
- Validate input at the route handler level before calling services
- Check for empty strings, invalid formats, and business rule violations
- Return early with `AppError::ValidationError` for invalid input

### TypeScript Frontend Standards

#### File Organization
- Use Next.js App Router structure (app directory)
- Place reusable components in `components/` directory
- Group related components in subdirectories (e.g., `components/ui/`, `components/dashboard/`)
- Keep API client logic in `lib/api.ts`

#### Naming Conventions
- **Files**: PascalCase for components (e.g., `Hero3D.tsx`), camelCase for utilities (e.g., `api.ts`)
- **Components**: PascalCase (e.g., `Button`, `Hero3D`)
- **Functions**: camelCase (e.g., `getUserPortfolio`, `initPayment`)
- **Interfaces**: PascalCase (e.g., `Asset`, `User`, `Payment`)
- **Constants**: SCREAMING_SNAKE_CASE or camelCase for config

#### Component Patterns
```typescript
'use client'  // Only when using client-side features

import { ComponentName } from '@/components/...'
import { motion } from 'framer-motion'

export default function PageName() {
  return (
    <main className="min-h-screen">
      {/* Content */}
    </main>
  )
}
```
- Use `'use client'` directive only when needed (hooks, events, animations)
- Default to Server Components when possible
- Export default for page components
- Use named exports for reusable components

#### API Client Pattern
```typescript
export const api = {
  auth: {
    signup: (data: SignupData) => apiClient.post<User>('/users/signup', data),
    login: (data: LoginData) => apiClient.post<User>('/users/login', data),
  },
  payments: {
    init: (data: InitData) => apiClient.post('/payments/init', data),
    confirm: (data: ConfirmData) => apiClient.post('/payments/confirm', data),
  },
};
```
- Group API methods by domain (auth, payments, orders, etc.)
- Use TypeScript interfaces for request/response types
- Include JWT token automatically via interceptor
- Use environment variables for API base URL

#### Type Definitions
- Define interfaces for all API responses
- Export interfaces from `lib/api.ts`
- Use explicit types for function parameters and return values
- Avoid `any` type - use `unknown` or proper types

#### Styling Conventions
- Use Tailwind CSS utility classes
- Custom colors: `text-gold`, `bg-gold`, `border-gold`
- Glass morphism: `glass-card` class for frosted glass effect
- Animations: Use Framer Motion for complex animations
- Responsive: Mobile-first with `md:`, `lg:` breakpoints

### Smart Contract Standards

#### Hardhat Configuration
```javascript
module.exports = {
  solidity: {
    version: "0.8.20",
    settings: {
      optimizer: {
        enabled: true,
        runs: 200
      }
    }
  },
  networks: {
    mainnet: { /* config */ },
    sepolia: { /* config */ },
  }
};
```
- Enable optimizer for production deployments
- Use environment variables for sensitive data (private keys, API keys)
- Support multiple networks (mainnet, testnet)
- Include etherscan API keys for verification

#### Deployment Scripts
```javascript
async function main() {
  console.log("Deploying ContractName...");
  
  const Contract = await hre.ethers.getContractFactory("ContractName");
  const contract = await Contract.deploy();
  await contract.waitForDeployment();
  
  const address = await contract.getAddress();
  console.log(`Contract deployed to: ${address}`);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
```
- Log deployment progress
- Wait for deployment confirmation
- Handle network-specific configurations
- Provide verification commands in output

## Architectural Patterns

### Layered Architecture (Backend)
```
Routes (HTTP handlers)
    ↓
Services (Business logic)
    ↓
Database (Data persistence)
```
- Routes handle HTTP concerns (parsing, validation, responses)
- Services contain all business logic
- Direct database access only from services

### API Client Pattern (Frontend)
```typescript
const apiClient = axios.create({
  baseURL: API_BASE_URL,
  headers: { 'Content-Type': 'application/json' },
});

apiClient.interceptors.request.use((config) => {
  const token = localStorage.getItem('token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});
```
- Centralized Axios instance
- Automatic JWT token injection
- Consistent error handling

### Component Composition (Frontend)
- Build complex UIs from small, reusable components
- Use Radix UI primitives for accessible base components
- Extend with custom styling using Tailwind CSS
- Compose with `class-variance-authority` for variants

## Common Implementation Patterns

### Authentication Flow
```rust
// Backend: Generate JWT token
let token = auth::generate_token(&user.id, &user.role)?;

// Frontend: Store and use token
localStorage.setItem('token', token);
// Automatically included in all requests via interceptor
```

### Payment Processing Flow
```rust
// 1. Initialize payment
let (wallet_address, amount) = PaymentService::init_payment(pool, booking_id, method)?;

// 2. User sends crypto to wallet_address

// 3. Confirm payment with transaction hash
PaymentService::confirm_payment(pool, booking_id, tx_hash)?;
```

### Database Query Pattern
```rust
let mut conn = pool.get()
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

let row = conn.query_one(
    "SELECT * FROM table WHERE id = $1",
    &[&id],
).map_err(|_| AppError::NotFound)?;
```

### Animation Pattern (Frontend)
```typescript
<motion.div
  initial={{ opacity: 0, y: 20 }}
  whileInView={{ opacity: 1, y: 0 }}
  viewport={{ once: true }}
  transition={{ delay: index * 0.2 }}
  whileHover={{ y: -10 }}
>
  {/* Content */}
</motion.div>
```
- Use `initial` and `whileInView` for scroll animations
- Add `viewport={{ once: true }}` to prevent re-animation
- Use `whileHover` for interactive feedback
- Stagger animations with delay based on index

## Configuration Management

### Environment Variables

#### Backend (.env)
```
DATABASE_URL=postgres://user:pass@localhost/aeroledger
JWT_SECRET=your-secret-key
RUST_LOG=info
```

#### Frontend (.env.local)
```
NEXT_PUBLIC_API_URL=http://127.0.0.1:8080
```
- Prefix public variables with `NEXT_PUBLIC_`
- Never commit .env files to version control
- Provide .env.example files as templates

### CORS Configuration
```rust
let cors = Cors::default()
    .allowed_origin("http://localhost:3000")
    .allowed_origin("http://localhost:3001")
    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
    .allowed_headers(vec!["Content-Type", "Authorization"])
    .max_age(3600);
```
- Allow specific origins (no wildcards in production)
- Specify allowed methods explicitly
- Include Authorization header for JWT

### Rate Limiting
```rust
let governor_conf = GovernorConfigBuilder::default()
    .per_second(2)
    .burst_size(100)
    .finish()
    .unwrap();
```
- Configure reasonable limits (100 requests/minute)
- Apply globally via middleware

## Testing Practices

### Backend Testing
- Write unit tests for service layer logic
- Test error handling paths
- Mock database connections for isolated tests
- Use `cargo test` to run all tests

### Frontend Testing
- Test component rendering
- Test user interactions
- Test API integration
- Use Jest and React Testing Library

## Documentation Standards

### Code Comments
- Document complex business logic
- Explain non-obvious decisions
- Use doc comments (`///`) for public APIs in Rust
- Use JSDoc for TypeScript functions

### API Documentation
- Document all endpoints with method, path, request, and response
- Include example payloads
- Document error responses
- Keep documentation in sync with implementation

## Security Best Practices

### Backend Security
- Use parameterized queries (never string concatenation)
- Hash passwords with bcrypt
- Validate all user input
- Use JWT for authentication
- Implement rate limiting
- Enable CORS with specific origins

### Frontend Security
- Store JWT in localStorage (consider httpOnly cookies for production)
- Validate user input before sending to API
- Sanitize user-generated content
- Use HTTPS in production
- Never expose API keys in client code

### Smart Contract Security
- Use OpenZeppelin libraries for standard functionality
- Enable optimizer for gas efficiency
- Audit contracts before mainnet deployment
- Use testnet for initial deployments
- Implement access control (onlyOwner, etc.)

## Performance Optimization

### Backend Performance
- Use connection pooling
- Implement async/await throughout
- Index database columns used in WHERE clauses
- Cache frequently accessed data
- Use efficient serialization (serde)

### Frontend Performance
- Use Next.js Server Components by default
- Implement code splitting
- Optimize images with Next.js Image component
- Lazy load heavy components
- Minimize bundle size

## Deployment Checklist

### Backend Deployment
- [ ] Set production DATABASE_URL
- [ ] Generate secure JWT_SECRET
- [ ] Enable HTTPS
- [ ] Configure production CORS origins
- [ ] Set up database backups
- [ ] Configure logging
- [ ] Set up monitoring

### Frontend Deployment
- [ ] Set production API URL
- [ ] Build with `npm run build`
- [ ] Enable production optimizations
- [ ] Configure CDN for static assets
- [ ] Set up error tracking
- [ ] Configure analytics

### Smart Contract Deployment
- [ ] Audit contract code
- [ ] Test on testnet
- [ ] Verify contract on Etherscan/BSCScan
- [ ] Document contract addresses
- [ ] Set up monitoring for contract events
