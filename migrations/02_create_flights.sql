CREATE TABLE flights (
  id TEXT PRIMARY KEY,
  provider TEXT NOT NULL,
  from_airport TEXT NOT NULL,
  to_airport TEXT NOT NULL,
  departure_time DATETIME NOT NULL,
  arrival_time DATETIME NOT NULL,
  base_price REAL NOT NULL,
  currency TEXT NOT NULL
);
