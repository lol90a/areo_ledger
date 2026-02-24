CREATE TABLE IF NOT EXISTS payments (
  id TEXT PRIMARY KEY,
  booking_id TEXT REFERENCES bookings(id),
  chain TEXT NOT NULL,
  token TEXT NOT NULL,
  amount DECIMAL(10,2) NOT NULL,
  sender_address TEXT,
  receiver_address TEXT NOT NULL,
  tx_hash TEXT,
  status TEXT NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_payments_booking_id ON payments(booking_id);
CREATE INDEX idx_payments_status ON payments(status);
CREATE INDEX idx_payments_tx_hash ON payments(tx_hash);
CREATE INDEX idx_payments_created_at ON payments(created_at DESC);
