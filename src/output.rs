use std::time::{Duration, Instant};

use serde::Deserialize;

use crate::{config::Config, formatter, message_store::MessageStore, models::EnrichedAlert};

/// Send or update a Zulip message for the given enriched alert.
///
/// # Routing
/// - **Stream** — resolved by mapping `alert.namespace` through
///   [`Config::stream_for_namespace`], falling back to `ZULIP_DEFAULT_STREAM`.
/// - **Topic** — `alerts-{ENVIRONMENT_NAME}` via [`Config::zulip_topic`].
///
/// # Deduplication
/// Uses `alert.fingerprint` as the correlation key:
/// - First time a fingerprint is seen → POST a new Zulip message, store the
///   returned `message_id` in `messages`.
/// - Subsequent updates → PATCH the existing message so the stream stays clean.
/// - On service restart the store is empty; the next update creates a fresh
///   message and resumes tracking from there.
///
/// # Timeouts
/// A single `reqwest::Client` is built here with `ZULIP_REQUEST_TIMEOUT_SECS`
/// (default 10 s) and shared across both the POST and PATCH calls so that a
/// slow or stuck Zulip server never blocks the handler indefinitely.
pub async fn send(alert: &EnrichedAlert, config: &Config, messages: &MessageStore) {
    let markdown = formatter::to_markdown(&alert.alert, config);

    tracing::info!("Zulip message preview:\n{}", markdown);

    // Bail early only when Zulip is enabled but credentials are missing.
    // When ZULIP_ENABLED=false we still run the full deduplication flow so that
    // the store is exercised and dry-run logs reflect exactly what would happen.
    if config.zulip_enabled && !config.zulip_configured() {
        tracing::warn!("Zulip is enabled but not fully configured — skipping");
        return;
    }

    let stream      = config.stream_for_namespace(alert.alert.namespace.as_deref());
    let topic       = config.zulip_topic();
    let fingerprint = &alert.alert.fingerprint;

    // Read the stored message ID without holding the lock across await points.
    let (existing_id, store_size) = {
        let store = messages.lock().expect("message store lock poisoned");
        (store.get(fingerprint).map(|(id, _)| *id), store.len())
    };

    tracing::info!(
        fingerprint,
        store_size,
        cached_msg_id = ?existing_id,
        "Fingerprint store lookup"
    );

    // Build one shared client for whichever API call follows.
    let client = build_client(config);

    if let Some(msg_id) = existing_id {
        tracing::info!(stream, topic, msg_id, fingerprint, "Updating existing Zulip message");

        if let Err(e) = patch_zulip_message(&client, config, msg_id, &markdown).await {
            tracing::error!(error = %e, msg_id, fingerprint, "Failed to update Zulip message");
        }
    } else {
        tracing::info!(stream, topic, fingerprint, "Sending new Zulip message");

        match post_zulip_message(&client, config, stream, &topic, &markdown).await {
            Ok(msg_id) => {
                messages
                    .lock()
                    .expect("message store lock poisoned")
                    .insert(fingerprint.clone(), (msg_id, Instant::now()));
                tracing::info!(
                    msg_id,
                    fingerprint,
                    store_size = store_size + 1,
                    "Stored new Zulip message ID"
                );
            }
            Err(e) => tracing::error!(error = %e, fingerprint, "Failed to send Zulip message"),
        }
    }
}

// ---------------------------------------------------------------------------
// HTTP client
// ---------------------------------------------------------------------------

/// Build a `reqwest::Client` with the configured request timeout.
///
/// Called once per `send()` invocation so the timeout is always applied and
/// no client state leaks between requests.
fn build_client(config: &Config) -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(config.zulip_request_timeout_secs))
        .build()
        .expect("Failed to build reqwest client")
}

// ---------------------------------------------------------------------------
// Zulip API
// ---------------------------------------------------------------------------

/// Decoded subset of the Zulip `POST /api/v1/messages` response.
#[derive(Deserialize)]
struct ZulipSendResponse {
    /// The ID of the newly created message — present on success.
    id: Option<u64>,
    result: String,
}

/// POST a new message to a Zulip stream.
///
/// Returns the Zulip message ID on success.
/// When `ZULIP_ENABLED=false` the HTTP call is skipped and a synthetic ID of
/// `0` is returned so the rest of the deduplication flow behaves normally.
async fn post_zulip_message(
    client: &reqwest::Client,
    config: &Config,
    stream: &str,
    topic: &str,
    content: &str,
) -> Result<u64, String> {
    if !config.zulip_enabled {
        tracing::info!(stream, topic, "[DRY-RUN] POST Zulip message (ZULIP_ENABLED=false)");
        return Ok(0);
    }

    let url = format!("https://{}/api/v1/messages", config.zulip_host);

    tracing::debug!(
        url,
        email       = %config.zulip_email,
        api_key_len = config.zulip_api_key.len(),
        stream,
        topic,
        timeout_secs = config.zulip_request_timeout_secs,
        "POST Zulip message"
    );

    let response = client
        .post(&url)
        .basic_auth(&config.zulip_email, Some(&config.zulip_api_key))
        .form(&[
            ("type",    "stream"),
            ("to",      stream),
            ("topic",   topic),
            ("content", content),
        ])
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    let status = response.status();
    let body   = response.text().await.unwrap_or_default();

    if !status.is_success() {
        return Err(format!("Zulip API error {status}: {body}"));
    }

    let resp: ZulipSendResponse = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse Zulip response: {e} — body: {body}"))?;

    if resp.result != "success" {
        return Err(format!("Zulip returned non-success result: {body}"));
    }

    resp.id
        .ok_or_else(|| format!("Zulip response missing 'id' field: {body}"))
}

/// PATCH an existing Zulip message with new content.
///
/// When `ZULIP_ENABLED=false` the HTTP call is skipped and `Ok(())` is
/// returned immediately so the caller's log messages still appear.
async fn patch_zulip_message(
    client: &reqwest::Client,
    config: &Config,
    msg_id: u64,
    content: &str,
) -> Result<(), String> {
    if !config.zulip_enabled {
        tracing::info!(msg_id, "[DRY-RUN] PATCH Zulip message (ZULIP_ENABLED=false)");
        return Ok(());
    }

    let url = format!("https://{}/api/v1/messages/{}", config.zulip_host, msg_id);

    tracing::debug!(
        url,
        msg_id,
        timeout_secs = config.zulip_request_timeout_secs,
        "PATCH Zulip message"
    );

    let response = client
        .patch(&url)
        .basic_auth(&config.zulip_email, Some(&config.zulip_api_key))
        .form(&[("content", content)])
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    let status = response.status();
    if status.is_success() {
        tracing::debug!(msg_id, "Zulip PATCH succeeded ({})", status);
        Ok(())
    } else {
        let body = response.text().await.unwrap_or_default();
        Err(format!("Zulip PATCH error {status}: {body}"))
    }
}
