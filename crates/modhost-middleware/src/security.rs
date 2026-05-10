//! Security header middleware.

use axum::{body::Body, http::Request, middleware::Next, response::Response};

/// Add conservative response headers that do not interfere with the Svelte app.
pub async fn security_headers(req: Request<Body>, next: Next) -> Response {
    let forwarded_proto = req
        .headers()
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok());
    let is_https = req.uri().scheme_str() == Some("https")
        || forwarded_proto
            .map(|value| value.eq_ignore_ascii_case("https"))
            .unwrap_or(false);
    let mut res = next.run(req).await;
    let headers = res.headers_mut();

    headers.insert("x-content-type-options", "nosniff".parse().unwrap());
    headers.insert("x-frame-options", "DENY".parse().unwrap());
    headers.insert("x-permitted-cross-domain-policies", "none".parse().unwrap());
    headers.insert(
        "referrer-policy",
        "strict-origin-when-cross-origin".parse().unwrap(),
    );
    headers.insert("origin-agent-cluster", "?1".parse().unwrap());
    headers.insert("cross-origin-opener-policy", "same-origin".parse().unwrap());
    headers.insert("cross-origin-resource-policy", "same-site".parse().unwrap());
    headers.insert(
        "permissions-policy",
        "camera=(), microphone=(), geolocation=(), payment=()"
            .parse()
            .unwrap(),
    );

    if is_https {
        headers.insert(
            "strict-transport-security",
            "max-age=31536000; includeSubDomains".parse().unwrap(),
        );
    }

    res
}
