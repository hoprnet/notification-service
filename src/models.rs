use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Alert — extracted from any arbitrary JSON payload
// ---------------------------------------------------------------------------

/// Well-known fields extracted from an incoming alert payload.
///
/// The service accepts **any** JSON object.  Only these fields are extracted;
/// everything else in the payload is silently ignored.
#[derive(Debug, Serialize, Clone)]
pub struct Alert {
    pub id: Option<String>,
    pub name: Option<String>,
    pub status: Option<String>,
    pub severity: Option<String>,
    /// Short human-readable message (may be absent or null).
    pub message: Option<String>,
    /// Longer description — preferred over `message` when formatting output.
    pub description: Option<String>,
    /// Originating system(s).  Keep sends an array; simple clients a string.
    pub source: Vec<String>,
    /// Key-value metadata tags.
    pub labels: HashMap<String, String>,
}

impl Alert {
    /// Extract well-known fields from an arbitrary `serde_json::Value`.
    ///
    /// Fields that are absent, null, or of an unexpected type are skipped
    /// without returning an error.
    pub fn from_value(v: &serde_json::Value) -> Self {
        Alert {
            id: v["id"].as_str().map(str::to_string),
            name: v["name"].as_str().map(str::to_string),
            status: v["status"].as_str().map(str::to_string),
            severity: v["severity"].as_str().map(str::to_string),
            message: v["message"].as_str().map(str::to_string),
            description: v["description"].as_str().map(str::to_string),
            source: extract_source(&v["source"]),
            labels: extract_labels(&v["labels"]),
        }
    }

    /// Return the most descriptive text available:
    /// `description` → `message` → `"(no description)"`.
    pub fn summary(&self) -> &str {
        self.description
            .as_deref()
            .or(self.message.as_deref())
            .unwrap_or("(no description)")
    }
}

/// Handle `source` as either a plain string or an array of strings.
fn extract_source(v: &serde_json::Value) -> Vec<String> {
    match v {
        serde_json::Value::String(s) => vec![s.clone()],
        serde_json::Value::Array(arr) => arr
            .iter()
            .filter_map(|x| x.as_str().map(str::to_string))
            .collect(),
        _ => vec![],
    }
}

/// Extract string-valued entries from a JSON object; skip non-string values.
fn extract_labels(v: &serde_json::Value) -> HashMap<String, String> {
    match v.as_object() {
        Some(obj) => obj
            .iter()
            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
            .collect(),
        None => HashMap::new(),
    }
}

// ---------------------------------------------------------------------------
// EnrichedAlert — after processing
// ---------------------------------------------------------------------------

/// An alert after enrichment by the processing layer.
#[derive(Debug, Serialize)]
pub struct EnrichedAlert {
    pub alert: Alert,
    /// UTC timestamp of when this service processed the alert.
    pub processed_at: DateTime<Utc>,
    /// Always `true` — signals that enrichment has been applied.
    pub enriched: bool,
    /// Name of the service that performed the enrichment.
    pub service_name: String,
}
