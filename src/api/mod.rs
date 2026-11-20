use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Per-endpoint rate limit configuration.
#[derive(Clone, Copy, Debug)]
pub struct RateLimit {
    /// Maximum number of requests allowed within the window.
    pub max_requests: u32,
    /// Length of the sliding window.
    pub window: Duration,
}

impl RateLimit {
    pub const fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            max_requests,
            window,
        }
    }
}

/// Rate limits for each endpoint, configurable per environment.
#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    pub default: RateLimit,
    pub endpoints: HashMap<String, RateLimit>,
    /// Clients whose requests bypass rate limiting entirely.
    pub trusted_clients: Vec<String>,
}

impl RateLimitConfig {
    /// Build a configuration appropriate for the given environment name.
    pub fn for_environment(env: &str) -> Self {
        let (default, endpoints) = match env {
            "production" => (
                RateLimit::new(100, Duration::from_secs(60)),
                HashMap::from([
                    ("login".to_string(), RateLimit::new(5, Duration::from_secs(60))),
                    ("register".to_string(), RateLimit::new(3, Duration::from_secs(60))),
                    ("search".to_string(), RateLimit::new(30, Duration::from_secs(60))),
                ]),
            ),
            "staging" => (
                RateLimit::new(500, Duration::from_secs(60)),
                HashMap::from([
                    ("login".to_string(), RateLimit::new(20, Duration::from_secs(60))),
                    ("register".to_string(), RateLimit::new(10, Duration::from_secs(60))),
                    ("search".to_string(), RateLimit::new(100, Duration::from_secs(60))),
                ]),
            ),
            // Development / test: generous limits to avoid friction.
            _ => (
                RateLimit::new(10_000, Duration::from_secs(60)),
                HashMap::from([
                    ("login".to_string(), RateLimit::new(1_000, Duration::from_secs(60))),
                    ("register".to_string(), RateLimit::new(1_000, Duration::from_secs(60))),
                    ("search".to_string(), RateLimit::new(5_000, Duration::from_secs(60))),
                ]),
            ),
        };

        Self {
            default,
            endpoints,
            trusted_clients: Vec::new(),
        }
    }

    /// Resolve the limit that applies to a given endpoint.
    pub fn limit_for(&self, endpoint: &str) -> RateLimit {
        self.endpoints
            .get(endpoint)
            .copied()
            .unwrap_or(self.default)
    }

    /// Whether the given client identifier is trusted to bypass limits.
    pub fn is_trusted(&self, client_id: &str) -> bool {
        self.trusted_clients.iter().any(|c| c == client_id)
    }
}

/// Result of a rate limit check, carrying the headers to emit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RateLimitOutcome {
    pub allowed: bool,
    pub limit: u32,
    pub remaining: u32,
    /// Seconds until the window resets.
    pub reset: u64,
}

impl RateLimitOutcome {
    /// Headers to attach to the response.
    pub fn headers(&self) -> Vec<(String, String)> {
        vec![
            ("X-RateLimit-Limit".to_string(), self.limit.to_string()),
            ("X-RateLimit-Remaining".to_string(), self.remaining.to_string()),
            ("X-RateLimit-Reset".to_string(), self.reset.to_string()),
        ]
    }
}

struct WindowState {
    window_start: Instant,
    count: u32,
}

/// Tracks request counts per (client, endpoint) pair.
pub struct RateLimiter {
    config: RateLimitConfig,
    state: Mutex<HashMap<(String, String), WindowState>>,
    /// Total number of requests rejected by the limiter.
    violations: Mutex<u64>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            state: Mutex::new(HashMap::new()),
            violations: Mutex::new(0),
        }
    }

    /// Check and record a request for the given client and endpoint.
    pub fn check(&self, client_id: &str, endpoint: &str) -> RateLimitOutcome {
        let limit = self.config.limit_for(endpoint);

        // Trusted clients bypass rate limiting.
        if self.config.is_trusted(client_id) {
            return RateLimitOutcome {
                allowed: true,
                limit: limit.max_requests,
                remaining: limit.max_requests,
                reset: limit.window.as_secs(),
            };
        }

        let now = Instant::now();
        let key = (client_id.to_string(), endpoint.to_string());
        let mut state = self.state.lock().unwrap();
        let entry = state.entry(key).or_insert(WindowState {
            window_start: now,
            count: 0,
        });

        // Reset the window if it has elapsed.
        if now.duration_since(entry.window_start) >= limit.window {
            entry.window_start = now;
            entry.count = 0;
        }

        let reset = limit
            .window
            .saturating_sub(now.duration_since(entry.window_start))
            .as_secs();

        if entry.count >= limit.max_requests {
            drop(state);
            *self.violations.lock().unwrap() += 1;
            return RateLimitOutcome {
                allowed: false,
                limit: limit.max_requests,
                remaining: 0,
                reset,
            };
        }

        entry.count += 1;
        let remaining = limit.max_requests.saturating_sub(entry.count);

        RateLimitOutcome {
            allowed: true,
            limit: limit.max_requests,
            remaining,
            reset,
        }
    }

    /// Number of requests rejected so far (for monitoring/metrics).
    pub fn violation_count(&self) -> u64 {
        *self.violations.lock().unwrap()
    }
}

/// A minimal HTTP response produced by the API layer.
pub struct Response {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

/// Rate limiting middleware: wraps a handler, enforcing limits and
/// attaching rate limit headers (or returning 429 when exceeded).
pub fn rate_limit_middleware<F>(
    limiter: &RateLimiter,
    client_id: &str,
    endpoint: &str,
    handler: F,
) -> Response
where
    F: FnOnce() -> Response,
{
    let outcome = limiter.check(client_id, endpoint);

    if !outcome.allowed {
        let mut headers = outcome.headers();
        headers.push(("Retry-After".to_string(), outcome.reset.to_string()));
        return Response {
            status: 429,
            headers,
            body: "Too Many Requests".to_string(),
        };
    }

    let mut response = handler();
    response.headers.extend(outcome.headers());
    response
}
