//! The version info route.

use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use axum_extra::extract::CookieJar;
use modhost_auth::get_user_from_req;
use modhost_core::AppError;
use modhost_core::Result;
use modhost_db::{ProjectVersion, ProjectVersionData, ProjectVisibility};
use modhost_db_util::{projects::get_full_project, vers::get_full_version};
use modhost_server_core::state::AppState;

/// Get Project Version
///
/// Get information about a specific project version
#[utoipa::path(
    get,
    path = "/{version}",
    tag = "Versions",
    responses(
        (status = 200, description = "Found project version!", body = ProjectVersion),
        (status = INTERNAL_SERVER_ERROR, description = "An internal error occured!"),
    ),
    params(
        ("version" = String, Path, description = "The version ID/name/number."),
    ),
)]
#[debug_handler]
pub async fn info_handler(
    jar: CookieJar,
    headers: HeaderMap,
    Path((project, version)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<ProjectVersionData>> {
    let pkg = get_full_project(project, &state.db).await?;

    if pkg.visibility == ProjectVisibility::Private {
        match get_user_from_req(&jar, &headers, &state.db).await {
            Ok(user) => {
                if !pkg.authors.iter().any(|v| v.github_id == user.github_id) && !user.admin {
                    return Err(AppError::NotFound);
                }
            }

            Err(_) => return Err(AppError::NotFound),
        }
    }

    Ok(Json(
        get_full_version(&pkg.into_project(), version, &state.db).await?,
    ))
}
