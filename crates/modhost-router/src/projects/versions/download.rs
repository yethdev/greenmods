//! The version download route.

use axum::{
    extract::{Path, State},
    http::HeaderMap,
};
use axum_extra::extract::CookieJar;
use modhost_auth::get_user_from_req;
use modhost_core::AppError;
use modhost_core::Result;
use modhost_db::{ProjectVisibility, get_version};
use modhost_db_util::{projects::get_full_project, vers::get_version_file};
use modhost_server_core::state::AppState;
use object_store::ObjectStore;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, IntoActiveModel};

/// Download Project Version
///
/// Download a specific project version
#[utoipa::path(
    get,
    path = "/{version}/download/{file}",
    tag = "Versions",
    responses(
        (status = 307, description = "Redirecting to download"),
        (status = INTERNAL_SERVER_ERROR, description = "An internal error occured!"),
    ),
    params(
        ("version" = String, Path, description = "The version ID/name/number."),
        ("file" = String, Path, description = "The file ID/name."),
    ),
)]
#[debug_handler]
pub async fn download_handler(
    jar: CookieJar,
    headers: HeaderMap,
    Path((project, version, file)): Path<(String, String, String)>,
    State(state): State<AppState>,
) -> Result<Vec<u8>> {
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

    let ver = get_version(pkg.id, version, &state.db).await?;
    let file = get_version_file(ver.id, file, &state.db).await?;

    let pkg_dl = pkg.downloads;
    let ver_dl = ver.downloads;
    let mut pkg = pkg.into_project().into_active_model();

    pkg.downloads = Set(pkg_dl + 1);

    let mut ver = ver.into_active_model();

    ver.downloads = Set(ver_dl + 1);

    let pkg = pkg.update(&state.db).await?;

    ver.update(&state.db).await?;

    state.search.update_project(pkg.id, &state.db).await?;

    let bytes = state
        .buckets
        .projects
        .get(&format!("/{}", file.s3_id).into())
        .await?
        .bytes()
        .await?
        .to_vec();

    Ok(bytes)
}
