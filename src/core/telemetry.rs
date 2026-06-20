use tracing_subscriber::{EnvFilter, fmt};

use crate::core::config::AppConfig;

pub fn init_telemetry(config: &AppConfig) -> anyhow::Result<()> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    if config.server.env.is_local() {
        fmt()
            .with_env_filter(env_filter)
            .with_target(true)
            .pretty()
            .init();
    } else {
        fmt()
            .with_env_filter(env_filter)
            .with_target(true)
            .json()
            .init();
    }

    Ok(())
}
