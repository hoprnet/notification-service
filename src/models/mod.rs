mod alert;
mod alert_with_incident;
mod incident;
mod reminder;

pub use alert::{Alert, EnrichedAlert};
pub use alert_with_incident::AlertWithIncident;
pub use incident::Incident;
pub use reminder::Reminder;

#[cfg(test)]
pub use alert::{AlertAnnotations, AlertLabels};
#[cfg(test)]
pub use reminder::{AlertSummary, IncidentSummary};

// ---------------------------------------------------------------------------
// Extraction helpers shared by `Alert::from_value` and `Incident::from_value`
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
