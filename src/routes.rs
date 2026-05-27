use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use std::sync::Arc;

use crate::{config::Config, models::Alert, output, processing};

/// `POST /alerts` — receive any JSON alert payload, extract known fields,
/// enrich it, and dispatch it.
///
/// # Request
/// Any valid JSON object.  Unknown fields are silently ignored.
///
/// # Response
/// - `200 OK` — alert was accepted and processed successfully.
/// - `422 Unprocessable Entity` — request body is not valid JSON.
pub async fn receive_alert(
    State(config): State<Arc<Config>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let alert = Alert::from_value(&payload);

    tracing::info!(
        id = ?alert.id,
        name = ?alert.name,
        severity = ?alert.severity,
        source = ?alert.source,
        "Received alert"
    );

    let enriched = processing::enrich(alert);
    output::send(&enriched, &config);

    (
        StatusCode::OK,
        Json(json!({
            "status": "processed",
            "id": enriched.alert.id,
        })),
    )
}
