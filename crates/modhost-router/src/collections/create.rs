//! The collection create route.

use super::{
    bad_request, resolve_project_ids, validate_collection_description, validate_collection_name,
    validate_collection_readme, validate_slug,
};
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
use modhost_db::{
    ProjectCollectionData, ProjectVisibility,
    prelude::ProjectCollections,
    project_collections,
};
use modhost_db_util::collections::get_full_collection;
use modhost_server_core::state::AppState;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
};

fn default_vis() -> ProjectVisibility {
    ProjectVisibility::Public
}

/// A model for creating a new project collection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, ToResponse)]
pub struct NewCollection {
    /// The collection slug.
    pub slug: String,

    /// The collection name.
    pub name: String,

    /// A short summary of the collection.
    pub description: String,

    /// A markdown readme for the collection.
    pub readme: String,

    /// The projects to include, referenced by project id or slug.
    #[serde(default)]
    pub projects: Vec<String>,

    /// The collection visibility.
    #[serde(default = "default_vis")]
    pub visibility: ProjectVisibility,
}

/// Create Collection
///
/// Create a project collection.
#[utoipa::path(
    put,
    path = "/",
    tag = "Collections",
    request_body(content = NewCollection, description = "Information about the collection to create"),
    responses(
        (status = 200, description = "Collection created successfully!", body = ProjectCollectionData),
        (status = INTERNAL_SERVER_ERROR, description = "An internal error occured!"),
    ),
    security(
        ("api_auth_token" = []),
    ),
)]
#[debug_handler]
pub async fn create_handler(
    jar: CookieJar,
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(body): Json<NewCollection>,
) -> Result<Response> {
    let user = get_user_from_req(&jar, &headers, &state.db).await?;
    let slug = body.slug.trim().to_ascii_lowercase();
    let name = body.name.trim().to_string();
    let description = body.description.trim().to_string();
    let readme = body.readme.trim().to_string();

    if let Some(err) = validate_slug(&slug) {
        return bad_request(err);
    }

    if let Some(err) = validate_collection_name(&name) {
        return bad_request(err);
    }

    if let Some(err) = validate_collection_description(&description) {
        return bad_request(err);
    }

    if let Some(err) = validate_collection_readme(&readme) {
        return bad_request(err);
    }

    let project_ids = match resolve_project_ids(&body.projects, &user, &state).await {
        Ok(project_ids) => project_ids,
        Err(err) => return bad_request(err),
    };

    if ProjectCollections::find()
        .filter(project_collections::Column::Slug.eq(slug.clone()))
        .count(&state.db)
        .await?
        > 0
    {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::new(
                "Collection with that slug already exists!".to_string(),
            ))?);
    }

    let collection = project_collections::ActiveModel {
        owner_id: Set(user.id),
        slug: Set(slug),
        name: Set(name),
        description: Set(description),
        readme: Set(readme),
        project_ids: Set(project_ids),
        visibility: Set(body.visibility),
        ..Default::default()
    }
    .insert(&state.db)
    .await?;

    Ok(Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::new(serde_json::to_string(
            &get_full_collection(collection.id.to_string(), Some(&user), &state.db).await?,
        )?))?)
}