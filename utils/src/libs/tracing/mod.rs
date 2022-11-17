use std::default::Default;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub async fn default_shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::warn!("signal received, starting graceful shutdown");
    opentelemetry::global::shutdown_tracer_provider();
}

pub struct TraceInit {
    t: Box<dyn Fn()>,
}
impl TraceInit {
    pub fn new(f: impl Fn() + 'static) -> Self {
        Self { t: Box::new(f) }
    }
    pub fn init(&self) {
        (&self.t)()
    }
}
impl Default for TraceInit {
    fn default() -> Self {
        Self {
            t: Box::new(|| {
                tracing_subscriber::registry()
                    .with(tracing_subscriber::EnvFilter::new(
                        std::env::var("RUST_LOG").unwrap_or_else(|_| {
                            "example_tracing_aka_logging=debug,tower_http=debug".into()
                        }),
                    ))
                    .with(tracing_subscriber::fmt::layer())
                    .init();
            }),
        }
    }
}
