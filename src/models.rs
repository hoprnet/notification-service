use chrono::{DateTime, Utc};
use serde::Serialize;

// ---------------------------------------------------------------------------
// Sub-structs
// ---------------------------------------------------------------------------

/// Fields extracted from the `labels` object.  All are optional — present
/// fields are shown in the Zulip message; absent ones are silently skipped.
#[derive(Debug, Serialize, Clone)]
pub struct AlertLabels {
    pub pod: Option<String>,
    pub reason: Option<String>,
    pub container: Option<String>,
    /// `labels.label_app_kubernetes_io_name`
    pub app_name: Option<String>,
}

/// Fields extracted from the `annotations` object.
#[derive(Debug, Serialize, Clone)]
pub struct AlertAnnotations {
    pub summary: Option<String>,
    pub runbook_url: Option<String>,
}

// ---------------------------------------------------------------------------
// Alert — validated extraction from an arbitrary JSON payload
// ---------------------------------------------------------------------------

/// Alert fields validated and extracted from an incoming JSON payload.
///
/// The service accepts **any** JSON object at `POST /alerts`.
/// `Alert::from_value` extracts the fields below and returns an error listing
/// every required field that is absent or has an unexpected type.
///
/// Fields not listed here are silently ignored.
#[derive(Debug, Serialize, Clone)]
pub struct Alert {
    // ── Required ─────────────────────────────────────────────────────────────
    pub id: String,
    pub name: String,
    pub status: String,
    pub severity: String,
    /// Raw timestamp string from `startedAt`.
    pub started_at: String,
    /// Raw timestamp string from `lastReceived`.
    pub last_received: String,
    /// Number of times the alert has fired (`firingCounter`).
    pub firing_counter: u64,
    pub fingerprint: String,
    /// `null` when unassigned; the key must be present in the payload.
    pub assignee: Option<String>,
    /// Raw timestamp string from `endsAt`.
    pub ends_at: String,
    /// Prometheus generator URL (`generatorURL`).
    pub generator_url: String,

    // ── Optional ─────────────────────────────────────────────────────────────
    pub description: Option<String>,
    /// Free-text root cause analysis summary (`rca_summary`).
    /// Rendered as a dedicated section in the Zulip message when present.
    pub rca_summary: Option<String>,
    /// Kubernetes namespace the alert originated from.
    /// Used to route the notification to the correct Zulip stream.
    pub namespace: Option<String>,
    pub labels: AlertLabels,
    pub annotations: AlertAnnotations,
}

// ---------------------------------------------------------------------------
// Extraction helpers
// ---------------------------------------------------------------------------

/// Extract a required non-null string; record the JSON-pointer path on failure.
fn req_str(v: &serde_json::Value, path: &str, missing: &mut Vec<String>) -> Option<String> {
    match v.pointer(path) {
        Some(serde_json::Value::String(s)) => Some(s.clone()),
        _ => {
            missing.push(path.to_string());
            None
        }
    }
}

/// Extract a required non-null unsigned integer; record the path on failure.
fn req_u64(v: &serde_json::Value, path: &str, missing: &mut Vec<String>) -> Option<u64> {
    match v.pointer(path).and_then(|val| val.as_u64()) {
        Some(n) => Some(n),
        None => {
            missing.push(path.to_string());
            None
        }
    }
}

/// Extract an optional string — absent or non-string → `None`, no error.
fn opt_str(v: &serde_json::Value, path: &str) -> Option<String> {
    v.pointer(path)
        .and_then(|val| val.as_str())
        .map(str::to_string)
}

/// Extract a required-but-nullable string:
/// - Key present, value is a string  → `Some(String)` (no error)
/// - Key present, value is JSON null  → `None`         (no error)
/// - Key absent or unexpected type    → records path in `missing`, returns `None`
///
/// Use this for fields that must be present in every payload but may carry a
/// `null` value (e.g. `assignee` when unassigned).
fn nullable_str(v: &serde_json::Value, path: &str, missing: &mut Vec<String>) -> Option<String> {
    match v.pointer(path) {
        Some(serde_json::Value::String(s)) => Some(s.clone()),
        Some(serde_json::Value::Null) => None,
        _ => {
            missing.push(path.to_string());
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Alert::from_value
// ---------------------------------------------------------------------------

impl Alert {
    /// Validate and extract well-known fields from an arbitrary JSON value.
    ///
    /// # Errors
    /// Returns `Err(missing)` where `missing` is the list of JSON-pointer paths
    /// for every required field that was absent or had an unexpected type.
    pub fn from_value(v: &serde_json::Value) -> Result<Self, Vec<String>> {
        let mut missing: Vec<String> = Vec::new();

        // ── Required fields ──────────────────────────────────────────────────
        let id = req_str(v, "/id", &mut missing);
        let name = req_str(v, "/name", &mut missing);
        let status = req_str(v, "/status", &mut missing);
        let severity = req_str(v, "/severity", &mut missing);
        let started_at = req_str(v, "/startedAt", &mut missing);
        let last_received = req_str(v, "/lastReceived", &mut missing);
        let fingerprint = req_str(v, "/fingerprint", &mut missing);
        let ends_at = req_str(v, "/endsAt", &mut missing);
        let generator_url = req_str(v, "/generatorURL", &mut missing);
        let firing_counter = req_u64(v, "/firingCounter", &mut missing);

        // `assignee` must be present as a key but its value may be null.
        // nullable_str distinguishes absent key (→ error) from explicit null (→ None).
        let assignee = nullable_str(v, "/assignee", &mut missing);

        if !missing.is_empty() {
            return Err(missing);
        }

        // ── Optional fields ──────────────────────────────────────────────────
        Ok(Alert {
            id: id.unwrap(),
            name: name.unwrap(),
            status: status.unwrap(),
            severity: severity.unwrap(),
            started_at: started_at.unwrap(),
            last_received: last_received.unwrap(),
            firing_counter: firing_counter.unwrap(),
            fingerprint: fingerprint.unwrap(),
            assignee,
            ends_at: ends_at.unwrap(),
            generator_url: generator_url.unwrap(),
            description: opt_str(v, "/description"),
            rca_summary: opt_str(v, "/rca_summary"),
            namespace: opt_str(v, "/namespace"),
            labels: AlertLabels {
                pod: opt_str(v, "/labels/pod"),
                reason: opt_str(v, "/labels/reason"),
                container: opt_str(v, "/labels/container"),
                app_name: opt_str(v, "/labels/label_app_kubernetes_io_name"),
            },
            annotations: AlertAnnotations {
                summary: opt_str(v, "/annotations/summary"),
                runbook_url: opt_str(v, "/annotations/runbook_url"),
            },
        })
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
