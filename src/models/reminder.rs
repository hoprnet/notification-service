use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Reminder — daily digest of currently open alerts/incidents
// ---------------------------------------------------------------------------
//
// Unlike `Alert`/`Incident`, this payload is produced internally (by a K8s
// CronJob that queries the Keep database) rather than by an arbitrary
// external webhook, so it is deserialized directly instead of going through
// the `from_value` + missing-field-list validation used for the other two.

/// A single open alert, as summarised for the daily reminder digest.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AlertSummary {
    pub alertname: String,
    pub namespace: Option<String>,
    pub application: Option<String>,
    #[serde(rename = "startTime")]
    pub start_time: String,
    pub occurrences: u64,
    pub fingerprint: String,
}

/// A single open incident, as summarised for the daily reminder digest.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IncidentSummary {
    pub name: String,
    pub assignee: Option<String>,
    #[serde(rename = "alerts_count")]
    pub alerts_count: u64,
    pub incident_id: String,
    /// When the incident was opened. Used to compute the "Days opened" column.
    #[serde(rename = "startTime")]
    pub start_time: String,
}

/// Daily digest of every currently open alert and incident, posted to Zulip
/// as a single Markdown table.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reminder {
    #[serde(default)]
    pub alerts: Vec<AlertSummary>,
    #[serde(default)]
    pub incidents: Vec<IncidentSummary>,
    /// Overrides `ZULIP_REMINDER_STREAM` for this message when present.
    #[serde(default)]
    pub stream: Option<String>,
    /// Overrides `ZULIP_REMINDER_TOPIC` for this message when present.
    #[serde(default)]
    pub topic: Option<String>,
    /// Logical environment this digest was built from (e.g. `staging`,
    /// `production`). Shown in the message header so a reminder is
    /// self-identifying regardless of which stream/topic it lands in.
    /// Overrides `ENVIRONMENT_NAME` for this message when present; otherwise
    /// falls back to the service's configured environment.
    #[serde(default)]
    pub environment: Option<String>,
}
