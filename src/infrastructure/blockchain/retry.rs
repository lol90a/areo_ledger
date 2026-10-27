use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::time::sleep;

use crate::domain::ports::blockchain_gateway::{BlockchainGateway, TxVerification, VerifyRequest};
use crate::shared::errors::DomainError;

/// Retry and circuit-breaker policy for calls to a blockchain provider.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Attempts per operation. Clamped to at least one by [`RetryConfig::attempts`].
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub jitter_factor: f64,
    /// Consecutive failed attempts that trip the breaker.
    pub circuit_open_after: u32,
    /// How long the breaker stays shut before it will let traffic through again.
    ///
    /// Without this the breaker had no way back: it was only ever reset on a
    /// success, and while it was open no call was made, so no success could
    /// happen. One bad patch of RPC bricked payment verification until the
    /// process was restarted.
    pub circuit_cooldown: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 4,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            jitter_factor: 0.2,
            circuit_open_after: 5,
            circuit_cooldown: Duration::from_secs(30),
        }
    }
}

impl RetryConfig {
    /// Attempts actually made, never zero.
    ///
    /// The fields are public, so `max_attempts: 0` is constructible. Treating it
    /// as "make no call and report the last error" is incoherent -- there is no
    /// last error -- so a zero is read as a single attempt instead.
    fn attempts(&self) -> u32 {
        self.max_attempts.max(1)
    }
}

/// Wraps a gateway with bounded retries and a circuit breaker.
///
/// Retries exist because an RPC provider dropping one request is routine and a
/// payment should not fail for it. The breaker exists because retrying into a
/// provider that is genuinely down multiplies the load on it and makes every
/// caller wait out the full backoff first.
pub struct RetryingBlockchainGateway<G: BlockchainGateway> {
    inner: G,
    config: RetryConfig,
    /// Consecutive failed attempts. Reset by any success.
    failure_count: Arc<AtomicU32>,
    /// When the breaker tripped, or `None` while it is closed.
    opened_at: Arc<Mutex<Option<Instant>>>,
}

impl<G: BlockchainGateway> RetryingBlockchainGateway<G> {
    pub fn new(inner: G, config: RetryConfig) -> Self {
        Self {
            inner,
            config,
            failure_count: Arc::new(AtomicU32::new(0)),
            opened_at: Arc::new(Mutex::new(None)),
        }
    }

    /// Whether the breaker should reject this call without touching the provider.
    ///
    /// This also performs the open -> half-open transition: once the cooldown has
    /// elapsed the timestamp is cleared and the call is let through as a trial.
    /// If that trial fails the breaker trips again immediately, because the
    /// failure count is still above the threshold and only a success clears it.
    fn breaker_is_blocking(&self) -> bool {
        // A poisoned lock means some other thread panicked while holding it. The
        // guarded value is a plain timestamp, so recovering it is safe and far
        // better than turning an unrelated panic into an outage here.
        let mut opened = self
            .opened_at
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        match *opened {
            Some(at) if at.elapsed() < self.config.circuit_cooldown => true,
            Some(_) => {
                *opened = None;
                false
            }
            None => false,
        }
    }

    fn on_success(&self) {
        self.failure_count.store(0, Ordering::Relaxed);
        *self
            .opened_at
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
    }

    fn on_failure(&self) {
        let failures = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        if failures >= self.config.circuit_open_after {
            *self
                .opened_at
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(Instant::now());
        }
    }

    /// Exponential backoff with jitter, capped at `max_delay`.
    ///
    /// The jitter spreads retries from concurrent callers so they do not all
    /// return to a struggling provider in the same instant. It is derived from
    /// the clock rather than a PRNG because nothing here depends on it being
    /// unpredictable -- only on it being uncorrelated between callers.
    fn delay(&self, attempt: u32) -> Duration {
        let ms = self.config.base_delay.as_millis() as f64 * 2f64.powi(attempt as i32);
        let jitter = ms * self.config.jitter_factor * clock_jitter();
        Duration::from_millis((ms + jitter) as u64).min(self.config.max_delay)
    }

