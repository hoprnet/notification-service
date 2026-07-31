use chrono::{DateTime, Utc};
use serde::Serialize;

use super::{nullable_str, opt_str, req_str, req_u64};

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
    /// `labels.app_kubernetes_io_name`
    pub app_name: Option<String>,
    /// `labels.app_kubernetes_io_instance`
    pub instance: Option<String>,
    /// `labels.infrastructure`
    pub infrastructure: Option<String>,
    /// `labels.network`
    pub network: Option<String>,
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
    /// Raw timestamp string from `startedAt`. Optional; falls back to `firing_start_time` or system time.
    pub started_at: Option<String>,
    /// Raw timestamp string from `firingStartTime` in the payload.
    pub firing_start_time: Option<String>,
    /// Raw timestamp string from `lastReceived`.
    pub last_received: String,
    /// Number of times the alert has fired (`firingCounter`).
    pub firing_counter: u64,
    pub fingerprint: String,
    /// `null` when unassigned; the key must be present in the payload.
    pub assignee: Option<String>,
    /// Raw timestamp string from `endsAt`.
    pub ends_at: Option<String>,
    /// Prometheus generator URL (`generatorURL`).
    pub generator_url: Option<String>,

    // ── Optional ─────────────────────────────────────────────────────────────
    pub description: Option<String>,
    /// Free-text root cause analysis summary (`rca_summary`).
    /// Rendered as a dedicated section in the Zulip message when present.
    pub rca_summary: Option<String>,
    /// Name of the parent alert this alert is correlated with (`correlated_parent_alert`).
    /// Rendered as a linked bullet when both this and `correlated_parent_fingerprint` are present.
    pub correlated_parent_alert: Option<String>,
    /// Fingerprint of the parent alert (`correlated_parent_fingerprint`).
    /// Used together with `KEEP_BASE_URL` to build the deep-link.
    pub correlated_parent_fingerprint: Option<String>,
    /// Kubernetes namespace the alert originated from.
    /// Used to route the notification to the correct Zulip stream.
    pub namespace: Option<String>,
    pub labels: AlertLabels,
    pub annotations: AlertAnnotations,
}

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
        let started_at = opt_str(v, "/startedAt");
        let firing_start_time = opt_str(v, "/firingStartTime");
        let last_received = req_str(v, "/lastReceived", &mut missing);
        let fingerprint = req_str(v, "/fingerprint", &mut missing);
        let ends_at = opt_str(v, "/endsAt");
        let generator_url = opt_str(v, "/generatorURL");
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
            started_at,
            firing_start_time,
            last_received: last_received.unwrap(),
            firing_counter: firing_counter.unwrap(),
            fingerprint: fingerprint.unwrap(),
            assignee,
            ends_at,
            generator_url,
            description: opt_str(v, "/description"),
            rca_summary: opt_str(v, "/rca_summary"),
            correlated_parent_alert: opt_str(v, "/correlated_parent_alert"),
            correlated_parent_fingerprint: opt_str(v, "/correlated_parent_fingerprint"),
            namespace: opt_str(v, "/namespace"),
            labels: AlertLabels {
                pod: opt_str(v, "/labels/pod"),
                reason: opt_str(v, "/labels/reason"),
                container: opt_str(v, "/labels/container"),
                app_name: opt_str(v, "/labels/app_kubernetes_io_name"),
                instance: opt_str(v, "/labels/app_kubernetes_io_instance"),
                infrastructure: opt_str(v, "/labels/infrastructure"),
                network: opt_str(v, "/labels/network"),
            },
            annotations: AlertAnnotations {
                summary: opt_str(v, "/annotations/summary"),
                runbook_url: opt_str(v, "/annotations/runbook_url"),
            },
        })
    }

    /// Resolves the effective start time using a fallback chain:
    /// 1. `started_at` from the payload
    /// 2. `firing_start_time` from the payload
    /// 3. current system time (UTC)
    pub fn effective_started_at(&self) -> String {
        self.started_at
            .clone()
            .or_else(|| self.firing_start_time.clone())
            .unwrap_or_else(|| Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
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
