-- AeroLedger Complete Schema
-- Safe to run multiple times — uses IF NOT EXISTS throughout

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ── Users ─────────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS users (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email               VARCHAR(255) NOT NULL UNIQUE,
    name                VARCHAR(255) NOT NULL,
    role                VARCHAR(50)  NOT NULL DEFAULT 'user',
    password_hash       TEXT NOT NULL,
    wallet_address_btc  VARCHAR(255),
    wallet_address_eth  VARCHAR(255),
    wallet_address_sol  VARCHAR(255),
    phone               VARCHAR(50),
    avatar_url          TEXT,
    email_verified      BOOLEAN DEFAULT FALSE,
    two_factor_enabled  BOOLEAN DEFAULT FALSE,
    created_at          TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at          TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_login          TIMESTAMP
);

-- ── Flights ───────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS flights (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    route           VARCHAR(255) NOT NULL,
    origin          VARCHAR(100) NOT NULL,
    destination     VARCHAR(100) NOT NULL,
    aircraft        VARCHAR(255) NOT NULL,
    price_usd       DECIMAL(15, 2) NOT NULL,
    duration        VARCHAR(50),
    seats           INTEGER NOT NULL,
    departure_time  TIMESTAMP NOT NULL,
    arrival_time    TIMESTAMP,
    status          VARCHAR(50) DEFAULT 'available',
    created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ── Products ──────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS products (
    id          SERIAL PRIMARY KEY,
    name        VARCHAR(255) NOT NULL,
    category    VARCHAR(100) NOT NULL,
    price       DECIMAL(15, 2) NOT NULL,
    image       TEXT,
    specs       TEXT,
    description TEXT,
    features    JSONB,
    stock       INTEGER DEFAULT 1,
    status      VARCHAR(50) DEFAULT 'available',
    created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ── Bookings ──────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS bookings (
    id                 UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id            UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    flight_id          UUID REFERENCES flights(id) ON DELETE SET NULL,
    base_price         DECIMAL(15, 2) NOT NULL,
    markup_percentage  DECIMAL(5, 2) DEFAULT 10.00,
    total_price        DECIMAL(15, 2) NOT NULL,
    payment_method     VARCHAR(50) NOT NULL,
    status             VARCHAR(50) DEFAULT 'pending',
    booking_reference  VARCHAR(50) UNIQUE,
    tx_hash            TEXT,
    created_at         TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at         TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'bookings' AND column_name = 'id' AND data_type <> 'uuid'
    ) THEN
        ALTER TABLE bookings ALTER COLUMN id DROP DEFAULT;
        ALTER TABLE bookings ALTER COLUMN id TYPE UUID USING NULLIF(id::TEXT, '')::UUID;
        ALTER TABLE bookings ALTER COLUMN id SET DEFAULT uuid_generate_v4();
    END IF;

    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'bookings' AND column_name = 'user_id' AND data_type <> 'uuid'
    ) THEN
        ALTER TABLE bookings ALTER COLUMN user_id TYPE UUID USING NULLIF(user_id::TEXT, '')::UUID;
    END IF;

    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'bookings' AND column_name = 'flight_id' AND data_type <> 'uuid'
    ) THEN
        ALTER TABLE bookings ALTER COLUMN flight_id TYPE UUID USING NULLIF(flight_id::TEXT, '')::UUID;
    END IF;
END $$;

ALTER TABLE bookings ADD COLUMN IF NOT EXISTS markup_percentage DECIMAL(5, 2) DEFAULT 10.00;
ALTER TABLE bookings ADD COLUMN IF NOT EXISTS booking_reference VARCHAR(50);
ALTER TABLE bookings ADD COLUMN IF NOT EXISTS tx_hash TEXT;
ALTER TABLE bookings ADD COLUMN IF NOT EXISTS updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP;

