use anyhow::{Result, anyhow};
use tracing_subscriber::EnvFilter;

use crate::core::config::LogFormat;

pub fn init(log_format: LogFormat) -> Result<()> {
    let env_filter =
        EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("aetherdb=info"))?;

    let builder = tracing_subscriber::fmt().with_env_filter(env_filter);

    match log_format {
        LogFormat::Text => builder
            .with_target(false)
            .compact()
            .try_init()
            .map_err(|error| anyhow!(error.to_string()))?,
        LogFormat::Json => builder
            .json()
            .with_target(false)
            .try_init()
            .map_err(|error| anyhow!(error.to_string()))?,
    }

    Ok(())
}
