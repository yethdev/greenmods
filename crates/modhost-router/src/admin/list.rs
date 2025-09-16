//! The admin list route.

use axum::{Json, extract::State, http::HeaderMap};
use axum_extra::extract::CookieJar;
use modhost_auth::get_user_from_req;
use modhost_core::{AppError, Result};
use modhost_db::{User, prelude::Users, users};
use modhost_server_core::state::AppState;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

/// List Admins
///
/// Get a list of admins.
#[utoipa::path(
    get,
    path = "/list",
    tag = "Admin",
    responses(
        (status = 200, description = "Got admins!", body = Vec<User>),
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
) -> Result<Json<Vec<User>>> {
    let user = get_user_from_req(&jar, &headers, &state.db).await?;

    if !user.admin {
        return Err(AppError::NoAccess);
    }

    Ok(Json(
        Users::find()
            .filter(users::Column::Admin.eq(true))
            .all(&state.db)
            .await?,
    ))
}
