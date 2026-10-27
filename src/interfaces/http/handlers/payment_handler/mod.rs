//! The payment HTTP surface: options, quoting, settlement and manual proof.
//!
//! Split by use case rather than by layer. Each submodule owns the wire types it
//! serves and the handler that serves them, so a change to the quote flow lands
//! in one file instead of being spread across a `dto` module, a `handlers`
//! module and a `responses` module.
//!
//! Only three things are genuinely shared, and each has a module of its own:
//! repository construction, booking authorization, and audit logging. Splitting
//! the handlers without extracting those first would have multiplied them --
//! the same twelve-line audit block already appeared five times in one file, and
//! scattering it over five files would have made it harder to see, not easier.

use actix_web::web;

mod access;
mod audit;
mod options;
mod proof;
mod quote;
mod repositories;
mod settlement;

// These were reachable as `payment_handler::CreateQuoteRequest` before the split.
// Re-exporting keeps that path working, so the move is invisible from outside.
pub use quote::{AcceptQuoteRequest, CreateQuoteRequest};
pub use settlement::{ConfirmPaymentRequest, InitPaymentRequest};

/// The routing table, kept whole in one place: the set of endpoints this scope
/// exposes is the one thing you should not have to open six files to read.
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/payments")
            .route("/options", web::get().to(options::payment_options))
            .route("/quote", web::post().to(quote::create_quote))
            .route("/quote/accept", web::post().to(quote::accept_quote))
            .route("/init", web::post().to(settlement::init_payment))
            .route("/confirm", web::post().to(settlement::confirm_payment))
            .route(
                "/proof/{booking_id}",
                web::post().to(proof::upload_payment_proof),
            ),
    );
}
