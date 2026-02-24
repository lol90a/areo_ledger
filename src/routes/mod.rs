use actix_web::web;

mod health;
mod flights;
mod bookings;
mod payments;
mod users;
mod admin;
pub mod dto;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .configure(health::config)
            .configure(flights::config)
            .configure(bookings::config)
            .configure(payments::config)
            .configure(users::config)
            .configure(admin::config)
    );
}
