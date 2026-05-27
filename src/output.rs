use crate::{config::Config, formatter, models::EnrichedAlert};

/// Send an enriched alert to the configured output.
///
/// **Current implementation:** renders the alert as Zulip Markdown and logs
/// it at INFO level, so the formatted message is visible in the service logs.
///
/// **Planned:** when `config.zulip_configured()` returns `true`, the rendered
/// message will be posted to a Zulip stream/topic via the REST API using
/// `config.zulip_host`, `config.zulip_email`, and `config.zulip_api_key`.
pub fn send(alert: &EnrichedAlert, config: &Config) {
    let markdown = formatter::to_zulip(&alert.alert);

    if config.zulip_configured() {
        // TODO: POST markdown to Zulip stream via reqwest
        tracing::debug!("Zulip output is configured but not yet implemented");
    }

    tracing::info!("Zulip message preview:\n{}", markdown);
}