-- ── Orders ────────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS orders (
    id               UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id          UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    total            DECIMAL(15, 2) NOT NULL,
    service_fee      DECIMAL(15, 2) NOT NULL,
    grand_total      DECIMAL(15, 2) NOT NULL,
    crypto_method    VARCHAR(50) NOT NULL,
    sender_wallet    TEXT,
    public_key       TEXT,
    tx_hash          TEXT,
    wallet_address   TEXT NOT NULL,
    crypto_amount    DECIMAL(20, 8) NOT NULL,
    status           VARCHAR(50) DEFAULT 'pending',
    order_reference  VARCHAR(50) UNIQUE,
    created_at       TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at       TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ── Payments (after bookings AND orders) ─────────────────────────────────────
CREATE TABLE IF NOT EXISTS payments (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    booking_id      UUID REFERENCES bookings(id) ON DELETE CASCADE,
    order_id        UUID REFERENCES orders(id) ON DELETE CASCADE,
    chain           VARCHAR(50) NOT NULL,
    token           VARCHAR(50) NOT NULL,
    amount          DECIMAL(20, 8) NOT NULL,
    amount_usd      DECIMAL(15, 2) NOT NULL DEFAULT 0,
    wallet_address  TEXT NOT NULL,
    sender_wallet   TEXT,
    tx_hash         TEXT,
    status          VARCHAR(50) DEFAULT 'pending',
    verified        BOOLEAN DEFAULT FALSE,
    verified_at     TIMESTAMP,
    created_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ── Order Items ───────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS order_items (
    id          SERIAL PRIMARY KEY,
    order_id    UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    product_id  INTEGER NOT NULL REFERENCES products(id),
    name        VARCHAR(255) NOT NULL,
    price       DECIMAL(15, 2) NOT NULL,
    quantity    INTEGER NOT NULL DEFAULT 1,
    created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ── Assets ────────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS assets (
    id                UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name              VARCHAR(255) NOT NULL,
    type              VARCHAR(100) NOT NULL,
    description       TEXT,
    image             TEXT,
    total_value       DECIMAL(15, 2) NOT NULL,
    token_price       DECIMAL(15, 2) NOT NULL,
    available_tokens  INTEGER NOT NULL,
    total_tokens      INTEGER NOT NULL,
    roi               DECIMAL(5, 2),
    location          VARCHAR(255),
    specifications    JSONB,
    status            VARCHAR(50) DEFAULT 'active',
    created_at        TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ── Portfolio ─────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS portfolio (
    id                UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id           UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    asset_id          UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    tokens_owned      INTEGER NOT NULL,
    investment_value  DECIMAL(15, 2) NOT NULL,
    current_value     DECIMAL(15, 2) NOT NULL,
    purchase_date     TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, asset_id)
);

-- ── Transactions ──────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS transactions (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    type        VARCHAR(50) NOT NULL,
    asset       VARCHAR(255),
    amount      DECIMAL(15, 2) NOT NULL,
    tokens      INTEGER DEFAULT 0,
    status      VARCHAR(50) DEFAULT 'completed',
    tx_hash     TEXT,
    date        TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ── Favorites ─────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS favorites (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    product_id  INTEGER NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, product_id)
);

-- ── Reviews ───────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS reviews (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    product_id  INTEGER REFERENCES products(id) ON DELETE CASCADE,
    flight_id   UUID REFERENCES flights(id) ON DELETE CASCADE,
    rating      INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    comment     TEXT,
    created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ── Referrals ─────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS referrals (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    referrer_id   UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    referred_id   UUID REFERENCES users(id) ON DELETE SET NULL,
    code          VARCHAR(50) NOT NULL UNIQUE,
    status        VARCHAR(50) DEFAULT 'pending',
    reward_amount DECIMAL(15, 2) DEFAULT 0,
    created_at    TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    completed_at  TIMESTAMP
);

-- ── Email Notifications ───────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS email_notifications (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    type          VARCHAR(100) NOT NULL,
    subject       VARCHAR(255) NOT NULL,
    body          TEXT NOT NULL,
    status        VARCHAR(50) DEFAULT 'pending',
    sent_at       TIMESTAMP,
    error_message TEXT,
    created_at    TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ── Wallet Addresses ──────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS wallet_addresses (
    id         SERIAL PRIMARY KEY,
    chain      VARCHAR(50) NOT NULL UNIQUE,
    address    TEXT NOT NULL,
    is_active  BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ── Crypto Prices Cache ───────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS crypto_prices (
    id           SERIAL PRIMARY KEY,
    symbol       VARCHAR(50) NOT NULL UNIQUE,
    name         VARCHAR(100) NOT NULL,
    price        DECIMAL(20, 8) NOT NULL,
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ── Audit Log ─────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS audit_log (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id     UUID REFERENCES users(id) ON DELETE SET NULL,
    action      VARCHAR(255) NOT NULL,
    entity_type VARCHAR(100),
    entity_id   TEXT,
    old_value   JSONB,
    new_value   JSONB,
    ip_address  VARCHAR(50),
    user_agent  TEXT,
    created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ── Indexes ───────────────────────────────────────────────────────────────────
CREATE INDEX IF NOT EXISTS idx_users_email          ON users(email);
CREATE INDEX IF NOT EXISTS idx_bookings_user_id     ON bookings(user_id);
CREATE INDEX IF NOT EXISTS idx_bookings_status      ON bookings(status);
CREATE INDEX IF NOT EXISTS idx_payments_booking_id  ON payments(booking_id);

DELETE FROM payments p
USING payments dup
WHERE p.booking_id IS NOT NULL
  AND dup.booking_id = p.booking_id
  AND (
      dup.created_at > p.created_at
      OR (dup.created_at = p.created_at AND dup.id > p.id)
  );

CREATE UNIQUE INDEX IF NOT EXISTS idx_payments_booking_unique ON payments(booking_id) WHERE booking_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_payments_tx_hash     ON payments(tx_hash);
CREATE INDEX IF NOT EXISTS idx_orders_user_id       ON orders(user_id);
CREATE INDEX IF NOT EXISTS idx_orders_status        ON orders(status);
CREATE INDEX IF NOT EXISTS idx_portfolio_user_id    ON portfolio(user_id);
CREATE INDEX IF NOT EXISTS idx_transactions_user_id ON transactions(user_id);
CREATE INDEX IF NOT EXISTS idx_favorites_user_id    ON favorites(user_id);
CREATE INDEX IF NOT EXISTS idx_reviews_product_id   ON reviews(product_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_user_id    ON audit_log(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_created_at ON audit_log(created_at);

-- ── updated_at Trigger Function ───────────────────────────────────────────────
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- ── Triggers (drop first so re-runs don't error) ──────────────────────────────
DROP TRIGGER IF EXISTS update_users_updated_at    ON users;
DROP TRIGGER IF EXISTS update_bookings_updated_at ON bookings;
DROP TRIGGER IF EXISTS update_payments_updated_at ON payments;
DROP TRIGGER IF EXISTS update_orders_updated_at   ON orders;

CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_bookings_updated_at
    BEFORE UPDATE ON bookings
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_payments_updated_at
    BEFORE UPDATE ON payments
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_orders_updated_at
    BEFORE UPDATE ON orders
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ── Seed Data (safe — only inserts if not already present) ────────────────────
INSERT INTO wallet_addresses (chain, address) VALUES
    ('BTC',  'bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh'),
    ('ETH',  '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb'),
    ('SOL',  'DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG5CNSKK'),
    ('USDT', '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb'),
    ('USDC', '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb'),
    ('BNB',  '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb')
ON CONFLICT (chain) DO NOTHING;

INSERT INTO crypto_prices (symbol, name, price) VALUES
    ('BTC',  'Bitcoin',      45000.00),
    ('ETH',  'Ethereum',      2500.00),
    ('USDT', 'Tether',           1.00),
    ('USDC', 'USD Coin',         1.00),
    ('SOL',  'Solana',         100.00),
    ('BNB',  'Binance Coin',   320.00)
ON CONFLICT (symbol) DO NOTHING;

-- ── Seed Flights ──────────────────────────────────────────────────────────────
INSERT INTO flights (id, route, origin, destination, aircraft, price_usd, duration, seats, departure_time) VALUES
    ('00000000-0000-0000-0000-000000000001', 'DXB-LHR', 'Dubai', 'London', 'Boeing 777', 9500.00, '7h 30m', 8, NOW() + INTERVAL '7 days'),
    ('00000000-0000-0000-0000-000000000002', 'JFK-LAX', 'New York', 'Los Angeles', 'Gulfstream G700', 25000.00, '5h 45m', 12, NOW() + INTERVAL '3 days'),
    ('00000000-0000-0000-0000-000000000003', 'LHR-DXB', 'London', 'Dubai', 'Airbus A380', 8500.00, '7h 15m', 6, NOW() + INTERVAL '5 days')
ON CONFLICT (id) DO NOTHING;



