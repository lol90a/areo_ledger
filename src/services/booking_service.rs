use uuid::Uuid;
use crate::services::profit_engine::calculate_price;
use crate::errors::AppError;

pub struct BookingInput {
    pub user_id: Uuid,
    pub flight_id: Uuid,
    pub base_price: f64,
    pub payment_method: String,
}

pub async fn create_booking(
    pool: &sqlx::PgPool,
    input: BookingInput,
) -> Result<Uuid, AppError> {
    let breakdown = calculate_price(input.base_price);

    let booking_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO bookings (
            id,
            user_id,
            flight_id,
            status,
            base_price,
            markup,
            service_fee,
            total_price,
            payment_method
        )
        VALUES (?,?,?,'pending',?,?,?,?,?)
        "#,
    )
    .bind(booking_id)
    .bind(input.user_id)
    .bind(input.flight_id)
    .bind(breakdown.base_price)
    .bind(breakdown.markup)
    .bind(breakdown.service_fee)
    .bind(breakdown.total)
    .bind(input.payment_method)
    .execute(pool)
    .await?;

    Ok(booking_id)
}
