# 🔧 Backend API Documentation

## ✅ Completed Endpoints

### 🔐 Authentication (`/api/users`)

#### POST `/api/users/signup`
Create new user account
```json
Request:
{
  "email": "user@test.com",
  "name": "John Doe",
  "password": "password123"
}

Response:
{
  "id": "uuid",
  "email": "user@test.com",
  "name": "John Doe",
  "role": "user"
}
```

#### POST `/api/users/login`
Login user (auto-creates if not exists)
```json
Request:
{
  "email": "user@test.com",
  "password": "password123"
}

Response:
{
  "id": "uuid",
  "email": "user@test.com",
  "name": "John Doe",
  "role": "user"
}
```

#### GET `/api/users/{id}`
Get user by ID

---

### ✈️ Flights (`/api/flights`)

#### GET `/api/flights?search={destination}`
Search flights
```json
Response:
[
  {
    "id": "1",
    "origin": "New York",
    "destination": "London",
    "departure_time": "2026-01-25T10:00:00Z",
    "price_usd": 500.0
  }
]
```

#### GET `/api/flights/{id}`
Get flight details

---

### 📋 Bookings (`/api/bookings`)

#### POST `/api/bookings`
Create booking
```json
Request:
{
  "user_id": "uuid",
  "flight_id": "1",
  "base_price": 500.0,
  "payment_method": "usdt"
}

Response:
{
  "booking_id": "uuid",
  "total_price": 575.0
}
```

---

### 💰 Payments (`/api/payments`)

#### POST `/api/payments/init`
Initialize payment
```json
Request:
{
  "booking_id": "uuid",
  "method": "usdt"
}

Response:
{
  "payment_id": "uuid",
  "receiver_address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
  "amount": 575.0,
  "chain": "Ethereum",
  "token": "USDT"
}
```

#### POST `/api/payments/confirm`
Confirm payment
```json
Request:
{
  "booking_id": "uuid",
  "tx_hash": "0x1234567890abcdef..."
}

Response:
{
  "status": "confirmed",
  "message": "Payment confirmed successfully"
}
```

---

### 👨‍💼 Admin (`/api/admin`)

#### GET `/api/admin/stats`
Get dashboard statistics
```json
Response:
{
  "bookings": 156,
  "revenue": 2450000.0,
  "users": 892,
  "flights": 45
}
```

#### GET `/api/admin/bookings`
Get all bookings (limit 10)

#### GET `/api/admin/payments`
Get all payments (limit 10)

---

## 🗄️ Database Schema

### users
- id (TEXT, PRIMARY KEY)
- email (TEXT, UNIQUE)
- name (TEXT)
- role (TEXT) - 'user' or 'admin'
- wallet_address (TEXT)
- created_at (DATETIME)

### flights
- id (TEXT, PRIMARY KEY)
- provider (TEXT)
- from_airport (TEXT)
- to_airport (TEXT)
- departure_time (DATETIME)
- arrival_time (DATETIME)
- base_price (REAL)
- currency (TEXT)

### bookings
- id (TEXT, PRIMARY KEY)
- user_id (TEXT, FK)
- flight_id (TEXT, FK)
- status (TEXT) - 'pending', 'confirmed'
- base_price (REAL)
- markup (REAL) - 10%
- service_fee (REAL) - 5%
- total_price (REAL) - base + markup + fee
- payment_method (TEXT)
- created_at (DATETIME)

### payments
- id (TEXT, PRIMARY KEY)
- booking_id (TEXT, FK)
- chain (TEXT) - Ethereum, Bitcoin, Solana, BSC
- token (TEXT) - USDT, USDC, ETH, BTC, SOL, BNB
- amount (REAL)
- sender_address (TEXT)
- receiver_address (TEXT)
- tx_hash (TEXT)
- status (TEXT) - 'pending', 'confirmed'
- created_at (DATETIME)

---

## 🚀 Running Backend

```bash
# Run migrations
cargo run

# Server starts on http://127.0.0.1:8080
```

---

## 🔑 Supported Crypto

1. **USDT** - Ethereum Network
2. **USDC** - Ethereum Network
3. **ETH** - Ethereum Network
4. **BTC** - Bitcoin Network
5. **SOL** - Solana Network
6. **BNB** - BSC Network

---

## ✅ What's Working

✅ User signup/login with auto-creation
✅ Role detection (admin if email contains "admin")
✅ Flight search with mock data
✅ Booking creation with 15% markup
✅ Payment initialization with crypto details
✅ Payment confirmation with tx hash
✅ Admin stats endpoint
✅ Admin bookings/payments list
✅ Database migrations
✅ Error handling
✅ CORS enabled

---

## 🔧 Tech Stack

- **Framework**: Actix-web 4.12
- **Database**: SQLite with SQLx
- **Validation**: Validator
- **Serialization**: Serde
- **UUID**: uuid v1.0

---

## 📝 Notes

- All endpoints return JSON
- Errors return proper HTTP status codes
- Database auto-creates on first run
- Mock flight data for demo
- Receiver address is hardcoded for demo
- Auto-creates user on login if not exists
- Admin role auto-assigned if email contains "admin"
