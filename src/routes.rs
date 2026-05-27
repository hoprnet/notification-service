use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use std::sync::Arc;

use crate::{config::Config, models::Alert, output, processing};

/// `POST /alerts` — receive an alert payload, enrich it, and dispatch it.
///
/// # Request
/// JSON body matching [`Alert`].
///
/// # Response
/// - `200 OK` — alert was accepted and processed successfully.
/// - `422 Unprocessable Entity` — request body is invalid or missing required fields.
pub async fn receive_alert(
    State(config): State<Arc<Config>>,
    Json(alert): Json<Alert>,
) -> impl IntoResponse {
    tracing::info!(
        id = %alert.id,
        name = %alert.name,
        severity = %alert.severity,
        source = %alert.source,
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
