CREATE TABLE IF NOT EXISTS bookings (
  id TEXT PRIMARY KEY,
  user_id TEXT REFERENCES users(id),
  flight_id TEXT NOT NULL,
  status TEXT NOT NULL,
  base_price DECIMAL(10,2) NOT NULL,
  markup DECIMAL(10,2) NOT NULL,
  service_fee DECIMAL(10,2) NOT NULL,
  total_price DECIMAL(10,2) NOT NULL,
  payment_method TEXT NOT NULL,
  payment_address TEXT,
  tx_hash TEXT,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_bookings_user_id ON bookings(user_id);
CREATE INDEX idx_bookings_status ON bookings(status);
CREATE INDEX idx_bookings_created_at ON bookings(created_at DESC);
