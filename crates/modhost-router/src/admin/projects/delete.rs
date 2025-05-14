//! The delete project route.

use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::Response,
};
use axum_extra::extract::CookieJar;
use diesel::{ExpressionMethods, delete};
use diesel_async::RunQueryDsl;
use modhost_auth::get_user_from_req;
use modhost_core::{AppError, Result};
use modhost_db::projects;
use modhost_db_util::projects::get_project;
use modhost_server_core::state::AppState;

/// Delete Project
///
/// Delete a project.
#[utoipa::path(
    delete,
    path = "/projects/{id}",
    tag = "Admin",
    responses(
        (status = 200, description = "Deleted!", body = String),
        (status = INTERNAL_SERVER_ERROR, description = "An internal error occured!"),
    ),
    security(
        ("api_auth_token" = []),
    ),
)]
#[debug_handler]
pub async fn delete_handler(
    jar: CookieJar,
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response> {
    let mut conn = state.pool.get().await?;
    let user = get_user_from_req(&jar, &headers, &mut conn).await?;

    if !user.admin {
        return Err(AppError::NoAccess);
    }

    let id = get_project(id, &mut conn).await?.id;

    delete(projects::table)
        .filter(projects::id.eq(id))
        .execute(&mut conn)
        .await?;

    state.search.delete_project(id).await?;

    Ok(Response::new("Deleted project successfully!".into()))
}
