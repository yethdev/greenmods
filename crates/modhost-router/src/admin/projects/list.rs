//! The (admin) list projects route.

use axum::{Json, extract::State, http::HeaderMap};
use axum_extra::extract::CookieJar;
use modhost_auth::get_user_from_req;
use modhost_core::{AppError, Result};
use modhost_search::{MeiliProject, get_all_projects};
use modhost_server_core::state::AppState;

/// List All Projects
///
/// Get a list of all projects.
#[utoipa::path(
    get,
    path = "/projects/list",
    tag = "Admin",
    responses(
        (status = 200, description = "Fetched!", body = Vec<MeiliProject>),
        (status = INTERNAL_SERVER_ERROR, description = "An internal error occured!"),
    ),
    security(
        ("api_auth_token" = []),
    ),
)]
#[debug_handler]
pub async fn list_handler(
    jar: CookieJar,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<Vec<MeiliProject>>> {
    let user = get_user_from_req(&jar, &headers, &state.db).await?;

    if !user.admin {
        return Err(AppError::NoAccess);
    }

    Ok(Json(get_all_projects(&state.db).await?))
}
