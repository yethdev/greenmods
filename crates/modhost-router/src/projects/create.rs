//! The project create route.

use axum::{
    Json,
    body::Body,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Response,
};
use axum_extra::extract::CookieJar;
use modhost_auth::get_user_from_req;
use modhost_core::Result;
use modhost_db::{ProjectData, ProjectVisibility, prelude::Projects, project_authors, projects};
use modhost_db_util::projects::get_full_project;
use modhost_server_core::state::AppState;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
};

fn default_vis() -> ProjectVisibility {
    ProjectVisibility::Public
}

/// A model for creating a new project.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, ToResponse)]
pub struct NewProject {
    /// The project's URL slug.
    pub slug: String,

    /// The project's name.
    pub name: String,

    /// The project's README.
    pub readme: String,

    /// A short description of the project.
    pub description: String,

    /// An optional link to the project's source code.
    #[serde(default)]
    pub source: Option<String>,

    /// An optional link to the project's issue tracker.
    #[serde(default)]
    pub issues: Option<String>,

    /// An optional link to the project's wiki.
    #[serde(default)]
    pub wiki: Option<String>,

    /// The visibility of a project. Optional. Defaults to public.
    #[serde(default = "default_vis")]
    pub visibility: ProjectVisibility,

    /// The license the project is under.
    #[serde(default)]
    pub license: Option<String>,

    /// A list of tags for this project.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Create Project
///
/// Create a project
#[utoipa::path(
    put,
    path = "/",
    tag = "Projects",
    responses(
        (status = 200, description = "Project created successfully!", body = ProjectData),
        (status = 401, description = "Project already exists!"),
        (status = INTERNAL_SERVER_ERROR, description = "An internal error occured!"),
    ),
    request_body(content = NewProject, description = "Information about the project to create"),
    security(
        ("api_auth_token" = []),
    ),
)]
#[debug_handler]
pub async fn create_handler(
    jar: CookieJar,
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(body): Json<NewProject>,
) -> Result<Response> {
    let user = get_user_from_req(&jar, &headers, &state.db).await?;

    if body.slug.is_empty() {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::new("Slug must not be empty!".to_string()))?);
    }

    if Projects::find()
        .filter(projects::Column::Slug.eq(body.slug.clone()))
        .count(&state.db)
        .await?
        > 0
    {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::new(
                "Project with that slug already exists!".to_string(),
            ))?);
    }

    let pkg = projects::ActiveModel {
        slug: Set(body.slug),
        name: Set(body.name),
        readme: Set(body.readme),
        description: Set(body.description),
        source: Set(body.source),
        issues: Set(body.issues),
        wiki: Set(body.wiki),
        visibility: Set(body.visibility),
        license: Set(body.license),
        tags: Set(body.tags),
        ..Default::default()
    };

    let pkg = pkg.insert(&state.db).await?;

    project_authors::ActiveModel {
        project: Set(pkg.id),
        user_id: Set(user.id),
    }
    .insert(&state.db)
    .await?;

    state.search.update_project(pkg.id, &state.db).await?;

    Ok(Response::builder().body(Body::new(serde_json::to_string(
        &get_full_project(pkg.id.to_string(), &state.db).await?,
    )?))?)
}
