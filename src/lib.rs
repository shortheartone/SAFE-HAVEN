//! Core library for the application.
//!
//! This module provides the foundational types and utilities used across the
//! service, including comprehensive audit logging for security-relevant
//! events (issue #964).

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// A security-relevant event that must be captured for compliance audits and
/// incident investigations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityEvent {
    /// A login attempt, successful or not.
    Login {
        user: String,
        success: bool,
        reason: Option<String>,
    },
    /// A read of sensitive data, with the stated purpose.
    DataAccess {
        user: String,
        resource: String,
        purpose: String,
    },
    /// A modification of data, with the stated purpose.
    DataModification {
        user: String,
        resource: String,
        purpose: String,
    },
    /// A permission or role change, capturing old and new values.
    PermissionChange {
        actor: String,
        subject: String,
        old_value: String,
        new_value: String,
    },
}

impl SecurityEvent {
    /// A stable, machine-readable event type used for querying and reporting.
    pub fn event_type(&self) -> &'static str {
        match self {
            SecurityEvent::Login { .. } => "login",
            SecurityEvent::DataAccess { .. } => "data_access",
            SecurityEvent::DataModification { .. } => "data_modification",
            SecurityEvent::PermissionChange { .. } => "permission_change",
        }
    }

    /// The acting user associated with the event, when applicable.
    pub fn actor(&self) -> Option<&str> {
        match self {
            SecurityEvent::Login { user, .. } => Some(user),
            SecurityEvent::DataAccess { user, .. } => Some(user),
            SecurityEvent::DataModification { user, .. } => Some(user),
            SecurityEvent::PermissionChange { actor, .. } => Some(actor),
        }
    }
}

/// An immutable, timestamped audit log record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditRecord {
    /// Monotonically increasing sequence number assigned on append.
    pub sequence: u64,
    /// Unix timestamp (seconds) at which the event was recorded.
    pub timestamp: u64,
    /// The captured security event.
    pub event: SecurityEvent,
}

/// Default number of records retained for compliance needs.
pub const DEFAULT_RETENTION: usize = 10_000;

/// Append-only audit log. Records are never mutated or removed except by
/// retention trimming of the oldest entries once the configured capacity is
/// exceeded.
#[derive(Debug, Clone)]
pub struct AuditLog {
    inner: Arc<Mutex<AuditLogInner>>,
}

#[derive(Debug)]
struct AuditLogInner {
    records: VecDeque<AuditRecord>,
    next_sequence: u64,
    capacity: usize,
}

impl AuditLog {
    /// Create an audit log with the default retention capacity.
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_RETENTION)
    }

    /// Create an audit log retaining at most `capacity` records.
    pub fn with_capacity(capacity: usize) -> Self {
        AuditLog {
            inner: Arc::new(Mutex::new(AuditLogInner {
                records: VecDeque::new(),
                next_sequence: 0,
                capacity: capacity.max(1),
            })),
        }
    }

    /// Append a security event, returning the immutable record created.
    pub fn record(&self, event: SecurityEvent) -> AuditRecord {
        let mut inner = self.inner.lock().expect("audit log poisoned");
        let record = AuditRecord {
            sequence: inner.next_sequence,
            timestamp: now_secs(),
            event,
        };
        inner.next_sequence += 1;
        inner.records.push_back(record.clone());
        while inner.records.len() > inner.capacity {
            inner.records.pop_front();
        }
        record
    }

    /// Return all retained records in append order.
    pub fn records(&self) -> Vec<AuditRecord> {
        let inner = self.inner.lock().expect("audit log poisoned");
        inner.records.iter().cloned().collect()
    }

    /// Query retained records by event type (see [`SecurityEvent::event_type`]).
    pub fn query_by_type(&self, event_type: &str) -> Vec<AuditRecord> {
        let inner = self.inner.lock().expect("audit log poisoned");
        inner
            .records
            .iter()
            .filter(|r| r.event.event_type() == event_type)
            .cloned()
            .collect()
    }

    /// Query retained records by acting user.
    pub fn query_by_actor(&self, actor: &str) -> Vec<AuditRecord> {
        let inner = self.inner.lock().expect("audit log poisoned");
        inner
            .records
            .iter()
            .filter(|r| r.event.actor() == Some(actor))
            .cloned()
            .collect()
    }

    /// Number of records currently retained.
    pub fn len(&self) -> usize {
        self.inner.lock().expect("audit log poisoned").records.len()
    }

    /// Whether the audit log currently holds no records.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_attempts_are_logged_with_result() {
        let log = AuditLog::new();
        log.record(SecurityEvent::Login {
            user: "alice".into(),
            success: true,
            reason: None,
        });
        log.record(SecurityEvent::Login {
            user: "bob".into(),
            success: false,
            reason: Some("bad password".into()),
        });
        let logins = log.query_by_type("login");
        assert_eq!(logins.len(), 2);
        assert!(matches!(
            logins[0].event,
            SecurityEvent::Login { success: true, .. }
        ));
        assert!(matches!(
            logins[1].event,
            SecurityEvent::Login { success: false, .. }
        ));
    }

    #[test]
    fn permission_changes_capture_old_and_new_values() {
        let log = AuditLog::new();
        log.record(SecurityEvent::PermissionChange {
            actor: "admin".into(),
            subject: "alice".into(),
            old_value: "viewer".into(),
            new_value: "editor".into(),
        });
        let changes = log.query_by_type("permission_change");
        assert_eq!(changes.len(), 1);
        match &changes[0].event {
            SecurityEvent::PermissionChange {
                old_value,
                new_value,
                ..
            } => {
                assert_eq!(old_value, "viewer");
                assert_eq!(new_value, "editor");
            }
            _ => panic!("unexpected event"),
        }
    }

    #[test]
    fn records_are_immutable_and_ordered() {
        let log = AuditLog::new();
        let first = log.record(SecurityEvent::DataAccess {
            user: "alice".into(),
            resource: "report".into(),
            purpose: "audit".into(),
        });
        let second = log.record(SecurityEvent::DataModification {
            user: "alice".into(),
            resource: "report".into(),
            purpose: "correction".into(),
        });
        assert_eq!(first.sequence, 0);
        assert_eq!(second.sequence, 1);
        assert_eq!(log.records()[0], first);
    }

    #[test]
    fn retention_trims_oldest_records() {
        let log = AuditLog::with_capacity(2);
        for i in 0..3 {
            log.record(SecurityEvent::Login {
                user: format!("user{i}"),
                success: true,
                reason: None,
            });
        }
        assert_eq!(log.len(), 2);
        assert_eq!(log.records()[0].sequence, 1);
    }
}
