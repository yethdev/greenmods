//! The project update route.

use super::{
    bad_request, clean_link, clean_tags, validate_project_description, validate_project_name,
    validate_project_readme, validate_project_tags,
};
use axum::{
    Json,
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};
use axum_extra::extract::CookieJar;
use modhost_auth::get_user_from_req;
use modhost_core::Result;
use modhost_db::{ProjectData, ProjectVisibility, prelude::ProjectAuthors};
use modhost_db_util::projects::{get_full_project, get_project};
use modhost_server_core::state::AppState;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, IntoActiveModel, ModelTrait};

/// A partial project for updating a project.
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToSchema, ToResponse, Serialize, Deserialize)]
pub struct PartialProject {
    /// The display name of the project.
    #[serde(default)]
    pub name: Option<String>,

    /// The project's readme.
    #[serde(default)]
    pub readme: Option<String>,

    /// A short description of the project.
    #[serde(default)]
    pub description: Option<String>,

    /// The project's source code URL.
    #[serde(default)]
    pub source: Option<String>,

    /// The project's issues URL.
    #[serde(default)]
    pub issues: Option<String>,

    /// The project's wiki URL.
    #[serde(default)]
    pub wiki: Option<String>,

    /// The project's visibility.
    #[serde(default)]
    pub visibility: Option<ProjectVisibility>,

    /// The project's license.
    #[serde(default)]
    pub license: Option<String>,

    /// The project's tags.
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

/// Update Project
///
/// Update a project's information.
#[utoipa::path(
    patch,
    path = "/{id}",
    tag = "Projects",
    responses(
        (status = 200, description = "Project updated successfully!", body = ProjectData),
        (status = INTERNAL_SERVER_ERROR, description = "Error: project might not exist, or another error occured!"),
    ),
    request_body(content = PartialProject, description = "The information to update"),
    security(
        ("api_auth_token" = []),
    ),
)]
#[debug_handler]
pub async fn update_handler(
    jar: CookieJar,
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(data): Json<PartialProject>,
) -> Result<Response> {
    let user = get_user_from_req(&jar, &headers, &state.db).await?;
    let pkg = get_project(id, &state.db).await?;
    let authors = pkg.find_related(ProjectAuthors).all(&state.db).await?;

    if !authors.iter().any(|v| v.user_id == user.id) && !user.admin {
        return Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::empty())?);
    }

    let mut pkg = pkg.into_active_model();

    if let Some(name) = data.name {
        let name = name.trim().to_string();

        if let Some(err) = validate_project_name(&name) {
            return bad_request(err);
        }

        pkg.name = Set(name);
    }

    if let Some(readme) = data.readme {
        let readme = readme.trim().to_string();

        if let Some(err) = validate_project_readme(&readme) {
            return bad_request(err);
        }

        pkg.readme = Set(readme);
    }

    if let Some(description) = data.description {
        let description = description.trim().to_string();

        if let Some(err) = validate_project_description(&description) {
            return bad_request(err);
        }

        pkg.description = Set(description);
    }

    if let Some(source) = data.source {
        pkg.source = Set(match clean_link("Source", Some(source)) {
            Ok(value) => value,
            Err(err) => return bad_request(err),
        });
    }

    if let Some(issues) = data.issues {
        pkg.issues = Set(match clean_link("Issue tracker", Some(issues)) {
            Ok(value) => value,
            Err(err) => return bad_request(err),
        });
    }

    if let Some(wiki) = data.wiki {
        pkg.wiki = Set(match clean_link("Wiki", Some(wiki)) {
            Ok(value) => value,
            Err(err) => return bad_request(err),
        });
    }

    if let Some(visibility) = data.visibility {
        pkg.visibility = Set(visibility);
    }

    if let Some(license) = data.license {
        if license.is_empty() {
            pkg.license = Set(None);
        } else {
            pkg.license = Set(Some(license));
        }
    }

    if let Some(tags) = data.tags {
        let tags = clean_tags(&tags);

        if let Some(err) = validate_project_tags(&tags, &state) {
            return bad_request(err);
        }

        pkg.tags = Set(tags);
    }

    let pkg = pkg.update(&state.db).await?;

    state.search.update_project(pkg.id, &state.db).await?;

    Ok(Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::new(serde_json::to_string(
            &get_full_project(pkg.id.to_string(), &state.db).await?,
        )?))?)
}
