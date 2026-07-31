use serde::Serialize;

use super::{req_str, Alert};

// ---------------------------------------------------------------------------
// AlertWithIncident — an Alert payload already correlated to a Keep incident
// ---------------------------------------------------------------------------

/// An [`Alert`] payload that additionally links to a Keep incident.
///
/// Received at `POST /alert-with-incident` instead of `POST /alerts` once
/// Keep has correlated the alert with an incident. Rather than routing to
/// the namespace's `alerts-{environment}` topic with the full alert detail,
/// the message is posted directly into the incident's dedicated Zulip topic
/// with reduced content, since that topic already carries most of the context.
#[derive(Debug, Serialize, Clone)]
pub struct AlertWithIncident {
    pub alert: Alert,
    pub incident_id: String,
    pub incident_name: String,
}

impl AlertWithIncident {
    /// Validate and extract fields from an arbitrary JSON value: every field
    /// required by [`Alert::from_value`], plus `incident_id`/`incident_name`.
    ///
    /// # Errors
    /// Returns `Err(missing)` listing every required field that was absent or
    /// had an unexpected type, combining `Alert`'s missing fields with this
    /// struct's own.
    pub fn from_value(v: &serde_json::Value) -> Result<Self, Vec<String>> {
        let mut missing: Vec<String> = Vec::new();

        let alert = match Alert::from_value(v) {
            Ok(alert) => Some(alert),
            Err(alert_missing) => {
                missing.extend(alert_missing);
                None
            }
        };

        let incident_id = req_str(v, "/incident_id", &mut missing);
        let incident_name = req_str(v, "/incident_name", &mut missing);

        if !missing.is_empty() {
            return Err(missing);
        }

        Ok(AlertWithIncident {
            alert: alert.unwrap(),
            incident_id: incident_id.unwrap(),
            incident_name: incident_name.unwrap(),
        })
    }
}
