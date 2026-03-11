use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct OrderStatusUpdate {
    pub status: String,
    pub notes: Option<String>,
}

#[derive(Serialize)]
pub struct OrderTrackingResponse {
    pub order_id: Uuid,
    pub status: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub history: Vec<OrderHistoryItem>,
}

#[derive(Serialize)]
pub struct OrderHistoryItem {
    pub status: String,
    pub notes: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}

pub async fn get_order_tracking(
    pool: web::Data<PgPool>,
    order_id: web::Path<Uuid>,
) -> impl Responder {
    let order_id = order_id.into_inner();

    let order = sqlx::query!(
        "SELECT id, status, created_at, updated_at FROM orders WHERE id = $1",
        order_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match order {
        Ok(Some(order)) => {
            let history = sqlx::query_as!(
                OrderHistoryItem,
                "SELECT status, notes, created_at FROM order_status_history WHERE order_id = $1 ORDER BY created_at DESC",
                order_id
            )
            .fetch_all(pool.get_ref())
            .await
            .unwrap_or_default();

            HttpResponse::Ok().json(OrderTrackingResponse {
                order_id: order.id,
                status: order.status,
                created_at: order.created_at,
                updated_at: order.updated_at,
                history,
            })
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({"error": "Order not found"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn update_order_status(
    pool: web::Data<PgPool>,
    order_id: web::Path<Uuid>,
    update: web::Json<OrderStatusUpdate>,
) -> impl Responder {
    let order_id = order_id.into_inner();

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    let result = sqlx::query!(
        "UPDATE orders SET status = $1, updated_at = NOW() WHERE id = $2",
        update.status,
        order_id
    )
    .execute(&mut *tx)
    .await;

    if let Err(e) = result {
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}));
    }

    let history_result = sqlx::query!(
        "INSERT INTO order_status_history (order_id, status, notes) VALUES ($1, $2, $3)",
        order_id,
        update.status,
        update.notes
    )
    .execute(&mut *tx)
    .await;

    if let Err(e) = history_result {
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}));
    }

    if let Err(e) = tx.commit().await {
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}));
    }

    HttpResponse::Ok().json(serde_json::json!({"message": "Order status updated"}))
}

pub async fn get_user_orders(
    pool: web::Data<PgPool>,
    user_id: web::Path<Uuid>,
) -> impl Responder {
    let user_id = user_id.into_inner();

    let orders = sqlx::query!(
        r#"
        SELECT o.id, o.status, o.total_amount, o.created_at, o.updated_at,
               COUNT(oi.id) as item_count
        FROM orders o
        LEFT JOIN order_items oi ON o.id = oi.order_id
        WHERE o.user_id = $1
        GROUP BY o.id
        ORDER BY o.created_at DESC
        "#,
        user_id
    )
    .fetch_all(pool.get_ref())
    .await;

    match orders {
        Ok(orders) => HttpResponse::Ok().json(orders),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/orders")
            .route("/{order_id}/tracking", web::get().to(get_order_tracking))
            .route("/{order_id}/status", web::put().to(update_order_status))
            .route("/user/{user_id}", web::get().to(get_user_orders))
    );
}
