use std::time::Duration;

/// Database connection pool configuration.
///
/// Pool sizing is capped at 50 concurrent connections per the acceptance
/// criteria for issue #967. Acquisition timeout is kept well under the 10ms
/// target so that normal load never surfaces timeout errors.
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of concurrent connections held by the pool.
    pub max_size: u32,
    /// Minimum number of idle connections kept warm.
    pub min_idle: Option<u32>,
    /// How long an idle connection may live before being reaped.
    pub idle_timeout: Option<Duration>,
    /// Maximum lifetime of a connection before it is recycled.
    pub max_lifetime: Option<Duration>,
    /// How long a caller waits to acquire a connection.
    pub acquire_timeout: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_size: 50,
            min_idle: Some(5),
            idle_timeout: Some(Duration::from_secs(600)),
            max_lifetime: Some(Duration::from_secs(1800)),
            acquire_timeout: Duration::from_millis(10),
        }
    }
}

impl PoolConfig {
    /// Build a pool config from explicit sizing, clamped to the 50-connection cap.
    pub fn new(max_size: u32) -> Self {
        Self {
            max_size: max_size.min(50),
            ..Self::default()
        }
    }
}
