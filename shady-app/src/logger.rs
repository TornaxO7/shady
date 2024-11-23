use tracing::debug;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

const ENV_LOG_NAME: &str = "SHADY_LOG";

pub fn init() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .without_time();

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(EnvFilter::from_env(ENV_LOG_NAME))
        .init();

    debug!("Logger initialised!");
}
