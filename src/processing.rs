use crate::models::{Alert, EnrichedAlert};
use chrono::Utc;

/// Enrich an incoming alert with processing metadata.
///
/// Currently this adds a timestamp and flags the alert as enriched.
/// Future implementations may add deduplication IDs, runbook links,
/// team routing, severity normalisation, and so on.
pub fn enrich(alert: Alert) -> EnrichedAlert {
    EnrichedAlert {
        alert,
        processed_at: Utc::now(),
        enriched: true,
        service_name: "notification-service".to_string(),
    }
}
