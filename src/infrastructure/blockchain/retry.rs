use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::time::sleep;

use crate::domain::ports::blockchain_gateway::{BlockchainGateway, TxVerification};
use crate::shared::errors::DomainError;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub jitter_factor: f64,
    pub circuit_open_after: u32,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 4,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            jitter_factor: 0.2,
            circuit_open_after: 5,
        }
    }
}

pub struct RetryingBlockchainGateway<G: BlockchainGateway> {
    inner: G,
    config: RetryConfig,
    failure_count: Arc<AtomicU32>,
    circuit_open: Arc<AtomicBool>,
}

impl<G: BlockchainGateway> RetryingBlockchainGateway<G> {
    pub fn new(inner: G, config: RetryConfig) -> Self {
        Self {
            inner,
            config,
            failure_count: Arc::new(AtomicU32::new(0)),
            circuit_open: Arc::new(AtomicBool::new(false)),
        }
    }

    fn is_open(&self) -> bool {
        self.circuit_open.load(Ordering::Relaxed)
    }

    fn on_success(&self) {
        self.failure_count.store(0, Ordering::Relaxed);
        self.circuit_open.store(false, Ordering::Relaxed);
    }

    fn on_failure(&self) {
        let n = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        if n >= self.config.circuit_open_after {
            self.circuit_open.store(true, Ordering::Relaxed);
        }
    }

    fn delay(&self, attempt: u32) -> Duration {
        let ms = self.config.base_delay.as_millis() as f64 * 2f64.powi(attempt as i32);
        let jitter = ms * self.config.jitter_factor * rand_jitter();
        Duration::from_millis((ms + jitter) as u64).min(self.config.max_delay)
    }

    async fn with_retry<F, Fut, T>(&self, op: F) -> Result<T, DomainError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, DomainError>>,
    {
        if self.is_open() {
            return Err(DomainError::InternalError(
                "Circuit breaker open".to_string(),
            ));
        }
        let mut last = None;
        for attempt in 0..self.config.max_attempts {
            match op().await {
                Ok(v) => {
                    self.on_success();
                    return Ok(v);
                }
                Err(e) => {
                    self.on_failure();
                    if attempt + 1 < self.config.max_attempts {
                        sleep(self.delay(attempt)).await;
                    }
                    last = Some(e);
                }
            }
        }
        Err(last.unwrap())
    }
}

#[async_trait::async_trait]
impl<G: BlockchainGateway> BlockchainGateway for RetryingBlockchainGateway<G> {
    fn receiving_address(&self, token: &str) -> Result<String, DomainError> {
        self.inner.receiving_address(token)
    }

    async fn verify_transaction(
        &self,
        chain: &str,
        tx_hash: &str,
        expected_to: &str,
        expected_amount_cents: i64,
    ) -> Result<TxVerification, DomainError> {
        let inner = &self.inner;
        self.with_retry(|| async move {
            inner
                .verify_transaction(chain, tx_hash, expected_to, expected_amount_cents)
                .await
        })
        .await
    }
}

fn rand_jitter() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (n % 1000) as f64 / 1000.0
}
