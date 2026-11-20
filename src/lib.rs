//! System health monitoring for the application.
//!
//! Provides a lightweight, in-process metrics collection system plus a
//! dashboard renderer and threshold-based alerting. Metrics are kept in a
//! bounded ring buffer so historical data is available for the last 30 days.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Number of seconds in the 30-day historical retention window.
pub const RETENTION_SECS: u64 = 30 * 24 * 60 * 60;

/// How long a computed dashboard snapshot stays cached before recomputation.
pub const DASHBOARD_CACHE_TTL_SECS: u64 = 5;

/// Explicit allowlist of origins permitted to make cross-origin API requests.
///
/// No wildcard (`*`) is ever used: only these origins receive CORS headers.
pub const CORS_ALLOWED_ORIGINS: &[&str] = &[
    "https://app.example.com",
    "https://admin.example.com",
    "http://localhost:3000",
];

/// HTTP methods permitted for cross-origin requests.
pub const CORS_ALLOWED_METHODS: &[&str] = &["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"];

/// Request headers permitted for cross-origin requests.
pub const CORS_ALLOWED_HEADERS: &[&str] = &["Authorization", "Content-Type", "Accept"];

/// How long (seconds) browsers may cache a preflight response.
pub const CORS_MAX_AGE_SECS: u64 = 600;

/// Resolved CORS policy for a single request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorsPolicy {
    /// The request `Origin`, echoed back only when it is on the allowlist.
    pub allow_origin: Option<String>,
    /// Whether the request was a preflight (`OPTIONS` with `Access-Control-Request-Method`).
    pub is_preflight: bool,
}

/// Evaluate the CORS policy for an incoming request.
///
/// Returns `None` when the origin is not on the allowlist, in which case no
/// CORS headers should be emitted. Preflight requests are detected so the
/// caller can short-circuit them with a `204 No Content` response.
pub fn cors_policy(origin: Option<&str>, method: &str, requested_method: Option<&str>) -> Option<CorsPolicy> {
    let origin = origin?;
    if !CORS_ALLOWED_ORIGINS.contains(&origin) {
        return None;
    }
    let is_preflight = method.eq_ignore_ascii_case("OPTIONS") && requested_method.is_some();
    Some(CorsPolicy {
        allow_origin: Some(origin.to_string()),
        is_preflight,
    })
}

/// Build the CORS response headers for a resolved [`CorsPolicy`].
///
/// Always includes `Vary: Origin` so caches do not serve one origin's
/// response to another. Preflight responses additionally advertise the
/// allowed methods, headers, and preflight cache lifetime.
pub fn cors_headers(policy: &CorsPolicy) -> Vec<(String, String)> {
    let mut headers = vec![
        ("Vary".to_string(), "Origin".to_string()),
        (
            "Access-Control-Allow-Origin".to_string(),
            policy.allow_origin.clone().unwrap_or_default(),
        ),
    ];
    if policy.is_preflight {
        headers.push((
            "Access-Control-Allow-Methods".to_string(),
            CORS_ALLOWED_METHODS.join(", "),
        ));
        headers.push((
            "Access-Control-Allow-Headers".to_string(),
            CORS_ALLOWED_HEADERS.join(", "),
        ));
        headers.push((
            "Access-Control-Max-Age".to_string(),
            CORS_MAX_AGE_SECS.to_string(),
        ));
    }
    headers
}

/// A single recorded health sample.
#[derive(Debug, Clone)]
pub struct HealthSample {
    /// Unix timestamp (seconds) when the sample was recorded.
    pub timestamp: u64,
    /// API response time in milliseconds.
    pub response_time_ms: f64,
    /// Whether the request resulted in an error.
    pub is_error: bool,
    /// Error type label, if this sample was an error.
    pub error_type: Option<String>,
    /// Database query time in milliseconds.
    pub db_time_ms: f64,
    /// Whether the database query failed.
    pub db_error: bool,
}

/// Aggregated health metrics over a window of samples.
#[derive(Debug, Clone, Default)]
pub struct HealthMetrics {
    pub uptime_secs: u64,
    pub total_requests: u64,
    pub error_count: u64,
    pub error_rate: f64,
    pub avg_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub avg_db_time_ms: f64,
    pub db_error_count: u64,
    pub error_types: Vec<(String, u64)>,
}

/// Thresholds that, when exceeded, trigger a service degradation alert.
#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub max_error_rate: f64,
    pub max_avg_response_time_ms: f64,
    pub max_db_time_ms: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            max_error_rate: 0.05,
            max_avg_response_time_ms: 500.0,
            max_db_time_ms: 250.0,
        }
    }
}

/// An alert raised when a threshold is breached.
#[derive(Debug, Clone)]
pub struct Alert {
    pub metric: String,
    pub message: String,
}

/// Thread-safe metrics collector with bounded 30-day history.
#[derive(Clone)]
pub struct MetricsCollector {
    inner: Arc<Mutex<CollectorState>>,
}

struct CollectorState {
    samples: VecDeque<HealthSample>,
    started_at: u64,
    /// Cached aggregation result and the sample count it was computed from.
    cached_metrics: Option<(usize, HealthMetrics)>,
    /// Timestamp of the last cached aggregation, for TTL-based reuse.
    cached_at: u64,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs()
}

