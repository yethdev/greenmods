//! The version update route.

use axum::{
    Json,
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use modhost_auth::get_user_from_req;
use modhost_core::Result;
use modhost_db::{ProjectVersion, get_version, prelude::ProjectAuthors};
use modhost_db_util::projects::get_project;
use modhost_server_core::state::AppState;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, IntoActiveModel, ModelTrait};
use semver::Version;

/// Information for updaing a project version.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, ToSchema, ToResponse, Serialize, Deserialize,
)]
pub struct PartialProjectVersion {
    /// The display name of the version.
    #[serde(default)]
    pub name: Option<String>,

    /// The version number.
    /// This must be a string confirming to the [SemVer](https://semver.org/) standard.
    #[serde(default)]
    pub version_number: Option<String>,

    /// The version changelog.
    #[serde(default)]
    pub changelog: Option<String>,

    /// The mod loaders this version works on.
    #[serde(default)]
    pub loaders: Option<Vec<String>>,

    /// The game versions this version works on.
    #[serde(default)]
    pub game_versions: Option<Vec<String>>,
}

/// Update Project Version
///
/// Update information about project version
#[utoipa::path(
    patch,
    path = "/{version}",
    tag = "Versions",
    responses(
        (status = 200, description = "Updated project version!", body = ProjectVersion),
        (status = INTERNAL_SERVER_ERROR, description = "An internal error occured!"),
    ),
    params(
        ("version" = String, Path, description = "The version ID/name/number."),
    ),
    request_body(content = PartialProjectVersion, description = "The information to update"),
    security(
        ("api_auth_token" = []),
    ),
)]
#[debug_handler]
pub async fn update_handler(
    jar: CookieJar,
    headers: HeaderMap,
    Path((project, version)): Path<(String, String)>,
    State(state): State<AppState>,
    Json(data): Json<PartialProjectVersion>,
) -> Result<Response> {
    let user = get_user_from_req(&jar, &headers, &state.db).await?;
    let pkg = get_project(project, &state.db).await?;
    let ver = get_version(pkg.id, version, &state.db).await?;
    let authors = pkg.find_related(ProjectAuthors).all(&state.db).await?;

    if !authors.iter().any(|v| v.user_id == user.id) && !user.admin {
        return Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::empty())?);
    }

    if let Some(ver_num) = &data.version_number {
        Version::parse(ver_num)?;
    }

    let mut ver = ver.into_active_model();

    if let Some(name) = data.name {
        ver.name = Set(name);
    }

    if let Some(num) = data.version_number {
        ver.version_number = Set(num);
    }

    if let Some(changelog) = data.changelog {
        ver.changelog = Set(Some(changelog));
    }

    if let Some(loaders) = data.loaders {
        ver.loaders = Set(loaders);
    }

    if let Some(vers) = data.game_versions {
        ver.game_versions = Set(vers);
    }

    ver.updated_at = Set(Utc::now().naive_utc());

    let ver = ver.update(&state.db).await?;

    state.search.update_project(pkg.id, &state.db).await?;

    Ok(Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::new(serde_json::to_string(&ver)?))?)
}
