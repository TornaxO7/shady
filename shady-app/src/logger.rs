use tracing::debug;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .without_time();

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(EnvFilter::from_env(EnvFilter::DEFAULT_ENV))
        .init();

    debug!("Logger initialised!");
}
