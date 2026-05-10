//! Health-related endpoints.

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use modhost_server_core::state::AppState;
use std::{
    fmt::Display,
    time::{Duration, Instant},
};

const HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(2);

/// The outcome of a health check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    /// The check passed.
    Ok,

    /// The check failed.
    Error,
}

/// The health of one dependency.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct DependencyHealth {
    /// The outcome of the dependency check.
    pub status: HealthStatus,

    /// The time spent checking the dependency.
    pub latency_ms: u64,

    /// The dependency error, if the check failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// The dependency breakdown for the health endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct HealthChecks {
    /// The database connection health.
    pub database: DependencyHealth,

    /// The search/index health.
    pub search: DependencyHealth,
}

/// The liveness response body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct LivenessResponse {
    /// The service name.
    pub service: String,

    /// The running crate version.
    pub version: String,

    /// The overall liveness result.
    pub status: HealthStatus,

    /// When the checks were completed.
    pub checked_at: String,
}

/// The health response body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct HealthResponse {
    /// The service name.
    pub service: String,

    /// The running crate version.
    pub version: String,

    /// The overall health result.
    pub status: HealthStatus,

    /// When the checks were completed.
    pub checked_at: String,

    /// The per-dependency checks.
    pub checks: HealthChecks,
}

/// Health
///
/// Report service liveness.
#[utoipa::path(
    get,
    path = "/health",
    tag = "Misc",
    responses(
        (status = 200, description = "The service process is alive.", body = LivenessResponse),
    ),
)]
#[debug_handler]
pub async fn livez_handler(State(_state): State<AppState>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(LivenessResponse {
            service: "greenmods".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            status: HealthStatus::Ok,
            checked_at: Utc::now().to_rfc3339(),
        }),
    )
}

/// Readiness
///
/// Report service dependency readiness.
#[utoipa::path(
    get,
    path = "/ready",
    tag = "Misc",
    responses(
        (status = 200, description = "The service and its dependencies are healthy.", body = HealthResponse),
        (status = SERVICE_UNAVAILABLE, description = "One or more dependencies are unavailable.", body = HealthResponse),
    ),
)]
#[debug_handler]
pub async fn readyz_handler(State(state): State<AppState>) -> impl IntoResponse {
    let (database, search) = tokio::join!(database_health(&state), search_health(&state));
    let healthy = database.status == HealthStatus::Ok && search.status == HealthStatus::Ok;
    let status = if healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let response = HealthResponse {
        service: "greenmods".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        status: if healthy {
            HealthStatus::Ok
        } else {
            HealthStatus::Error
        },
        checked_at: Utc::now().to_rfc3339(),
        checks: HealthChecks { database, search },
    };

    (status, Json(response))
}

async fn database_health(state: &AppState) -> DependencyHealth {
    let started = Instant::now();

    match tokio::time::timeout(HEALTH_CHECK_TIMEOUT, state.db.ping()).await {
        Ok(Ok(())) => healthy(started),
        Ok(Err(error)) => unhealthy(started, error),
        Err(_) => unhealthy(started, "Database ping timed out"),
    }
}

async fn search_health(state: &AppState) -> DependencyHealth {
    let started = Instant::now();
    let url = format!(
        "{}/indexes/{}/stats",
        state.config.meilisearch.url(),
        state.config.meilisearch.project_index,
    );

    let request = oauth2::reqwest::Client::new()
        .get(url)
        .bearer_auth(&state.config.meilisearch.key);

    match tokio::time::timeout(HEALTH_CHECK_TIMEOUT, request.send()).await {
        Ok(Ok(response)) if response.status().is_success() => healthy(started),
        Ok(Ok(response)) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            unhealthy(
                started,
                format!(
                    "Meilisearch stats returned {status}: {}",
                    compact_error_body(&body)
                ),
            )
        }
        Ok(Err(error)) => unhealthy(started, error),
        Err(_) => unhealthy(started, "Meilisearch stats timed out"),
    }
}

fn healthy(started: Instant) -> DependencyHealth {
    DependencyHealth {
        status: HealthStatus::Ok,
        latency_ms: elapsed_ms(started),
        error: None,
    }
}

fn unhealthy(started: Instant, error: impl Display) -> DependencyHealth {
    DependencyHealth {
        status: HealthStatus::Error,
        latency_ms: elapsed_ms(started),
        error: Some(error.to_string()),
    }
}

fn elapsed_ms(started: Instant) -> u64 {
    started.elapsed().as_millis().try_into().unwrap_or(u64::MAX)
}

fn compact_error_body(body: &str) -> String {
    let compact = body.split_whitespace().collect::<Vec<_>>().join(" ");

    if compact.len() <= 160 {
        compact
    } else {
        format!("{}...", &compact[..157])
    }
}