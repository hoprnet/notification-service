use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Incoming alert payload.
///
/// This is a generic format that will eventually map to Keep "Alert" and
/// Keep "Incident" payloads. Fields are kept intentionally minimal for now.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Alert {
    /// Unique identifier for this alert.
    pub id: String,
    /// Human-readable alert name.
    pub name: String,
    /// Severity level (e.g. `"critical"`, `"warning"`, `"info"`).
    pub severity: String,
    /// Detailed description of the alert condition.
    pub message: String,
    /// System or integration that originated the alert.
    pub source: String,
    /// Arbitrary key-value metadata (environment, service, region, …).
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

/// An alert after enrichment by the processing layer.
///
/// Flattened into the same JSON object as `Alert` for a clean wire format.
#[derive(Debug, Serialize)]
pub struct EnrichedAlert {
    #[serde(flatten)]
    pub alert: Alert,
    /// UTC timestamp of when this service processed the alert.
    pub processed_at: DateTime<Utc>,
    /// Always `true` — signals that enrichment has been applied.
    pub enriched: bool,
    /// Name of the service that performed the enrichment.
    pub service_name: String,
}
