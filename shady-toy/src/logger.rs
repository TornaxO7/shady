use tracing::debug;
use tracing_subscriber::EnvFilter;

pub fn init() {
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .pretty()
        .init();

    debug!("Logger initialised!");
}
