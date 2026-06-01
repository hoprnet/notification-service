use std::fmt::Write;

use chrono::{DateTime, NaiveDateTime, Utc};

use crate::{config::Config, models::Alert};

/// Render an [`Alert`] as a Zulip-flavoured Markdown message.
///
/// # Layout
/// ```text
/// {severity_emoji} **[{name}]({keep_url})** · {status_emoji} {status}
///
/// *{annotations.summary}*               ← if present
///
/// {description}                          ← if present (and different from summary)
///
/// - **Application:** `…`                 ← only shown when present
/// - **Occurrences:** N
/// - **Assignee:** …
/// - **Reason:** `…`                       ← only shown when present
/// - **Pod:** `…`
/// - **Container:** `…`
/// - **Start time:** `…`              ← UTC ISO 8601
/// - **End time:** `…`                ← only shown when present, UTC ISO 8601
///
/// 📊 [Prometheus](…)  · 📖 [Runbook](…) ← Runbook only when present
/// ```
///
/// # Zulip message-update strategy (TODO)
/// To avoid flooding the stream with duplicate messages, the Zulip integration
/// should:
/// 1. Send a **new** message the first time an alert fires (identified by its
///    `fingerprint`).
/// 2. On every subsequent update for the same fingerprint, **edit** the
///    existing message via `PATCH /api/v1/messages/{id}` instead of posting a
///    new one.
///
/// This requires maintaining a persistent `fingerprint → zulip_message_id`
/// mapping (e.g. an in-memory `HashMap` wrapped in `Arc<Mutex<…>>`, or an
/// external store such as Redis).
pub fn to_markdown(alert: &Alert, config: &Config) -> String {
    let mut out = String::new();

    // ── Header ────────────────────────────────────────────────────────────────
    // The alert name is linked to the Keep alert page when KEEP_BASE_URL is
    // configured.  When it is not, the name is rendered as plain bold text —
    // there is no fallback to Prometheus (which appears separately in the footer).
    let header_name = match config.keep_alert_url(&alert.fingerprint) {
        Some(keep_url) => format!("**[{}]({})**", alert.name, keep_url),
        None => format!("**{}**", alert.name),
    };

    writeln!(
        out,
        "{} {} · {} {}",
        severity_emoji(&alert.severity),
        header_name,
        status_emoji(&alert.status),
        alert.status,
    )
    .unwrap();

    // ── Annotations summary (short, italic) ───────────────────────────────────
    if let Some(summary) = &alert.annotations.summary {
        writeln!(out).unwrap();
        writeln!(out, "{}", summary).unwrap();
    }

    // ── Description ───────────────────────────────────────────────────────────
    if let Some(description) = &alert.description {
        let same_as_summary = alert
            .annotations
            .summary
            .as_deref()
            .map(|s| s == description.as_str())
            .unwrap_or(false);

        if !same_as_summary {
            writeln!(out).unwrap();
            writeln!(out, "**Description:** {}", description).unwrap();
        }
    }


    // ── Root cause analysis ───────────────────────────────────────────────────
    if let Some(rca) = &alert.rca_summary {
        writeln!(out).unwrap();
        writeln!(out, "**Root cause analysis:** {}", rca).unwrap();
    }


    // ── Bullet list ───────────────────────────────────────────────────────────
    writeln!(out, "**Details:**").unwrap();

    if let Some(app) = &alert.labels.app_name {
        writeln!(out, "- **Application:** `{}`", app).unwrap();
    }

    writeln!(out, "- **Occurrences:** {}", alert.firing_counter).unwrap();

    match &alert.assignee {
        Some(a) if !a.is_empty() => writeln!(out, "- **Assignee:** {}", a).unwrap(),
        _ => writeln!(out, "- **Assignee:** —").unwrap(),
    }

    if let Some(reason) = &alert.labels.reason {
        writeln!(out, "- **Reason:** `{}`", reason).unwrap();
    }
    if let Some(pod) = &alert.labels.pod {
        writeln!(out, "- **Pod:** `{}`", pod).unwrap();
    }
    if let Some(container) = &alert.labels.container {
        writeln!(out, "- **Container:** `{}`", container).unwrap();
    }
    // Only shown when both fields are present AND a Keep URL can be built.
    if let (Some(parent_name), Some(url)) = (
        &alert.correlated_parent_alert,
        alert.correlated_parent_fingerprint.as_deref().and_then(|fp| config.keep_alert_url(fp)),
    ) {
        writeln!(out, "- **Correlated alerts:** [{}]({})", parent_name, url).unwrap();
    }

    if let Some(ts) = parse_datetime_to_iso(&alert.started_at) {
        writeln!(out, "- **Start time:** {}", ts).unwrap();
    }
    if let Some(ref ends_at) = alert.ends_at {
        if let Some(ts) = parse_datetime_to_iso(ends_at) {
            writeln!(out, "- **End time:** {}", ts).unwrap();
        }
    }

    // ── Footer links ──────────────────────────────────────────────────────────
    writeln!(out).unwrap();

    let mut footer: Vec<String> = Vec::new();
    if let Some(ref generator_url) = alert.generator_url {
        footer.push(format!("📊 [Prometheus]({})", generator_url));
    }
    if let Some(runbook_url) = &alert.annotations.runbook_url {
        footer.push(format!("📖 [Runbook]({})", runbook_url));
    }
    if let Some(graylog_url) = build_graylog_url(alert, config) {
        footer.push(format!("🔍 [Graylog]({})", graylog_url));
    }
    writeln!(out, "{}", footer.join(" · ")).unwrap();

    out
}

