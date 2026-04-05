use super::types::NameCacheEntry;
use std::collections::HashMap;

const NAME_REFRESH_INTERVAL_MS: u64 = 15_000;
const NAME_CACHE_EXPIRY_MS: u64 = 15_000;
const LOG_NAME_CACHE_ACTIVITY: bool = false;

pub struct NameCache {
    entries: HashMap<i32, NameCacheEntry>,
}

impl NameCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn resolve_or_query<F>(&mut self, entity_id: i32, current_tick: u64, query_fn: F) -> String
    where
        F: FnOnce() -> String,
    {
        let entry = self
            .entries
            .entry(entity_id)
            .or_insert_with(|| NameCacheEntry {
                entity_id,
                name: String::new(),
                last_refresh_tick: 0,
                last_seen_tick: 0,
            });

        entry.last_seen_tick = current_tick;

        let should_refresh = entry.name.is_empty()
            || current_tick < entry.last_refresh_tick
            || (current_tick - entry.last_refresh_tick) >= NAME_REFRESH_INTERVAL_MS;

        if should_refresh {
            let refreshed = query_fn();
            if !refreshed.is_empty() {
                if LOG_NAME_CACHE_ACTIVITY && entry.name != refreshed {
                    println!(
                        "[ESP] Refreshed name cache: {} [id={}]",
                        refreshed, entity_id
                    );
                }
                entry.name = refreshed;
            }
            entry.last_refresh_tick = current_tick;
        }

        entry.name.clone()
    }

    pub fn prune(&mut self, current_tick: u64) {
        self.entries.retain(|_, entry| {
            let expired = current_tick < entry.last_seen_tick
                || (current_tick - entry.last_seen_tick) >= NAME_CACHE_EXPIRY_MS;
            if expired && LOG_NAME_CACHE_ACTIVITY && !entry.name.is_empty() {
                println!(
                    "[ESP] Invalidated cache: {} [id={}]",
                    entry.name, entry.entity_id
                );
            }
            !expired
        });
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}
