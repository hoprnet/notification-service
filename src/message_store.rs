use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// Shared in-memory map of `fingerprint → (Zulip message ID, insertion time)`.
///
/// # Lifecycle
/// - On the first update for a given fingerprint the service POSTs a new
///   Zulip message and stores `(id, Instant::now())` here.
/// - On every subsequent update the service PATCHes the existing message.
/// - Entries are evicted by a background task once they exceed the configured
///   TTL (default: 7 days), causing the next update to open a fresh message.
/// - On service restart the map is empty.
///
/// # Thread safety
/// The `Arc<Mutex<…>>` allows the store to be shared across Tokio tasks.
/// Keep critical sections short — never hold the lock across `await` points.
pub type MessageStore = Arc<Mutex<HashMap<String, (u64, Instant)>>>;

/// Create a new, empty [`MessageStore`].
pub fn new() -> MessageStore {
    Arc::new(Mutex::new(HashMap::new()))
}

/// Spawn a Tokio background task that periodically removes entries older than
/// `ttl` from the store.
///
/// The sweep runs every `sweep_interval` (typically 1 hour).  Each run locks
/// the store briefly, removes stale entries with [`HashMap::retain`], then
/// releases the lock before sleeping again.
pub fn spawn_eviction_task(store: MessageStore, ttl: Duration, sweep_interval: Duration) {
    tokio::spawn(async move {
        tracing::info!(
            ttl_secs     = ttl.as_secs(),
            interval_secs = sweep_interval.as_secs(),
            "Message store eviction task started"
        );

        let mut ticker = tokio::time::interval(sweep_interval);
        ticker.tick().await; // skip the immediate first tick

        loop {
            ticker.tick().await;

            let (before, after) = {
                let mut map = store.lock().expect("message store lock poisoned");
                let before = map.len();
                map.retain(|_, (_, inserted_at)| inserted_at.elapsed() < ttl);
                (before, map.len())
            };

            let removed = before - after;
            if removed > 0 {
                tracing::info!(
                    removed,
                    remaining = after,
                    "Evicted expired Zulip message IDs from store"
                );
            } else {
                tracing::debug!(store_size = after, "Message store sweep: nothing evicted");
            }
        }
    });
}
