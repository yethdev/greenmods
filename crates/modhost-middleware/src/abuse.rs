//! Lightweight abuse throttling middleware.

use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{
        HeaderValue, Method, Request, StatusCode,
        header::{AUTHORIZATION, FORWARDED, RETRY_AFTER},
    },
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
    Meta,
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
    if should_bypass(&req, addr.ip()) {
        return next.run(req).await;
    }

    let client_ip = resolve_client_ip(&req, addr.ip());
    let scope = classify(req.method(), req.uri().path());
    let (max, window) = policy(scope);
    let now = Instant::now();
    let key = BucketKey {
        ip: client_ip,
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

fn should_bypass(req: &Request<Body>, ip: IpAddr) -> bool {
    if !ip.is_loopback() || *req.method() == Method::GET {
        return false;
    }

    req.headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.trim_start().starts_with("Bearer "))
}

fn resolve_client_ip(req: &Request<Body>, socket_ip: IpAddr) -> IpAddr {
    if !socket_ip.is_loopback() {
        return socket_ip;
    }

    req.headers()
        .get("cf-connecting-ip")
        .and_then(|value| value.to_str().ok())
        .and_then(parse_ip_token)
        .or_else(|| {
            req.headers()
                .get("x-forwarded-for")
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.split(',').find_map(parse_ip_token))
        })
        .or_else(|| {
            req.headers()
                .get(FORWARDED)
                .and_then(|value| value.to_str().ok())
                .and_then(parse_forwarded_for)
        })
        .unwrap_or(socket_ip)
}

fn parse_forwarded_for(value: &str) -> Option<IpAddr> {
    value.split(',').find_map(|entry| {
        entry.split(';').find_map(|part| {
            let part = part.trim();
            part.strip_prefix("for=").and_then(parse_ip_token)
        })
    })
}

fn parse_ip_token(value: &str) -> Option<IpAddr> {
    let value = value.trim().trim_matches('"');

    if value.is_empty() || value.eq_ignore_ascii_case("unknown") {
        return None;
    }

    if let Some(stripped) = value.strip_prefix('[') {
        let end = stripped.find(']')?;
        return stripped[..end].parse().ok();
    }

    value
        .parse::<IpAddr>()
        .ok()
        .or_else(|| value.parse::<SocketAddr>().ok().map(|addr| addr.ip()))
        .or_else(|| {
            value.rsplit_once(':').and_then(|(host, port)| {
                port.parse::<u16>()
                    .ok()
                    .and_then(|_| host.parse::<IpAddr>().ok())
            })
        })
}

fn classify(method: &Method, path: &str) -> Scope {
    if path.starts_with("/api/v1/auth") {
        return Scope::Auth;
    }

    if path.starts_with("/api/v1/meta") {
        return Scope::Meta;
    }

    if path == "/api/v1/projects/search" {
        return Scope::Search;
    }

    if path.contains("/download/") {
        return Scope::Download;
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
        Scope::Search => (1_200, Duration::from_secs(60)),
        Scope::Meta => (3_600, Duration::from_secs(60)),
        Scope::Download => (600, Duration::from_secs(60)),
        Scope::General => (2_400, Duration::from_secs(60)),
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
