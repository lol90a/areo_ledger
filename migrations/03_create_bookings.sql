CREATE TABLE bookings (
  id TEXT PRIMARY KEY,
  user_id TEXT REFERENCES users(id),
  flight_id TEXT REFERENCES flights(id),
  status TEXT NOT NULL,
  base_price REAL NOT NULL,
  markup REAL NOT NULL,
  service_fee REAL NOT NULL,
  total_price REAL NOT NULL,
  payment_method TEXT NOT NULL,
  payment_address TEXT,
  tx_hash TEXT,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
