use serde::Serialize;

use super::{opt_str, req_str};

// ---------------------------------------------------------------------------
// Incident — validated extraction from a Keep incident payload
// ---------------------------------------------------------------------------

/// Incident fields validated and extracted from an incoming Keep incident payload.
#[derive(Debug, Serialize, Clone)]
pub struct Incident {
    pub id: String,
    /// Used as the Zulip topic name.
    pub topic_name: String,
    pub description: Option<String>,
    pub assignee: Option<String>,
    pub severity: Option<String>,
    pub alerts_count: Option<u64>,
    /// Linear issue identifier (e.g. `CORE-123`).
    pub linear_id: Option<String>,
    /// Linear issue URL.
    pub linear_url: Option<String>,
    /// Kubernetes namespace — used to route to the correct Zulip stream.
    pub namespace: Option<String>,
    pub status: Option<String>,
}

impl Incident {
    /// Validate and extract well-known fields from an arbitrary JSON value.
    ///
    /// # Errors
    /// Returns `Err(missing)` listing every required field that was absent or
    /// had an unexpected type.
    pub fn from_value(v: &serde_json::Value) -> Result<Self, Vec<String>> {
        let mut missing: Vec<String> = Vec::new();

        let id = req_str(v, "/id", &mut missing);
        let topic_name = req_str(v, "/topic_name", &mut missing);

        if !missing.is_empty() {
            return Err(missing);
        }

        Ok(Incident {
            id: id.unwrap(),
            topic_name: topic_name.unwrap(),
            description: opt_str(v, "/description"),
            assignee: opt_str(v, "/assignee"),
            severity: opt_str(v, "/severity"),
            alerts_count: v.pointer("/alerts_count").and_then(|val| {
                val.as_u64().or_else(|| val.as_str().and_then(|s| s.parse().ok()))
            }),
            linear_id: opt_str(v, "/linear_id"),
            linear_url: opt_str(v, "/linear_url"),
            namespace: opt_str(v, "/namespace"),
            status: opt_str(v, "/status"),
        })
    }
}
