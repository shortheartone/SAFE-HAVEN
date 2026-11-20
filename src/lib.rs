//! Feature flag system with percentage rollouts and user/group targeting.
//!
//! Flags are defined in code (or loaded from storage) and evaluated at runtime,
//! so toggling a flag does not require a redeploy. Evaluation is deterministic
//! per user so a given user consistently sees the same variant.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// A stable hash of a user id combined with a flag key, mapped to 0..100.
fn bucket(flag_key: &str, user_id: &str) -> u32 {
    // FNV-1a 64-bit hash for stable, dependency-free bucketing.
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in flag_key.as_bytes().iter().chain(user_id.as_bytes()) {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    (hash % 100) as u32
}

/// Targeting rules for a flag.
#[derive(Debug, Clone, Default)]
pub struct Targeting {
    /// Specific user ids that always receive the flag.
    pub users: Vec<String>,
    /// Groups that always receive the flag.
    pub groups: Vec<String>,
    /// Percentage (0-100) of remaining users that receive the flag.
    pub percentage: u32,
}

/// Definition of a single feature flag.
#[derive(Debug, Clone)]
pub struct Flag {
    pub key: String,
    pub enabled: bool,
    pub targeting: Targeting,
}

impl Flag {
    pub fn new(key: impl Into<String>) -> Self {
        Flag {
            key: key.into(),
            enabled: false,
            targeting: Targeting::default(),
        }
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn percentage(mut self, percentage: u32) -> Self {
        self.targeting.percentage = percentage.min(100);
        self
    }

    pub fn target_user(mut self, user_id: impl Into<String>) -> Self {
        self.targeting.users.push(user_id.into());
        self
    }

    pub fn target_group(mut self, group: impl Into<String>) -> Self {
        self.targeting.groups.push(group.into());
        self
    }

    /// Evaluate the flag for a user, optionally belonging to groups.
    pub fn evaluate(&self, user_id: &str, groups: &[String]) -> bool {
        if !self.enabled {
            return false;
        }
        if self.targeting.users.iter().any(|u| u == user_id) {
            return true;
        }
        if groups
            .iter()
            .any(|g| self.targeting.groups.iter().any(|tg| tg == g))
        {
            return true;
        }
        if self.targeting.percentage > 0 {
            return bucket(&self.key, user_id) < self.targeting.percentage;
        }
        // Enabled with no targeting: on for everyone.
        true
    }
}

/// Thread-safe registry of feature flags.
#[derive(Clone, Default)]
pub struct FlagStore {
    flags: Arc<RwLock<HashMap<String, Flag>>>,
}

impl FlagStore {
    pub fn new() -> Self {
        FlagStore::default()
    }

    /// Register or replace a flag definition.
    pub fn set(&self, flag: Flag) {
        let mut flags = self.flags.write().unwrap();
        flags.insert(flag.key.clone(), flag);
    }

    /// Toggle a flag on or off without redeploying.
    pub fn toggle(&self, key: &str, enabled: bool) -> bool {
        let mut flags = self.flags.write().unwrap();
        match flags.get_mut(key) {
            Some(flag) => {
                flag.enabled = enabled;
                true
            }
            None => false,
        }
    }

    /// Update the percentage rollout for a flag.
    pub fn set_percentage(&self, key: &str, percentage: u32) -> bool {
        let mut flags = self.flags.write().unwrap();
        match flags.get_mut(key) {
            Some(flag) => {
                flag.targeting.percentage = percentage.min(100);
                true
            }
            None => false,
        }
    }

    /// Evaluate a flag for a user. Unknown flags evaluate to `false`.
    pub fn is_enabled(&self, key: &str, user_id: &str, groups: &[String]) -> bool {
        let flags = self.flags.read().unwrap();
        flags
            .get(key)
            .map(|flag| flag.evaluate(user_id, groups))
            .unwrap_or(false)
    }

    /// Snapshot of all flags, useful for a management UI/API.
    pub fn list(&self) -> Vec<Flag> {
        let flags = self.flags.read().unwrap();
        flags.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_flag_is_off() {
        let store = FlagStore::new();
        store.set(Flag::new("new-ui"));
        assert!(!store.is_enabled("new-ui", "user-1", &[]));
    }

    #[test]
    fn toggle_without_redeploy() {
        let store = FlagStore::new();
        store.set(Flag::new("new-ui"));
        assert!(store.toggle("new-ui", true));
        assert!(store.is_enabled("new-ui", "user-1", &[]));
    }

    #[test]
    fn targets_specific_user() {
        let store = FlagStore::new();
        store.set(Flag::new("beta").enabled(true).target_user("alice"));
        assert!(store.is_enabled("beta", "alice", &[]));
        assert!(!store.is_enabled("beta", "bob", &[]));
    }

    #[test]
    fn targets_group() {
        let store = FlagStore::new();
        store.set(Flag::new("beta").enabled(true).target_group("staff"));
        assert!(store.is_enabled("beta", "bob", &["staff".to_string()]));
        assert!(!store.is_enabled("beta", "bob", &[]));
    }

    #[test]
    fn percentage_rollout_is_deterministic() {
        let store = FlagStore::new();
        store.set(Flag::new("rollout").enabled(true).percentage(50));
        let first = store.is_enabled("rollout", "user-42", &[]);
        let second = store.is_enabled("rollout", "user-42", &[]);
        assert_eq!(first, second);
    }

    #[test]
    fn unknown_flag_is_off() {
        let store = FlagStore::new();
        assert!(!store.is_enabled("missing", "user-1", &[]));
    }
}
