//! Core library for the application.
//!
//! This crate exposes the authentication helpers and a lightweight,
//! versioned database migration system used to keep dev, staging, and
//! production schemas in sync.

pub mod auth;
pub mod migrations;
pub mod settings;

pub use migrations::{Migration, MigrationRunner, MigrationStatus};
pub use settings::{
    NotificationFrequency, SettingsChange, SettingsError, SettingsManager, SettingsStore,
    Theme, UserSettings,
};
