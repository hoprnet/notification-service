use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::{models::{Alert, Incident, Reminder}, output, processing, AppState};

/// `POST /incident` — receive a Keep incident payload, format it, and post a
/// new Zulip topic.
///
/// Each call always creates a new topic — there is no update/deduplication.
///
/// # Responses
/// - `200 OK` — incident accepted and dispatched.
/// - `422 Unprocessable Entity` — JSON is valid but required fields are absent.
/// - `400 Bad Request` — body is not valid JSON.
pub async fn receive_incident(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    tracing::debug!(payload = %payload, "Incoming incident payload");
    let incident = match Incident::from_value(&payload) {
        Ok(i) => i,
        Err(missing) => {
            tracing::warn!(
                fields  = ?missing,
                payload = %payload,
                "Rejected incident: missing required fields"
            );
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({
                    "error": "missing required fields",
                    "fields": missing,
                })),
            )
                .into_response();
        }
    };

    tracing::info!(
        id     = %incident.id,
        name   = %incident.topic_name,
        status = ?incident.status,
        "Received incident"
    );

    output::send_incident(&incident, &state.config).await;

    (
        StatusCode::OK,
        Json(json!({
            "status": "processed",
            "id": incident.id,
        })),
    )
        .into_response()
}

/// `POST /alerts` — receive any JSON alert payload, validate required fields,
/// enrich the alert, and dispatch it.
///
/// # Request
/// Any valid JSON object.  Required fields are validated after parsing;
/// unknown fields are silently ignored.
///
/// # Responses
/// - `200 OK` — alert accepted and processed.
/// - `422 Unprocessable Entity` — JSON is valid but one or more required
///   fields are absent or have an unexpected type.  The response body lists
///   every missing field path.
pub async fn receive_alert(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    tracing::debug!(payload = %payload, "Incoming alert payload");
    let alert = match Alert::from_value(&payload) {
        Ok(a) => a,
        Err(missing) => {
            tracing::warn!(
                fields  = ?missing,
                payload = %payload,
                "Rejected alert: missing required fields"
            );
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({
                    "error": "missing required fields",
                    "fields": missing,
                })),
            )
                .into_response();
        }
    };

    tracing::info!(
        id        = %alert.id,
        name      = %alert.name,
        status    = %alert.status,
        severity  = %alert.severity,
        namespace = ?alert.namespace,
        "Received alert"
    );

    let enriched = processing::enrich(alert);
    output::send(&enriched, &state.config, &state.messages).await;

    (
        StatusCode::OK,
        Json(json!({
            "status": "processed",
            "id": enriched.alert.id,
        })),
    )
        .into_response()
}

/// `POST /reminders` — receive a daily digest of open alerts and incidents
/// (built by an external K8s CronJob that queries the Keep database) and post
/// it as a single Markdown table message to Zulip.
///
/// Unlike `/alerts` and `/incidents`, the payload shape is fixed and produced
/// internally, so it is deserialized directly via [`Reminder`] rather than
/// validated field-by-field; malformed JSON is rejected by the `Json`
/// extractor with `400 Bad Request`.
///
/// # Responses
/// - `200 OK` — reminder accepted and dispatched.
pub async fn receive_reminder(
    State(state): State<AppState>,
    Json(reminder): Json<Reminder>,
) -> Response {
    tracing::info!(
        alerts = reminder.alerts.len(),
        incidents = reminder.incidents.len(),
        "Received daily reminder"
    );

    output::send_reminder(&reminder, &state.config).await;

    (
        StatusCode::OK,
        Json(json!({ "status": "processed" })),
    )
        .into_response()
}
