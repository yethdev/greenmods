//! The collection list route.

use axum::{
    Json,
    extract::State,
    http::HeaderMap,
};
use axum_extra::extract::CookieJar;
use modhost_auth::get_user_from_req;
use modhost_core::Result;
use modhost_db::ProjectCollectionData;
use modhost_db_util::collections::list_collections;
use modhost_server_core::state::AppState;

/// List Collections
///
/// List all collections visible to the current user.
#[utoipa::path(
    get,
    path = "/",
    tag = "Collections",
    responses(
        (status = 200, description = "Method returned ok", body = [ProjectCollectionData]),
        (status = INTERNAL_SERVER_ERROR, description = "An internal error occured!"),
    ),
)]
#[debug_handler]
pub async fn list_handler(
    jar: CookieJar,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<Vec<ProjectCollectionData>>> {
    let viewer = get_user_from_req(&jar, &headers, &state.db).await.ok();

    Ok(Json(list_collections(viewer.as_ref(), &state.db).await?))
}