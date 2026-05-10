//! The collection update route.

use super::{
    bad_request, resolve_project_ids, validate_collection_description, validate_collection_name,
    validate_collection_readme, validate_slug,
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
use modhost_db::{
    ProjectCollectionData, ProjectVisibility,
    prelude::ProjectCollections,
    project_collections,
};
use modhost_db_util::collections::{get_collection, get_full_collection};
use modhost_server_core::state::AppState;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel,
    PaginatorTrait, QueryFilter,
};

/// A partial collection for updating a collection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, ToResponse)]
pub struct PartialCollection {
    /// The collection slug.
    #[serde(default)]
    pub slug: Option<String>,

    /// The collection name.
    #[serde(default)]
    pub name: Option<String>,

    /// The collection summary.
    #[serde(default)]
    pub description: Option<String>,

    /// The markdown readme.
    #[serde(default)]
    pub readme: Option<String>,

    /// The collection project ids or slugs.
    #[serde(default)]
    pub projects: Option<Vec<String>>,

    /// The visibility.
    #[serde(default)]
    pub visibility: Option<ProjectVisibility>,
}

/// Update Collection
///
/// Update a project collection.
#[utoipa::path(
    patch,
    path = "/{id}",
    tag = "Collections",
    request_body(content = PartialCollection, description = "The collection fields to update"),
    responses(
        (status = 200, description = "Collection updated successfully!", body = ProjectCollectionData),
        (status = INTERNAL_SERVER_ERROR, description = "An internal error occured!"),
    ),
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
    Json(data): Json<PartialCollection>,
) -> Result<Response> {
    let user = get_user_from_req(&jar, &headers, &state.db).await?;
    let collection = get_collection(id, &state.db).await?;

    if !user.admin && collection.owner_id != user.id {
        return Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::empty())?);
    }

    let mut collection = collection.into_active_model();

    if let Some(slug) = data.slug {
        let slug = slug.trim().to_ascii_lowercase();

        if let Some(err) = validate_slug(&slug) {
            return bad_request(err);
        }

        let collection_id = collection.id.clone().take().unwrap_or_default();

        if ProjectCollections::find()
            .filter(project_collections::Column::Slug.eq(slug.clone()))
            .filter(project_collections::Column::Id.ne(collection_id))
            .count(&state.db)
            .await?
            > 0
        {
            return bad_request("Collection with that slug already exists!");
        }

        collection.slug = Set(slug);
    }

    if let Some(name) = data.name {
        let name = name.trim().to_string();

        if let Some(err) = validate_collection_name(&name) {
            return bad_request(err);
        }

        collection.name = Set(name);
    }

    if let Some(description) = data.description {
        let description = description.trim().to_string();

        if let Some(err) = validate_collection_description(&description) {
            return bad_request(err);
        }

        collection.description = Set(description);
    }

    if let Some(readme) = data.readme {
        let readme = readme.trim().to_string();

        if let Some(err) = validate_collection_readme(&readme) {
            return bad_request(err);
        }

        collection.readme = Set(readme);
    }

    if let Some(project_refs) = data.projects {
        let project_ids = match resolve_project_ids(&project_refs, &user, &state).await {
            Ok(project_ids) => project_ids,
            Err(err) => return bad_request(err),
        };

        collection.project_ids = Set(project_ids);
    }

    if let Some(visibility) = data.visibility {
        collection.visibility = Set(visibility);
    }

    let collection = collection.update(&state.db).await?;

    Ok(Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::new(serde_json::to_string(
            &get_full_collection(collection.id.to_string(), Some(&user), &state.db).await?,
        )?))?)
}