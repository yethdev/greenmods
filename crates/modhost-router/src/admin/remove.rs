//! The remove admin route.

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

/// Remove Admin
///
/// Demote a user from admin to a normal user.
/// If a user was not already an admin, this function will still succeed.
#[utoipa::path(
    delete,
    path = "/remove/{id}",
    tag = "Admin",
    params(
        ("id" = i32, description = "The user ID."),
    ),
    responses(
        (status = 200, description = "Removed!"),
        (status = INTERNAL_SERVER_ERROR, description = "An internal error occured!"),
    ),
    security(
        ("api_auth_token" = []),
    ),
)]
#[debug_handler]
pub async fn remove_handler(
    jar: CookieJar,
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(user): Path<String>,
) -> Result<()> {
    let to_remove = get_user(user, &state.db).await?;
    let user = get_user_from_req(&jar, &headers, &state.db).await?;

    if !user.admin {
        return Err(AppError::NoAccess);
    }

    let mut to_remove = to_remove.into_active_model();

    to_remove.admin = Set(false);
    to_remove.update(&state.db).await?;

    Ok(())
}
