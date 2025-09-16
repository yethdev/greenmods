//! The add admin route.

use axum::{
    extract::{Path, State},
    http::HeaderMap,
};
use axum_extra::extract::CookieJar;
use modhost_auth::get_user_from_req;
use modhost_core::{AppError, Result};
use modhost_db::get_user;
use modhost_server_core::state::AppState;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, IntoActiveModel};

/// Add Admin
///
/// Promote a user to admin.
#[utoipa::path(
    put,
    path = "/add/{id}",
    tag = "Admin",
    params(
        ("id" = i32, description = "The user ID."),
    ),
    responses(
        (status = 200, description = "Added!"),
        (status = INTERNAL_SERVER_ERROR, description = "An internal error occured!"),
    ),
    security(
        ("api_auth_token" = []),
    ),
)]
#[debug_handler]
pub async fn add_handler(
    jar: CookieJar,
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(user): Path<String>,
) -> Result<()> {
    let to_add = get_user(user, &state.db).await?;
    let user = get_user_from_req(&jar, &headers, &state.db).await?;

    if !user.admin {
        return Err(AppError::NoAccess);
    }

    let mut user = to_add.into_active_model();

    user.admin = Set(true);
    user.update(&state.db).await?;

    Ok(())
}
