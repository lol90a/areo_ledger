use r2d2_postgres::{PostgresConnectionManager, r2d2};
use postgres::NoTls;

pub type DbPool = r2d2::Pool<PostgresConnectionManager<NoTls>>;

pub fn init_db(database_url: &str) -> DbPool {
    let manager = PostgresConnectionManager::new(
        database_url.parse().expect("Invalid database URL"),
        NoTls,
    );

    r2d2::Pool::builder()
        .max_size(15)
        .build(manager)
        .expect("Failed to create pool")
}

pub fn create_tables(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = pool.get()?;

    conn.batch_execute(
        "
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY,
            email VARCHAR(255) NOT NULL UNIQUE,
            name VARCHAR(255) NOT NULL,
            role VARCHAR(50) NOT NULL DEFAULT 'user',
            password_hash VARCHAR(255) NOT NULL,
            wallet_address VARCHAR(255),
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS bookings (
            id UUID PRIMARY KEY,
            user_id UUID,
            flight_id UUID,
            status VARCHAR(50) NOT NULL,
            base_price DECIMAL(15, 2) NOT NULL,
            markup DECIMAL(15, 2) NOT NULL,
            service_fee DECIMAL(15, 2) NOT NULL,
            total_price DECIMAL(15, 2) NOT NULL,
            payment_method VARCHAR(50) NOT NULL,
            payment_address VARCHAR(255),
            tx_hash VARCHAR(255),
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS payments (
            id UUID PRIMARY KEY,
            booking_id UUID,
            chain VARCHAR(50) NOT NULL,
            token VARCHAR(50) NOT NULL,
            amount DECIMAL(15, 8) NOT NULL,
            sender_address VARCHAR(255),
            receiver_address VARCHAR(255) NOT NULL,
            tx_hash VARCHAR(255),
            status VARCHAR(50) NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS orders (
            id UUID PRIMARY KEY,
            user_id UUID NOT NULL,
            total DECIMAL(15, 2) NOT NULL,
            service_fee DECIMAL(15, 2) NOT NULL,
            grand_total DECIMAL(15, 2) NOT NULL,
            crypto_method VARCHAR(50) NOT NULL,
            sender_wallet VARCHAR(255) NOT NULL,
            tx_hash VARCHAR(255),
            wallet_address VARCHAR(255) NOT NULL,
            crypto_amount DECIMAL(15, 8) NOT NULL,
            status VARCHAR(50) NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS order_items (
            id SERIAL PRIMARY KEY,
            order_id UUID NOT NULL,
            product_id INTEGER NOT NULL,
            name VARCHAR(255) NOT NULL,
            price DECIMAL(15, 2) NOT NULL,
            quantity INTEGER NOT NULL
        );
        "
    )?;

    Ok(())
}
