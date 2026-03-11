use actix_web::web;

mod health;
mod flights;
mod bookings;
mod payments;
mod users;
mod admin;
mod products;
mod orders;
mod assets;
mod portfolio;
mod transactions;
mod calculator;
mod order_tracking;
mod profile;
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
            .configure(products::config)
            .configure(orders::config)
            .configure(assets::config)
            .configure(portfolio::config)
            .configure(transactions::config)
            .configure(calculator::config)
            .configure(order_tracking::configure)
            .configure(profile::configure)
    );
}
