//! The (admin) list projects route.

use axum::{Json, extract::State, http::HeaderMap};
use axum_extra::extract::CookieJar;
use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use itertools::Itertools;
use modhost_auth::get_user_from_req;
use modhost_core::{AppError, Result};
use modhost_db::{
    Project, ProjectVersion, User, project_authors, project_versions, projects, users,
};
use modhost_search::MeiliProject;
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
    let mut conn = state.pool.get().await?;
    let user = get_user_from_req(&jar, &headers, &mut conn).await?;

    if !user.admin {
        return Err(AppError::NoAccess);
    }

    Ok(Json(
        projects::table
            .inner_join(project_authors::table.inner_join(users::table))
            .inner_join(project_versions::table)
            .select((
                Project::as_select(),
                User::as_select(),
                ProjectVersion::as_select(),
            ))
            .load::<(Project, User, ProjectVersion)>(&mut conn)
            .await?
            .into_iter()
            .into_group_map_by(|v: &(Project, User, ProjectVersion)| v.0.clone())
            .into_iter()
            .map(|v: (Project, Vec<(Project, User, ProjectVersion)>)| {
                (
                    v.0,
                    v.1.into_iter()
                        .map(|v| (v.1, v.2))
                        .unzip::<User, ProjectVersion, Vec<User>, Vec<ProjectVersion>>(),
                )
            })
            .map(|v| MeiliProject::from_data(v.0, v.1.0, v.1.1))
            .collect_vec(),
    ))
}
