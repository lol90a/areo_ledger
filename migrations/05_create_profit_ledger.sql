CREATE TABLE profit_ledger (
  id TEXT PRIMARY KEY,
  booking_id TEXT REFERENCES bookings(id),
  base_price REAL NOT NULL,
  profit_amount REAL NOT NULL,
  token TEXT NOT NULL,
  chain TEXT NOT NULL,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
