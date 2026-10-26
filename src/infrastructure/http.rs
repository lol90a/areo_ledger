//! Shared outbound HTTP client construction.
//!
//! Exists because of a silent failure. `reqwest` sends no `User-Agent` unless
//! told to, and CoinGecko answers an agent-less request with **HTTP 403**. The
//! price oracle treated that as "no price available" and quietly fell back to
//! the seeded `crypto_prices` row, so every quote in live mode was priced from
//! a stale placeholder while reporting itself as working. Nothing logged an
//! error, because from the oracle's point of view nothing had gone wrong.
//!
//! Third-party APIs increasingly reject or throttle unidentified clients, so the
//! header belongs on every outbound client rather than only the one that was
//! caught. Building them all here means a new adapter cannot forget it.

use reqwest::Client;
use std::time::Duration;

/// Identifies this backend to the APIs it calls.
///
/// CoinGecko's rejection message asks for a *descriptive* agent, and providers
/// use it to contact an operator about traffic rather than silently blocking
/// them, so it names the project rather than imitating a browser.
const USER_AGENT: &str = concat!(
    "aeroledger-backend/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/lol90a/areo_ledger)"
);

/// An HTTP client with the shared identity and an explicit timeout.
///
/// Falls back to a default client if the builder fails, which keeps a
/// misconfiguration from taking the process down at startup -- the caller's own
/// error handling still covers the request itself.
pub fn client(timeout: Duration) -> Client {
    Client::builder()
        .timeout(timeout)
        .user_agent(USER_AGENT)
        .build()
        .unwrap_or_else(|_| Client::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_user_agent_is_descriptive_and_versioned() {
        // CoinGecko rejects agent-less requests outright, and an empty or
        // generic value is what it is asking to avoid.
        assert!(USER_AGENT.starts_with("aeroledger-backend/"));
        assert!(USER_AGENT.contains(env!("CARGO_PKG_VERSION")));
        assert!(USER_AGENT.len() > 20);
    }

    #[test]
    fn a_client_builds() {
        let _ = client(Duration::from_secs(5));
    }
}
