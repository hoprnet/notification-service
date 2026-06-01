use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::{models::Alert, output, processing, AppState};

/// `POST /incident` — receive a Keep incident payload.
///
/// The incident payload schema is not yet defined.  This handler accepts any
/// valid JSON, logs the raw body at INFO level so the fields can be inspected,
/// and returns `200 OK`.  Once the schema is known, replace this stub with
/// a proper model, formatter, and Zulip dispatch (mirroring `receive_alert`).
///
/// # Responses
/// - `200 OK` — payload accepted (always, as long as it is valid JSON).
/// - `422 Unprocessable Entity` — body is not valid JSON.
pub async fn receive_incident(
    Json(payload): Json<serde_json::Value>,
) -> Response {
    tracing::info!(payload = %payload, "Received incident");
    (
        StatusCode::OK,
        Json(json!({ "status": "received" })),
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