// ---------------------------------------------------------------------------
// Graylog URL builder
// ---------------------------------------------------------------------------

/// Build a Graylog deep-link for the alert, or return `None` when:
/// - `GRAYLOG_BASE_URL` is not configured, or
/// - `alert.namespace` or `alert.labels.app_name` is absent.
///
/// URL pattern:
/// `{GRAYLOG_BASE_URL}namespace%3A+%22{ns}%22+AND+label_name%3A+%22{app}%22
///  &rangetype=absolute&from={start_iso}&to={now_iso}`
fn build_graylog_url(alert: &Alert, config: &Config) -> Option<String> {
    if config.graylog_base_url.is_empty() {
        return None;
    }
    let namespace = alert.namespace.as_deref()?;
    let app_name = alert.labels.app_name.as_deref()?;

    let start_iso = parse_datetime_to_iso(&alert.started_at)?;
    let end_iso = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

    Some(format!(
        "{}namespace%3A+%22{}%22+AND+label_name%3A+%22{}%22\
         &rangetype=absolute&from={}&to={}",
        config.graylog_base_url,
        url_encode_value(namespace),
        url_encode_value(app_name),
        url_encode_timestamp(&start_iso),
        url_encode_timestamp(&end_iso),
    ))
}

/// Try to parse a datetime string in several formats and return an ISO 8601
/// UTC string (`2026-05-26T09:06:25.383Z`).  Returns `None` if no format
/// matches.
fn parse_datetime_to_iso(s: &str) -> Option<String> {
    // "2026-05-26 09:06:25.383000"  (Keep format — space separator, microseconds)
    if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f") {
        return Some(naive.and_utc().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string());
    }
    // "2026-05-26T09:06:25.265Z"  (ISO 8601 with Z)
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(
            dt.with_timezone(&Utc)
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string(),
        );
    }
    // "2026-05-26T09:06:25.383000"  (T separator, no timezone)
    if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f") {
        return Some(naive.and_utc().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string());
    }
    None
}

/// Percent-encode special characters for use inside a query-string value.
/// Uses `+` for spaces (form-encoding style, matching the Graylog URL format).
fn url_encode_value(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "+".to_string(),
            '"' => "%22".to_string(),
            '%' => "%25".to_string(),
            '+' => "%2B".to_string(),
            '&' => "%26".to_string(),
            '=' => "%3D".to_string(),
            '#' => "%23".to_string(),
            _ => c.to_string(),
        })
        .collect()
}

