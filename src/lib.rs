// AeroLedger — Clean Architecture root
//
// Dependency rule (enforced by Rust's module system):
//
//   shared  <── domain <── application <── infrastructure
//                                      <── interfaces
//
// Nothing in `domain` imports from `infrastructure` or `interfaces`.
// Nothing in `application` imports from `infrastructure` or `interfaces`.

pub mod shared;
pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod interfaces;
