use std::env;

/// Application configuration, loaded from environment variables at startup.
///
/// Zulip credentials are optional right now (the output adapter is not yet
/// implemented). The service starts and logs a warning for each missing
/// variable so the operator can see what needs to be configured before the
/// Zulip integration is enabled.
#[derive(Debug, Clone)]
pub struct Config {
    /// TCP port the HTTP server listens on. Defaults to `8080`.
    pub port: u16,
    /// Zulip bot e-mail address used for API authentication.
    pub zulip_email: String,
    /// Zulip bot API key.
    pub zulip_api_key: String,
    /// Zulip server hostname (e.g. `yourorg.zulipchat.com`).
    pub zulip_host: String,
}

impl Config {
    /// Build a [`Config`] from the process environment.
    pub fn from_env() -> Self {
        let port = env::var("PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8080);

        Config {
            port,
            zulip_email: env_warn("ZULIP_EMAIL"),
            zulip_api_key: env_warn("ZULIP_API_KEY"),
            zulip_host: env_warn("ZULIP_HOST"),
        }
    }

    /// Returns `true` when all three Zulip credentials are present and non-empty.
    pub fn zulip_configured(&self) -> bool {
        !self.zulip_email.is_empty()
            && !self.zulip_api_key.is_empty()
            && !self.zulip_host.is_empty()
    }
}

/// Read an environment variable, logging a warning if it is absent or empty.
fn env_warn(key: &str) -> String {
    match env::var(key) {
        Ok(v) if !v.is_empty() => v,
        _ => {
            tracing::warn!(
                "{} is not set — Zulip output will be unavailable until it is configured",
                key
            );
            String::new()
        }
    }
}
