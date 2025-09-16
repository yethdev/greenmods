//! The version delete route.

use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};
use axum_extra::extract::CookieJar;
use modhost_auth::get_user_from_req;
use modhost_core::Result;
use modhost_db::{
    get_version,
    prelude::{ProjectAuthors, VersionFiles},
    version_files,
};
use modhost_db_util::projects::get_project;
use modhost_server_core::state::AppState;
use object_store::ObjectStore;
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter};

/// Delete Project Version
///
/// Delete a project version
#[utoipa::path(
    delete,
    path = "/{version}",
    tag = "Versions",
    responses(
        (status = 200, description = "Deleted project version!", body = String),
        (status = INTERNAL_SERVER_ERROR, description = "An internal error occured!"),
    ),
    params(
        ("version" = String, Path, description = "The version ID/name/number."),
    ),
    security(
        ("api_auth_token" = []),
    ),
)]
#[debug_handler]
pub async fn delete_handler(
    jar: CookieJar,
    headers: HeaderMap,
    Path((project, version)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Response> {
    let user = get_user_from_req(&jar, &headers, &state.db).await?;
    let pkg = get_project(project, &state.db).await?;
    let ver = get_version(pkg.id, version, &state.db).await?;
    let files = ver.find_related(VersionFiles).all(&state.db).await?;
    let authors = pkg.find_related(ProjectAuthors).all(&state.db).await?;

    if !authors.iter().any(|v| v.user_id == user.id) && !user.admin {
        return Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::empty())?);
    }

    for file in files {
        let all_referencing = VersionFiles::find()
            .filter(version_files::Column::S3Id.eq(file.s3_id.clone()))
            .all(&state.db)
            .await?;

        if all_referencing.len() <= 1 {
            state
                .buckets
                .projects
                .delete(&format!("/{}", file.s3_id).into())
                .await?;
        }

        // We don't manually delete the file here because CASCADE will take care of it.
    }

    ver.delete(&state.db).await?;
    state.search.update_project(pkg.id, &state.db).await?;

    Ok(Response::builder().body(Body::new(
        "Deleted project version successfully!".to_string(),
    ))?)
}
