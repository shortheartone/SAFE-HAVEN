use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single feature flag definition with targeting and rollout rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag {
    /// Unique flag name, e.g. "new_dashboard".
    pub name: String,
    /// Master on/off switch. When false the flag is off for everyone.
    pub enabled: bool,
    /// Percentage (0-100) of users for whom the flag is enabled.
    #[serde(default)]
    pub rollout_percentage: u8,
    /// Explicit user IDs that always get the flag.
    #[serde(default)]
    pub target_users: Vec<String>,
    /// Explicit group names that always get the flag.
    #[serde(default)]
    pub target_groups: Vec<String>,
}

impl FeatureFlag {
    /// Evaluate the flag for a given user and their groups.
    ///
    /// Targeting takes precedence over percentage rollout. Percentage
    /// rollout is deterministic per user so a user always sees a stable
    /// result for a given flag.
    pub fn evaluate(&self, user_id: &str, groups: &[String]) -> bool {
        if !self.enabled {
            return false;
        }
        if self.target_users.iter().any(|u| u == user_id) {
            return true;
        }
        if self
            .target_groups
            .iter()
            .any(|g| groups.iter().any(|ug| ug == g))
        {
            return true;
        }
        if self.rollout_percentage == 0 {
            return false;
        }
        if self.rollout_percentage >= 100 {
            return true;
        }
        bucket_for(&self.name, user_id) < self.rollout_percentage
    }
}

/// Deterministic bucket (0-99) for a user within a flag's rollout.
fn bucket_for(flag_name: &str, user_id: &str) -> u8 {
    // FNV-1a hash keeps evaluation stable across processes without
    // pulling in extra dependencies.
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in flag_name.bytes().chain(b"\0").chain(user_id.bytes()) {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    (hash % 100) as u8
}

/// Collection of feature flags keyed by name.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeatureFlags {
    #[serde(default)]
    pub flags: HashMap<String, FeatureFlag>,
}

impl FeatureFlags {
    /// Returns true when the named flag is enabled for the user.
    /// Unknown flags evaluate to false.
    pub fn is_enabled(&self, name: &str, user_id: &str, groups: &[String]) -> bool {
        self.flags
            .get(name)
            .map(|flag| flag.evaluate(user_id, groups))
            .unwrap_or(false)
    }

    /// Toggle a flag on or off without a redeploy.
    pub fn set_enabled(&mut self, name: &str, enabled: bool) -> bool {
        match self.flags.get_mut(name) {
            Some(flag) => {
                flag.enabled = enabled;
                true
            }
            None => false,
        }
    }

    /// Update the percentage rollout for a flag (clamped to 0-100).
    pub fn set_rollout_percentage(&mut self, name: &str, percentage: u8) -> bool {
        match self.flags.get_mut(name) {
            Some(flag) => {
                flag.rollout_percentage = percentage.min(100);
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flag(name: &str, enabled: bool, pct: u8) -> FeatureFlag {
        FeatureFlag {
            name: name.to_string(),
            enabled,
            rollout_percentage: pct,
            target_users: Vec::new(),
            target_groups: Vec::new(),
        }
    }

    #[test]
    fn disabled_flag_is_off() {
        assert!(!flag("f", false, 100).evaluate("u1", &[]));
    }

    #[test]
    fn full_rollout_is_on() {
        assert!(flag("f", true, 100).evaluate("u1", &[]));
    }

    #[test]
    fn zero_rollout_is_off() {
        assert!(!flag("f", true, 0).evaluate("u1", &[]));
    }

    #[test]
    fn targeted_user_is_on() {
        let mut f = flag("f", true, 0);
        f.target_users.push("u1".to_string());
        assert!(f.evaluate("u1", &[]));
        assert!(!f.evaluate("u2", &[]));
    }

    #[test]
    fn targeted_group_is_on() {
        let mut f = flag("f", true, 0);
        f.target_groups.push("beta".to_string());
        assert!(f.evaluate("u1", &["beta".to_string()]));
        assert!(!f.evaluate("u1", &[]));
    }

    #[test]
    fn percentage_rollout_is_deterministic() {
        let f = flag("f", true, 50);
        let first = f.evaluate("u1", &[]);
        assert_eq!(first, f.evaluate("u1", &[]));
    }

    #[test]
    fn toggle_without_redeploy() {
        let mut flags = FeatureFlags::default();
        flags.flags.insert("f".to_string(), flag("f", false, 100));
        assert!(!flags.is_enabled("f", "u1", &[]));
        assert!(flags.set_enabled("f", true));
        assert!(flags.is_enabled("f", "u1", &[]));
    }
}
