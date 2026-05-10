//! The collection delete route.

use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};
use axum_extra::extract::CookieJar;
use modhost_auth::get_user_from_req;
use modhost_core::Result;
use modhost_db_util::collections::get_collection;
use modhost_server_core::state::AppState;
use sea_orm::ModelTrait;

/// Delete Collection
///
/// Delete a project collection.
#[utoipa::path(
    delete,
    path = "/{id}",
    tag = "Collections",
    responses(
        (status = 200, description = "Collection deleted successfully!"),
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
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Response> {
    let user = get_user_from_req(&jar, &headers, &state.db).await?;
    let collection = get_collection(id, &state.db).await?;

    if !user.admin && collection.owner_id != user.id {
        return Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::empty())?);
    }

    collection.delete(&state.db).await?;

    Ok(Response::builder().status(StatusCode::OK).body(Body::empty())?)
}