use std::fmt::Write;

use crate::models::Alert;

/// Render an [`Alert`] as Zulip-flavoured Markdown.
///
/// # Layout
/// ```text
/// {severity_emoji} **[{name}]({generator_url})** · {status_emoji} {status}
///
/// *{annotations.summary}*               ← if present
///
/// {description}                          ← if present (and different from summary)
///
/// **Pod:** `…` · **Reason:** `…` · …   ← only labels that are present
///
/// | | |
/// |---|---|
/// | **Started**      | … |
/// | **Last seen**    | … |
/// | **Ends**         | … |
/// | **Firing count** | … |
/// | **Fingerprint**  | `…` |
/// | **Assignee**     | … |
///
/// 📖 [Runbook](…)                        ← only if runbook_url is present
/// ```
pub fn to_zulip(alert: &Alert) -> String {
    let mut out = String::new();

    // ── Header ────────────────────────────────────────────────────────────────
    writeln!(
        out,
        "{} **[{}]({})** · {} {}",
        severity_emoji(&alert.severity),
        alert.name,
        alert.generator_url,
        status_emoji(&alert.status),
        alert.status,
    )
    .unwrap();

    // ── Annotations summary (short, italic) ───────────────────────────────────
    if let Some(summary) = &alert.annotations.summary {
        writeln!(out).unwrap();
        writeln!(out, "*{}*", summary).unwrap();
    }

    // ── Description (longer text, only if different from summary) ─────────────
    if let Some(desc) = &alert.description {
        let same_as_summary = alert
            .annotations
            .summary
            .as_deref()
            .map(|s| s == desc.as_str())
            .unwrap_or(false);

        if !same_as_summary {
            writeln!(out).unwrap();
            writeln!(out, "{}", desc).unwrap();
        }
    }

    // ── Labels line ───────────────────────────────────────────────────────────
    let label_parts: Vec<String> = [
        alert.labels.pod.as_deref().map(|v| format!("**Pod:** `{}`", v)),
        alert.labels.reason.as_deref().map(|v| format!("**Reason:** `{}`", v)),
        alert.labels.container.as_deref().map(|v| format!("**Container:** `{}`", v)),
        alert.labels.app_name.as_deref().map(|v| format!("**App:** `{}`", v)),
    ]
    .into_iter()
    .flatten()
    .collect();

    if !label_parts.is_empty() {
        writeln!(out).unwrap();
        writeln!(out, "{}", label_parts.join(" · ")).unwrap();
    }

    // ── Info table ────────────────────────────────────────────────────────────
    writeln!(out).unwrap();
    writeln!(out, "| | |").unwrap();
    writeln!(out, "|---|---|").unwrap();
    writeln!(out, "| **Started** | {} |", alert.started_at).unwrap();
    writeln!(out, "| **Last seen** | {} |", alert.last_received).unwrap();
    writeln!(out, "| **Ends** | {} |", alert.ends_at).unwrap();
    writeln!(out, "| **Firing count** | {} |", alert.firing_counter).unwrap();
    writeln!(out, "| **Fingerprint** | `{}` |", alert.fingerprint).unwrap();

    match &alert.assignee {
        Some(a) if !a.is_empty() => writeln!(out, "| **Assignee** | {} |", a).unwrap(),
        _ => writeln!(out, "| **Assignee** | — |").unwrap(),
    }

    // ── Runbook link ──────────────────────────────────────────────────────────
    if let Some(runbook_url) = &alert.annotations.runbook_url {
        writeln!(out).unwrap();
        writeln!(out, "📖 [Runbook]({})", runbook_url).unwrap();
    }

    out
}

// ---------------------------------------------------------------------------
// Emoji helpers
// ---------------------------------------------------------------------------

fn severity_emoji(severity: &str) -> &'static str {
    match severity.to_lowercase().as_str() {
        "critical" => "🔴",
        "warning"  => "🟡",
        "info"     => "🔵",
        "page"     => "🚨",
        _          => "⚪",
    }
}

fn status_emoji(status: &str) -> &'static str {
    match status.to_lowercase().as_str() {
        "firing"        => "🔥",
        "resolved"      => "✅",
        "acknowledged"  => "👀",
        "suppressed"    => "🔕",
        _               => "❓",
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AlertAnnotations, AlertLabels};

    fn make_alert(status: &str, severity: &str) -> Alert {
        Alert {
            id: "test-001".into(),
            name: "TestAlert".into(),
            status: status.into(),
            severity: severity.into(),
            started_at: "2026-01-01 00:00:00".into(),
            last_received: "2026-01-01T01:00:00Z".into(),
            firing_counter: 1,
            fingerprint: "abc123".into(),
            assignee: None,
            ends_at: "2026-01-01T02:00:00Z".into(),
            generator_url: "http://prometheus/graph".into(),
            description: None,
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
        let msg = to_zulip(&make_alert("firing", "critical"));
        assert!(msg.contains("🔴"));
        assert!(msg.contains("🔥"));
    }

    #[test]
    fn resolved_warning_contains_emoji() {
        let msg = to_zulip(&make_alert("resolved", "warning"));
        assert!(msg.contains("🟡"));
        assert!(msg.contains("✅"));
    }

    #[test]
    fn absent_labels_not_shown() {
        let msg = to_zulip(&make_alert("firing", "warning"));
        assert!(!msg.contains("Container"));
        assert!(!msg.contains("App"));
    }

    #[test]
    fn absent_runbook_not_shown() {
        let msg = to_zulip(&make_alert("firing", "warning"));
        assert!(!msg.contains("Runbook"));
    }
}
