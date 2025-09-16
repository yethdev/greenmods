//! The add author route.

use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};
use axum_extra::extract::CookieJar;
use modhost_auth::get_user_from_req;
use modhost_core::Result;
use modhost_db::{ProjectData, get_user, prelude::ProjectAuthors, project_authors};
use modhost_db_util::projects::{get_full_project, get_project};
use modhost_server_core::state::AppState;
use sea_orm::{ActiveValue::Set, EntityTrait, ModelTrait};

/// Add Project Author
///
/// Add an author to a project.
#[utoipa::path(
    put,
    path = "/",
    tag = "Projects",
    responses(
        (status = 200, description = "Project updated successfully!", body = ProjectData),
        (status = UNAUTHORIZED, description = "You do not have access to modify this project!"),
        (status = BAD_REQUEST, description = "The user is already a member of the project!"),
        (status = INTERNAL_SERVER_ERROR, description = "Error: project might not exist, or another error occured!"),
    ),
    request_body(content = String, description = "The ID/username of the author to add."),
    security(
        ("api_auth_token" = []),
    ),
)]
#[debug_handler]
pub async fn add_handler(
    jar: CookieJar,
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<AppState>,
    body: String,
) -> Result<Response> {
    let user = get_user_from_req(&jar, &headers, &state.db).await?;
    let pkg = get_project(id, &state.db).await?;
    let authors = pkg.find_related(ProjectAuthors).all(&state.db).await?;

    if !authors.iter().any(|v| v.user_id == user.id) && !user.admin {
        return Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::empty())?);
    }

    let to_add = get_user(body, &state.db).await?;

    if authors.iter().any(|v| v.user_id == to_add.id) {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::new(
                "Author is already a member of the project!".to_string(),
            ))?);
    }

    ProjectAuthors::insert(project_authors::ActiveModel {
        project: Set(pkg.id),
        user_id: Set(to_add.id),
    })
    .exec(&state.db)
    .await?;

    state.search.update_project(pkg.id, &state.db).await?;

    Ok(Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::new(serde_json::to_string(
            &get_full_project(pkg.id.to_string(), &state.db).await?,
        )?))?)
}
