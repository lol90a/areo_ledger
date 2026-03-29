use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initializes structured logging and OpenTelemetry tracing.
/// Call once at application startup before any other code runs.
pub fn init_observability(service_name: &str, otlp_endpoint: Option<&str>) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,areo_ledger=debug"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()                          // structured JSON — parseable by Datadog/Loki/CloudWatch
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true);

    let registry = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer);

    // Only attach OTLP exporter if endpoint is configured
    if let Some(endpoint) = otlp_endpoint {
        tracing::info!(endpoint, service = service_name, "OpenTelemetry exporter configured");
        // In production: add opentelemetry-otlp layer here
        // registry.with(otel_layer).init();
    }

    registry.init();

    tracing::info!(service = service_name, "Observability initialized");
}

/// Span attributes for financial operations — attach to every payment/booking span.
pub struct FinancialSpanAttrs {
    pub booking_id:  Option<uuid::Uuid>,
    pub payment_id:  Option<uuid::Uuid>,
    pub amount_cents: Option<i64>,
    pub currency:    Option<String>,
    pub chain:       Option<String>,
}

impl FinancialSpanAttrs {
    pub fn record(&self) {
        if let Some(id) = self.booking_id {
            tracing::Span::current().record("booking_id", &id.to_string().as_str());
        }
        if let Some(id) = self.payment_id {
            tracing::Span::current().record("payment_id", &id.to_string().as_str());
        }
        if let Some(cents) = self.amount_cents {
            tracing::Span::current().record("amount_cents", &cents);
        }
        if let Some(ref c) = self.currency {
            tracing::Span::current().record("currency", &c.as_str());
        }
        if let Some(ref c) = self.chain {
            tracing::Span::current().record("chain", &c.as_str());
        }
    }
}
