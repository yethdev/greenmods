//! Lightweight abuse throttling middleware.

use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{HeaderValue, Method, Request, StatusCode, header::RETRY_AFTER},
    middleware::Next,
    response::{IntoResponse, Response},
};
use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    net::{IpAddr, SocketAddr},
    sync::Mutex,
    time::{Duration, Instant},
};

const MAX_KEYS: usize = 10_000;

lazy_static! {
    static ref LIMITS: Mutex<HashMap<BucketKey, Window>> = Mutex::new(HashMap::new());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Scope {
    Auth,
    Upload,
    Write,
    Search,
    Download,
    General,
}

#[derive(Debug, Clone, Copy, Eq)]
struct BucketKey {
    ip: IpAddr,
    scope: Scope,
}

#[derive(Debug, Clone, Copy)]
struct Window {
    count: u32,
    reset_at: Instant,
}

impl PartialEq for BucketKey {
    fn eq(&self, other: &Self) -> bool {
        self.ip == other.ip && self.scope == other.scope
    }
}

impl Hash for BucketKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ip.hash(state);
        self.scope.hash(state);
    }
}

/// Apply IP-scoped rate limits to common bot and abuse targets.
pub async fn abuse_guard(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let scope = classify(req.method(), req.uri().path());
    let (max, window) = policy(scope);
    let now = Instant::now();
    let key = BucketKey {
        ip: addr.ip(),
        scope,
    };

    let retry_after = {
        let mut limits = LIMITS.lock().unwrap();

        if limits.len() > MAX_KEYS {
            limits.retain(|_, window| window.reset_at > now);
        }

        let entry = limits.entry(key).or_insert(Window {
            count: 0,
            reset_at: now + window,
        });

        if now >= entry.reset_at {
            entry.count = 0;
            entry.reset_at = now + window;
        }

        if entry.count >= max {
            Some(entry.reset_at.saturating_duration_since(now))
        } else {
            entry.count += 1;
            None
        }
    };

    if let Some(retry_after) = retry_after {
        return too_many(retry_after);
    }

    next.run(req).await
}

fn classify(method: &Method, path: &str) -> Scope {
    if path.starts_with("/api/v1/auth") {
        return Scope::Auth;
    }

    if path.contains("/download/") {
        return Scope::Download;
    }

    if path == "/api/v1/projects/search" {
        return Scope::Search;
    }

    if method != Method::GET {
        if path.contains("/versions") || path.contains("/gallery") {
            Scope::Upload
        } else {
            Scope::Write
        }
    } else {
        Scope::General
    }
}

fn policy(scope: Scope) -> (u32, Duration) {
    match scope {
        Scope::Auth => (20, Duration::from_secs(10 * 60)),
        Scope::Upload => (40, Duration::from_secs(60 * 60)),
        Scope::Write => (90, Duration::from_secs(10 * 60)),
        Scope::Search => (180, Duration::from_secs(60)),
        Scope::Download => (600, Duration::from_secs(60)),
        Scope::General => (360, Duration::from_secs(60)),
    }
}

fn too_many(retry_after: Duration) -> Response {
    let mut res = (
        StatusCode::TOO_MANY_REQUESTS,
        "Too many requests. Please wait and try again.",
    )
        .into_response();

    let secs = retry_after.as_secs().max(1).to_string();

    res.headers_mut()
        .insert(RETRY_AFTER, HeaderValue::from_str(&secs).unwrap());

    res
}
