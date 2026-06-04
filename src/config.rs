use std::{collections::HashMap, env};

/// Application configuration, loaded from environment variables at startup.
#[derive(Debug, Clone)]
pub struct Config {
    /// TCP port the HTTP server listens on. Defaults to `8080`.
    pub port: u16,

    /// Keep base URL used to build per-alert deep-links.
    ///
    /// Example: `https://incidents.staging.hoprnet.link`
    /// Full alert URL: `{keep_base_url}/alerts/feed?alertPayloadFingerprint={fingerprint}`
    pub keep_base_url: String,

    /// Graylog base URL including the saved-search path and `?q=` suffix.
    ///
    /// Example: `https://graylog.staging.hoprnet.link/search/<id>?q=`
    /// The query, date-range, and timestamp parameters are appended at render time.
    /// Leave empty to omit the Graylog link from messages.
    pub graylog_base_url: String,

    /// Logical environment name (e.g. `staging`, `production`).
    /// Used to build the Zulip topic: `alerts-{environment_name}`.
    pub environment_name: String,

    // ── Zulip ────────────────────────────────────────────────────────────────
    /// Zulip bot e-mail address used for API authentication.
    pub zulip_email: String,
    /// Zulip bot API key.
    pub zulip_api_key: String,
    /// Zulip server hostname (e.g. `yourorg.zulipchat.com`).
    pub zulip_host: String,
    /// Zulip stream to use when the alert namespace has no explicit mapping.
    pub zulip_default_stream: String,
    /// Namespace → Zulip stream name map.
    /// Populated from the `ZULIP_NAMESPACE_STREAMS` env var (JSON object).
    ///
    /// Example: `{"monitoring": "devops-alerts", "production": "prod-alerts"}`
    pub zulip_namespace_streams: HashMap<String, String>,

    /// When `false`, Zulip API calls are skipped and replaced with an info-level
    /// log message that shows what *would* have been sent.  Defaults to `true`.
    /// Set `ZULIP_ENABLED=false` to run in dry-run / silent mode.
    pub zulip_enabled: bool,

    /// How many days a fingerprint→message-ID mapping is kept in the in-memory
    /// store before the eviction task removes it.  Defaults to `7`.
    /// Controlled by `MESSAGE_TTL_DAYS`.
    pub message_ttl_days: u64,

    /// Timeout in seconds applied to every outbound Zulip HTTP request.
    /// Defaults to `10`.  Controlled by `ZULIP_REQUEST_TIMEOUT_SECS`.
    pub zulip_request_timeout_secs: u64,
}

impl Config {
    /// Build a [`Config`] from the process environment.
    pub fn from_env() -> Self {
        let port = env::var("PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8080);

        let zulip_namespace_streams =
            parse_namespace_streams(&env::var("ZULIP_NAMESPACE_STREAMS").unwrap_or_default());

        Config {
            port,
            keep_base_url: env_warn("KEEP_BASE_URL"),
            graylog_base_url: env_opt("GRAYLOG_BASE_URL"),
            environment_name: env_warn("ENVIRONMENT_NAME"),
            zulip_email: env_warn("ZULIP_EMAIL"),
            zulip_api_key: env_warn("ZULIP_API_KEY"),
            zulip_host: env_warn("ZULIP_HOST"),
            zulip_default_stream: env_warn("ZULIP_DEFAULT_STREAM"),
            zulip_namespace_streams,
            zulip_enabled: env::var("ZULIP_ENABLED")
                .map(|v| v.trim().to_lowercase() != "false")
                .unwrap_or(true),
            message_ttl_days: env::var("MESSAGE_TTL_DAYS")
                .ok()
                .and_then(|v| v.trim().parse().ok())
                .unwrap_or(7),
            zulip_request_timeout_secs: env::var("ZULIP_REQUEST_TIMEOUT_SECS")
                .ok()
                .and_then(|v| v.trim().parse().ok())
                .unwrap_or(10),
        }
    }

    /// Returns `true` when all Zulip credentials and routing config are present,
    /// regardless of whether `ZULIP_ENABLED` is set.
    ///
    /// Use this to guard real API calls; check `zulip_enabled` separately to
    /// decide whether to mock or actually send.
    pub fn zulip_configured(&self) -> bool {
        !self.zulip_email.is_empty()
            && !self.zulip_api_key.is_empty()
            && !self.zulip_host.is_empty()
            && !self.zulip_default_stream.is_empty()
    }

    /// Build the Keep deep-link URL for a given alert fingerprint.
    ///
    /// Returns `None` when `KEEP_BASE_URL` is not configured.
    pub fn keep_alert_url(&self, fingerprint: &str) -> Option<String> {
        if self.keep_base_url.is_empty() {
            return None;
        }
        Some(format!(
            "{}/alerts/feed?alertPayloadFingerprint={}",
            self.keep_base_url.trim_end_matches('/'),
            fingerprint,
        ))
    }

    /// Returns `None` when `KEEP_BASE_URL` is not configured.
    pub fn keep_incident_url(&self, incident_id: &str) -> Option<String> {
        if self.keep_base_url.is_empty() {
            return None;
        }
        Some(format!(
            "{}/incidents/{}/alerts",
            self.keep_base_url.trim_end_matches('/'),
            incident_id,
        ))
    }

    /// Resolve the Zulip stream name for a given Kubernetes namespace.
    ///
    /// Looks up `namespace` in `zulip_namespace_streams`; falls back to
    /// `zulip_default_stream` when no explicit mapping is found.
    pub fn stream_for_namespace(&self, namespace: Option<&str>) -> &str {
        if let Some(ns) = namespace {
            if let Some(stream) = self.zulip_namespace_streams.get(ns) {
                return stream.as_str();
            }
        }
        &self.zulip_default_stream
    }

    /// Build the Zulip topic for this environment: `alerts-{environment_name}`.
    pub fn zulip_topic(&self) -> String {
        format!("alerts-{}", self.environment_name)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Read an environment variable, trimming whitespace and logging a warning
/// if the result is absent or empty after trimming.
fn env_warn(key: &str) -> String {
    match env::var(key) {
        Ok(v) => {
            let v = v.trim().to_string();
            if !v.is_empty() {
                v
            } else {
                tracing::warn!("{} is not set", key);
                String::new()
            }
        }
        _ => {
            tracing::warn!("{} is not set", key);
            String::new()
        }
    }
}

/// Read an optional environment variable without logging a warning when absent.
fn env_opt(key: &str) -> String {
    env::var(key)
        .map(|v| v.trim().to_string())
        .unwrap_or_default()
}

/// Parse `ZULIP_NAMESPACE_STREAMS` from a JSON object string.
///
/// Invalid JSON or missing variable → empty map (logs a warning).
fn parse_namespace_streams(raw: &str) -> HashMap<String, String> {
    if raw.is_empty() {
        return HashMap::new();
    }
    match serde_json::from_str::<HashMap<String, String>>(raw) {
        Ok(map) => map,
        Err(e) => {
            tracing::warn!(
                "ZULIP_NAMESPACE_STREAMS contains invalid JSON — namespace routing \
                 will use the default stream: {}",
                e
            );
            HashMap::new()
        }
    }
}
