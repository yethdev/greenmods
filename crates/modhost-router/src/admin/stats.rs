//! Admin stats route.

use crate::util::stats::{AdminStats, fetch_stats};
use axum::{Json, extract::State, http::HeaderMap};
use axum_extra::extract::CookieJar;
use modhost_auth::get_user_from_req;
use modhost_core::{AppError, Result};
use modhost_server_core::state::AppState;

/// Stats
///
/// Get statistics about this ModHost instance.
#[utoipa::path(
    get,
    path = "/stats",
    tag = "Admin",
    responses(
        (status = 200, description = "Got stats!", body = AdminStats),
        (status = INTERNAL_SERVER_ERROR, description = "An internal error occured!"),
    ),
    security(
        ("api_auth_token" = []),
    ),
)]
#[debug_handler]
pub async fn stats_handler(
    jar: CookieJar,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<AdminStats>> {
    let user = get_user_from_req(&jar, &headers, &state.db).await?;

    if !user.admin {
        return Err(AppError::NoAccess);
    }

    Ok(Json(
        fetch_stats(
            &state.buckets.projects,
            &state.buckets.gallery,
            &state.search.projects(),
            &state.db,
        )
        .await?,
    ))
}
