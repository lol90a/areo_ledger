CREATE TABLE payments (
  id TEXT PRIMARY KEY,
  booking_id TEXT REFERENCES bookings(id),
  chain TEXT NOT NULL,
  token TEXT NOT NULL,
  amount REAL NOT NULL,
  sender_address TEXT,
  receiver_address TEXT NOT NULL,
  tx_hash TEXT,
  status TEXT NOT NULL,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