/// Percent-encode `:` in an ISO 8601 timestamp for use as a query-string value.
fn url_encode_timestamp(s: &str) -> String {
    s.replace(':', "%3A")
}

// ---------------------------------------------------------------------------
// Emoji helpers
// ---------------------------------------------------------------------------

fn severity_emoji(severity: &str) -> &'static str {
    match severity.to_lowercase().as_str() {
        "critical" => "🔴",
        "warning" => "🟡",
        "info" => "🔵",
        "page" => "🚨",
        _ => "⚪",
    }
}

fn status_emoji(status: &str) -> &'static str {
    match status.to_lowercase().as_str() {
        "firing" => "🔥",
        "resolved" => "✅",
        "acknowledged" => "👀",
        "suppressed" => "🔕",
        _ => "❓",
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AlertAnnotations, AlertLabels};

    fn make_config(keep_base_url: &str) -> Config {
        Config {
            port: 8080,
            keep_base_url: keep_base_url.to_string(),
            graylog_base_url: String::new(),
            environment_name: "test".to_string(),
            zulip_email: String::new(),
            zulip_api_key: String::new(),
            zulip_host: String::new(),
            zulip_default_stream: String::new(),
            zulip_namespace_streams: std::collections::HashMap::new(),
            zulip_enabled: false,
            message_ttl_days: 7,
            zulip_request_timeout_secs: 10,
        }
    }

    fn make_alert(status: &str, severity: &str) -> Alert {
        Alert {
            id: "test-001".into(),
            name: "TestAlert".into(),
            status: status.into(),
            severity: severity.into(),
            started_at: "2026-01-01 00:00:00".into(),
            last_received: "2026-01-01T01:00:00Z".into(),
            firing_counter: 3,
            fingerprint: "abc123def456".into(),
            assignee: None,
            ends_at: Some("2026-01-01T02:00:00Z".into()),
            generator_url: Some("http://prometheus/graph".into()),
            description: None,
            rca_summary: None,
            correlated_parent_alert: None,
            correlated_parent_fingerprint: None,
            namespace: Some("monitoring".into()),
            labels: AlertLabels {
                pod: Some("my-pod".into()),
                reason: Some("OOMKilled".into()),
                container: None,
                app_name: None,
            },
            annotations: AlertAnnotations {
                summary: Some("Pod is out of memory.".into()),
                runbook_url: None,
            },
        }
    }

    #[test]
    fn firing_critical_contains_emoji() {
        let msg = to_markdown(&make_alert("firing", "critical"), &make_config(""));
        assert!(msg.contains("🔴"));
        assert!(msg.contains("🔥"));
    }

    #[test]
    fn resolved_warning_contains_emoji() {
        let msg = to_markdown(&make_alert("resolved", "warning"), &make_config(""));
        assert!(msg.contains("🟡"));
        assert!(msg.contains("✅"));
    }

    #[test]
    fn absent_labels_not_shown() {
        let msg = to_markdown(&make_alert("firing", "warning"), &make_config(""));
        assert!(!msg.contains("Container"));
        assert!(!msg.contains("Application"));
    }

    #[test]
    fn absent_runbook_not_shown() {
        let msg = to_markdown(&make_alert("firing", "warning"), &make_config(""));
        assert!(!msg.contains("Runbook"));
    }

    #[test]
    fn prometheus_link_always_present() {
        let msg = to_markdown(&make_alert("firing", "warning"), &make_config(""));
        assert!(msg.contains("📊"));
        assert!(msg.contains("http://prometheus/graph"));
    }

    #[test]
    fn keep_url_used_when_configured() {
        let msg = to_markdown(
            &make_alert("firing", "warning"),
            &make_config("https://incidents.example.com"),
        );
        assert!(msg.contains(
            "https://incidents.example.com/alerts/feed?alertPayloadFingerprint=abc123def456"
        ));
    }

    #[test]
    fn plain_name_when_no_keep_configured() {
        let msg = to_markdown(&make_alert("firing", "warning"), &make_config(""));
        // Name appears as bold text, not a hyperlink
        assert!(msg.contains("**TestAlert**"));
        assert!(!msg.contains("[TestAlert]"));
        // Prometheus URL still appears in the footer, not in the header
        assert!(msg.contains("http://prometheus/graph"));
    }

    #[test]
    fn occurrences_label_used() {
        let msg = to_markdown(&make_alert("firing", "warning"), &make_config(""));
        assert!(msg.contains("Occurrences"));
        assert!(!msg.contains("Firing count"));
        assert!(msg.contains("3"));
    }

    #[test]
    fn graylog_link_shown_when_configured() {
        let mut alert = make_alert("firing", "warning");
        alert.labels.app_name = Some("my-app".into());
        alert.namespace = Some("monitoring".into());
        let mut config = make_config("");
        config.graylog_base_url = "https://graylog.example.com/search/abc123?q=".to_string();
        let msg = to_markdown(&alert, &config);
        assert!(msg.contains("🔍"));
        assert!(msg.contains("graylog.example.com"));
        assert!(msg.contains("namespace%3A"));
        assert!(msg.contains("my-app"));
    }

    #[test]
    fn graylog_link_hidden_without_base_url() {
        let mut alert = make_alert("firing", "warning");
        alert.labels.app_name = Some("my-app".into());
        let msg = to_markdown(&alert, &make_config(""));
        assert!(!msg.contains("Graylog"));
    }

    #[test]
    fn graylog_link_hidden_without_app_name() {
        let mut config = make_config("");
        config.graylog_base_url = "https://graylog.example.com/search/abc123?q=".to_string();
        // make_alert has app_name = None
        let msg = to_markdown(&make_alert("firing", "warning"), &config);
        assert!(!msg.contains("Graylog"));
    }

    #[test]
    fn rca_summary_shown_when_present() {
        let mut alert = make_alert("firing", "critical");
        alert.rca_summary = Some("Memory leak in the connection pool introduced in v1.4.2.".into());
        let msg = to_markdown(&alert, &make_config(""));
        assert!(msg.contains("**Root cause analysis:**"));
        assert!(msg.contains("Memory leak in the connection pool introduced in v1.4.2."));
    }

    #[test]
    fn rca_summary_hidden_when_absent() {
        let msg = to_markdown(&make_alert("firing", "warning"), &make_config(""));
        assert!(!msg.contains("Root cause analysis"));
    }

    #[test]
    fn correlated_alert_shown_with_keep_url() {
        let mut alert = make_alert("firing", "critical");
        alert.correlated_parent_alert = Some("ParentAlert".into());
        alert.correlated_parent_fingerprint = Some("deadbeef1234".into());
        let msg = to_markdown(&alert, &make_config("https://incidents.example.com"));
        assert!(msg.contains("**Correlated alerts:**"));
        assert!(msg.contains("[ParentAlert]"));
        assert!(msg.contains("alertPayloadFingerprint=deadbeef1234"));
    }

    #[test]
    fn correlated_alert_hidden_without_keep_url() {
        let mut alert = make_alert("firing", "critical");
        alert.correlated_parent_alert = Some("ParentAlert".into());
        alert.correlated_parent_fingerprint = Some("deadbeef1234".into());
        // No KEEP_BASE_URL configured → bullet must be suppressed entirely
        let msg = to_markdown(&alert, &make_config(""));
        assert!(!msg.contains("Correlated alerts"));
    }

    #[test]
    fn correlated_alert_hidden_when_absent() {
        let msg = to_markdown(&make_alert("firing", "warning"), &make_config("https://incidents.example.com"));
        assert!(!msg.contains("Correlated alerts"));
    }

    #[test]
    fn no_dates_in_output() {
        let msg = to_markdown(&make_alert("firing", "warning"), &make_config(""));
        assert!(!msg.contains("Started"));
        assert!(!msg.contains("Last seen"));
        assert!(!msg.contains("Ends"));
        assert!(!msg.contains("Fingerprint"));
    }
}
