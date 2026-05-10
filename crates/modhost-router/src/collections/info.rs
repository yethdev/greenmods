//! The collection info route.

use super::is_visible_to_viewer;
use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use axum_extra::extract::CookieJar;
use modhost_auth::get_user_from_req;
use modhost_core::{AppError, Result};
use modhost_db::ProjectCollectionData;
use modhost_db_util::collections::get_full_collection;
use modhost_server_core::state::AppState;

/// Get Collection
///
/// Get a collection by id or slug.
#[utoipa::path(
    get,
    path = "/{id}",
    tag = "Collections",
    responses(
        (status = 200, description = "Method returned ok", body = ProjectCollectionData),
        (status = INTERNAL_SERVER_ERROR, description = "An internal error occured!"),
    ),
)]
#[debug_handler]
pub async fn info_handler(
    jar: CookieJar,
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ProjectCollectionData>> {
    let viewer = get_user_from_req(&jar, &headers, &state.db).await.ok();
    let collection = get_full_collection(id, viewer.as_ref(), &state.db).await?;

    if !is_visible_to_viewer(collection.visibility, collection.owner.id, viewer.as_ref()) {
        return Err(AppError::NotFound);
    }

    Ok(Json(collection))
}