impl MetricsCollector {
    /// Create a new collector with an empty history.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(CollectorState {
                samples: VecDeque::new(),
                started_at: now_secs(),
                cached_metrics: None,
                cached_at: 0,
            })),
        }
    }

    /// Record a request sample, pruning anything older than the retention window.
    pub fn record(&self, sample: HealthSample) {
        let mut state = self.inner.lock().expect("metrics lock poisoned");
        state.samples.push_back(sample);
        let cutoff = now_secs().saturating_sub(RETENTION_SECS);
        while let Some(front) = state.samples.front() {
            if front.timestamp < cutoff {
                state.samples.pop_front();
            } else {
                break;
            }
        }
        // Invalidate the cached aggregation so the next read reflects new data.
        state.cached_metrics = None;
    }

    /// Convenience helper to record a request outcome.
    pub fn record_request(
        &self,
        response_time_ms: f64,
        error_type: Option<&str>,
        db_time_ms: f64,
        db_error: bool,
    ) {
        self.record(HealthSample {
            timestamp: now_secs(),
            response_time_ms,
            is_error: error_type.is_some(),
            error_type: error_type.map(|s| s.to_string()),
            db_time_ms,
            db_error,
        });
    }

    /// Compute aggregated metrics over all retained samples.
    ///
    /// Results are cached for [`DASHBOARD_CACHE_TTL_SECS`] so repeated dashboard
    /// loads reuse the previous aggregation instead of rescanning the buffer.
    pub fn metrics(&self) -> HealthMetrics {
        let mut state = self.inner.lock().expect("metrics lock poisoned");
        let now = now_secs();
        if let Some((count, cached)) = &state.cached_metrics {
            if *count == state.samples.len()
                && now.saturating_sub(state.cached_at) < DASHBOARD_CACHE_TTL_SECS
            {
                let mut metrics = cached.clone();
                metrics.uptime_secs = now.saturating_sub(state.started_at);
                return metrics;
            }
        }

        let mut metrics = HealthMetrics {
            uptime_secs: now.saturating_sub(state.started_at),
            ..Default::default()
        };
        if state.samples.is_empty() {
            state.cached_metrics = Some((0, metrics.clone()));
            state.cached_at = now;
            return metrics;
        }

        let mut response_times: Vec<f64> = Vec::with_capacity(state.samples.len());
        let mut error_types: std::collections::HashMap<String, u64> =
            std::collections::HashMap::new();
        let mut total_response = 0.0;
        let mut total_db = 0.0;

        for sample in &state.samples {
            metrics.total_requests += 1;
            total_response += sample.response_time_ms;
            total_db += sample.db_time_ms;
            response_times.push(sample.response_time_ms);
            if sample.is_error {
                metrics.error_count += 1;
                if let Some(kind) = &sample.error_type {
                    *error_types.entry(kind.clone()).or_insert(0) += 1;
                }
            }
            if sample.db_error {
                metrics.db_error_count += 1;
            }
        }

        let count = metrics.total_requests as f64;
        metrics.avg_response_time_ms = total_response / count;
        metrics.avg_db_time_ms = total_db / count;
        metrics.error_rate = metrics.error_count as f64 / count;

        response_times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let idx = ((response_times.len() as f64) * 0.95).ceil() as usize;
        let idx = idx.saturating_sub(1).min(response_times.len() - 1);
        metrics.p95_response_time_ms = response_times[idx];

        let mut error_types: Vec<(String, u64)> = error_types.into_iter().collect();
        error_types.sort_by(|a, b| b.1.cmp(&a.1));
        metrics.error_types = error_types;

        state.cached_metrics = Some((state.samples.len(), metrics.clone()));
        state.cached_at = now;
        metrics
    }

    /// Evaluate current metrics against thresholds, returning any alerts.
    pub fn check_alerts(&self, thresholds: &AlertThresholds) -> Vec<Alert> {
        let metrics = self.metrics();
        let mut alerts = Vec::new();
        if metrics.total_requests == 0 {
            return alerts;
        }
        if metrics.error_rate > thresholds.max_error_rate {
            alerts.push(Alert {
                metric: "error_rate".to_string(),
                message: format!(
                    "Error rate {:.2}% exceeds threshold {:.2}%",
                    metrics.error_rate * 100.0,
                    thresholds.max_error_rate * 100.0
                ),
            });
        }
        if metrics.avg_response_time_ms > thresholds.max_avg_response_time_ms {
            alerts.push(Alert {
                metric: "response_time".to_string(),
                message: format!(
                    "Avg response time {:.0}ms exceeds threshold {:.0}ms",
                    metrics.avg_response_time_ms,
                    thresholds.max_avg_response_time_ms
                ),
            });
        }
        if metrics.avg_db_time_ms > thresholds.max_db_time_ms {
            alerts.push(Alert {
                metric: "db_time".to_string(),
                message: format!(
                    "Avg DB time {:.0}ms exceeds threshold {:.0}ms",
                    metrics.avg_db_time_ms,
                    thresholds.max_db_time_ms
                ),
            });
        }
        alerts
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