    async fn with_retry<F, Fut, T>(&self, op: F) -> Result<T, DomainError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, DomainError>>,
    {
        if self.breaker_is_blocking() {
            return Err(DomainError::InternalError(
                "Blockchain provider is unavailable; the circuit breaker is open".to_string(),
            ));
        }

        let attempts = self.config.attempts();
        let mut remaining = attempts;
        loop {
            match op().await {
                Ok(value) => {
                    self.on_success();
                    return Ok(value);
                }
                Err(e) => {
                    self.on_failure();
                    remaining -= 1;
                    // Returning the error by value keeps the "no attempts were
                    // made" case unrepresentable: the previous version stored it
                    // in an Option and unwrapped it after the loop, which panicked
                    // outright when the loop body never ran.
                    if remaining == 0 {
                        return Err(e);
                    }
                    sleep(self.delay(attempts - remaining - 1)).await;
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl<G: BlockchainGateway> BlockchainGateway for RetryingBlockchainGateway<G> {
    fn receiving_address(&self, token: &str) -> Result<String, DomainError> {
        // Reading configuration cannot fail transiently, so it is not retried.
        self.inner.receiving_address(token)
    }

    async fn verify_transaction(
        &self,
        request: &VerifyRequest,
    ) -> Result<TxVerification, DomainError> {
        let inner = &self.inner;
        self.with_retry(|| async move { inner.verify_transaction(request).await })
            .await
    }
}

fn clock_jitter() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A gateway whose first `failures_before_success` calls fail.
    /// `u32::MAX` makes it fail forever.
    struct ScriptedGateway {
        calls: Arc<AtomicU32>,
        failures_before_success: u32,
    }

    impl ScriptedGateway {
        fn new(failures_before_success: u32) -> Self {
            Self {
                calls: Arc::new(AtomicU32::new(0)),
                failures_before_success,
            }
        }

        fn always_fails() -> Self {
            Self::new(u32::MAX)
        }

        fn call_count(&self) -> u32 {
            self.calls.load(Ordering::Relaxed)
        }
    }

    #[async_trait::async_trait]
    impl BlockchainGateway for ScriptedGateway {
        fn receiving_address(&self, _token: &str) -> Result<String, DomainError> {
            Ok("platform-wallet".to_string())
        }

        async fn verify_transaction(
            &self,
            _request: &VerifyRequest,
        ) -> Result<TxVerification, DomainError> {
            let n = self.calls.fetch_add(1, Ordering::Relaxed);
            if n < self.failures_before_success {
                return Err(DomainError::InternalError(format!("provider down ({})", n)));
            }
            Ok(TxVerification {
                verified: true,
                observed: true,
                base_units: 1,
                confirmations: 32,
                from_address: "buyer".to_string(),
                to_address: "platform-wallet".to_string(),
            })
        }
    }

    fn request() -> VerifyRequest {
        VerifyRequest {
            chain: "Solana".to_string(),
            token: "SOL".to_string(),
            tx_hash: "sig".to_string(),
            expected_to: "platform-wallet".to_string(),
            expected_base_units: 1,
            decimals: 9,
            token_contract: None,
        }
    }

    /// Fast enough that a test never actually waits out a backoff.
    fn fast(max_attempts: u32, circuit_open_after: u32) -> RetryConfig {
        RetryConfig {
            max_attempts,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(5),
            jitter_factor: 0.0,
            circuit_open_after,
            circuit_cooldown: Duration::from_millis(50),
        }
    }

    #[tokio::test]
    async fn a_call_that_succeeds_is_not_retried() {
        let gateway = RetryingBlockchainGateway::new(ScriptedGateway::new(0), fast(4, 5));
        assert!(gateway.verify_transaction(&request()).await.is_ok());
        assert_eq!(gateway.inner.call_count(), 1);
    }

    #[tokio::test]
    async fn a_transient_failure_is_retried_and_then_succeeds() {
        let gateway = RetryingBlockchainGateway::new(ScriptedGateway::new(2), fast(4, 5));
        assert!(gateway.verify_transaction(&request()).await.is_ok());
        assert_eq!(gateway.inner.call_count(), 3, "two failures then a success");
    }

    #[tokio::test]
    async fn an_exhausted_retry_reports_the_last_error() {
        let gateway = RetryingBlockchainGateway::new(ScriptedGateway::always_fails(), fast(3, 99));
        let err = gateway.verify_transaction(&request()).await.unwrap_err();
        assert!(format!("{:?}", err).contains("provider down"));
        assert_eq!(gateway.inner.call_count(), 3);
    }

    /// The regression test. `max_attempts: 0` used to leave the loop body unrun,
    /// so the error Option was still None and `.unwrap()` panicked -- taking down
    /// a worker thread inside payment verification.
    #[tokio::test]
    async fn a_zero_attempt_config_makes_one_call_instead_of_panicking() {
        let gateway = RetryingBlockchainGateway::new(ScriptedGateway::always_fails(), fast(0, 99));
        assert!(gateway.verify_transaction(&request()).await.is_err());
        assert_eq!(gateway.inner.call_count(), 1);
    }

    #[tokio::test]
    async fn the_breaker_opens_and_stops_calling_the_provider() {
        let gateway = RetryingBlockchainGateway::new(ScriptedGateway::always_fails(), fast(1, 2));

        assert!(gateway.verify_transaction(&request()).await.is_err());
        assert!(gateway.verify_transaction(&request()).await.is_err());
        assert_eq!(gateway.inner.call_count(), 2, "breaker trips on the second");

        let err = gateway.verify_transaction(&request()).await.unwrap_err();
        assert!(format!("{:?}", err).contains("circuit breaker is open"));
        assert_eq!(
            gateway.inner.call_count(),
            2,
            "a blocked call must not reach the provider"
        );
    }

    /// The defect that mattered more than the panic: the breaker was only ever
    /// reset on a success, and while it was open no call was made, so a success
    /// could never happen. Once tripped it stayed tripped until a restart.
    #[tokio::test]
    async fn the_breaker_recovers_once_the_provider_does() {
        // Fails twice -- enough to trip a threshold of 2 -- then is healthy.
        let gateway = RetryingBlockchainGateway::new(ScriptedGateway::new(2), fast(1, 2));

        assert!(gateway.verify_transaction(&request()).await.is_err());
        assert!(gateway.verify_transaction(&request()).await.is_err());
        assert!(
            gateway.verify_transaction(&request()).await.is_err(),
            "still inside the cooldown"
        );

        sleep(Duration::from_millis(70)).await;

        assert!(
            gateway.verify_transaction(&request()).await.is_ok(),
            "after the cooldown the breaker must let traffic through again"
        );
    }

    #[tokio::test]
    async fn a_failed_trial_call_re_opens_the_breaker() {
        let gateway = RetryingBlockchainGateway::new(ScriptedGateway::always_fails(), fast(1, 2));

        assert!(gateway.verify_transaction(&request()).await.is_err());
        assert!(gateway.verify_transaction(&request()).await.is_err());
        sleep(Duration::from_millis(70)).await;

        // The trial call reaches the provider and fails.
        assert!(gateway.verify_transaction(&request()).await.is_err());
        assert_eq!(gateway.inner.call_count(), 3);

        // Which must trip the breaker again rather than needing another full
        // run-up of consecutive failures.
        let err = gateway.verify_transaction(&request()).await.unwrap_err();
        assert!(format!("{:?}", err).contains("circuit breaker is open"));
        assert_eq!(gateway.inner.call_count(), 3);
    }

    #[tokio::test]
    async fn a_success_clears_the_failure_count() {
        // Threshold of 2: one failure, then a success, then one more failure
        // must NOT trip the breaker -- the two failures are not consecutive.
        let gateway = RetryingBlockchainGateway::new(ScriptedGateway::new(1), fast(1, 2));

        assert!(gateway.verify_transaction(&request()).await.is_err());
        assert!(gateway.verify_transaction(&request()).await.is_ok());
        assert_eq!(gateway.failure_count.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn a_zero_attempt_config_is_read_as_one_attempt() {
        assert_eq!(fast(0, 5).attempts(), 1);
        assert_eq!(fast(4, 5).attempts(), 4);
    }

    #[test]
    fn backoff_grows_and_then_stops_at_the_cap() {
        let config = RetryConfig {
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_millis(400),
            jitter_factor: 0.0,
            ..RetryConfig::default()
        };
        let gateway = RetryingBlockchainGateway::new(ScriptedGateway::new(0), config);

        assert_eq!(gateway.delay(0), Duration::from_millis(100));
        assert_eq!(gateway.delay(1), Duration::from_millis(200));
        assert_eq!(gateway.delay(2), Duration::from_millis(400));
        assert_eq!(gateway.delay(3), Duration::from_millis(400), "capped");
        assert_eq!(gateway.delay(60), Duration::from_millis(400), "no overflow");
    }
}
