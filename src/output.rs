use crate::{config::Config, models::EnrichedAlert};

/// Send an enriched alert to the configured output.
///
/// **Current implementation:** pretty-prints the alert as JSON to stdout
/// via the `tracing` log stream at INFO level.
///
/// **Planned:** when `config.zulip_configured()` returns `true`, the enriched
/// alert will be formatted and posted to a Zulip stream/topic using
/// `config.zulip_host`, `config.zulip_email`, and `config.zulip_api_key`.
pub fn send(alert: &EnrichedAlert, config: &Config) {
    if config.zulip_configured() {
        // TODO: implement Zulip message dispatch
        tracing::debug!("Zulip output is configured but not yet implemented");
    }

    match serde_json::to_string_pretty(alert) {
        Ok(json) => tracing::info!("Processed alert:\n{}", json),
        Err(e) => tracing::error!("Failed to serialize enriched alert: {}", e),
    }
}
