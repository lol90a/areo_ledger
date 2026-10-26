//! AeroLedger backend.
//!
//! Clean-architecture layering: `domain` holds entities, value objects and port
//! traits and depends on nothing else; `application` orchestrates use cases over
//! those ports; `infrastructure` supplies the adapters (Postgres, chain gateways,
//! price oracle, workers); `interfaces` is the HTTP edge.
//!
//! ## Lint policy
//!
//! `cargo clippy --all-targets -- -D warnings` is a quality gate for this crate.
//! Four lints are allowed crate-wide, each for a deliberate convention rather than
//! to silence a real finding:

// Money is written in minor units with the decimal point marked by the grouping,
// so `1_092_500` cents reads as `10_925_00` -> $10,925.00. Clippy wants uniform
// 3-digit groups, which would make every monetary literal in the tests harder to
// check against the dollar figure it represents.
#![allow(clippy::inconsistent_digit_grouping)]
// Status enums expose an infallible `from_str(&str) -> Self` that maps anything
// unrecognised to the safe default (e.g. an unknown payment status reads as
// `Pending`). `std::str::FromStr` is fallible, so implementing it would force every
// database read to handle an error that this design deliberately does not have.
#![allow(clippy::should_implement_trait)]
// Entity constructors take one argument per field by design: they are the single
// place field order is fixed, and grouping them into a parameter struct would just
// move the same arity behind another type.
#![allow(clippy::too_many_arguments)]
// `#[async_trait]` desugars a trait method into a `#[must_use]` function that
// returns a `#[must_use]` boxed Future, which rustc 1.99's `double_must_use`
// flags on every one of them -- 31 errors across the repository ports, none of
// them in code anyone wrote. The lint is about the macro's output, and the only
// alternative is dropping the whole clippy gate.
#![allow(clippy::double_must_use)]

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod interfaces;
pub mod shared;
