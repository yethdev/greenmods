//! Moderation queue routes.

use axum::{Json, extract::State, http::HeaderMap};
use axum_extra::extract::CookieJar;
use modhost_auth::get_user_from_req;
use modhost_core::{AppError, Result};
use modhost_db::{ModerationQueueItem, sea_orm_active_enums::ModerationStatusEnum};
use modhost_db_util::moderation::{get_moderation_queue, get_queue_by_status};
use modhost_server_core::state::AppState;

/// Full Queue
///
/// Get the entire moderation queue.
#[utoipa::path(
    get,
    path = "/queue",
    tag = "Moderation",
    responses(
        (status = 200, description = "Fetched the queue!", body = Vec<ModerationQueueItem>),
        (status = INTERNAL_SERVER_ERROR, description = "An internal error occured!"),
    ),
    security(
        ("api_auth_token" = []),
    ),
)]
#[debug_handler]
pub async fn list_queue(
    jar: CookieJar,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<Vec<ModerationQueueItem>>> {
    let user = get_user_from_req(&jar, &headers, &state.db).await?;

    if !user.admin || !user.moderator {
        return Err(AppError::NoAccess);
    }

    Ok(Json(get_moderation_queue(&state.db).await?))
}

/// Pending Queue
///
/// Get the pending moderation queue.
#[utoipa::path(
    get,
    path = "/queue/pending",
    tag = "Moderation",
    responses(
        (status = 200, description = "Fetched the queue!", body = Vec<ModerationQueueItem>),
        (status = INTERNAL_SERVER_ERROR, description = "An internal error occured!"),
    ),
    security(
        ("api_auth_token" = []),
    ),
)]
#[debug_handler]
pub async fn list_queue_pending(
    jar: CookieJar,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<Vec<ModerationQueueItem>>> {
    let user = get_user_from_req(&jar, &headers, &state.db).await?;

    if !user.admin || !user.moderator {
        return Err(AppError::NoAccess);
    }

    Ok(Json(
        get_queue_by_status(ModerationStatusEnum::Pending, &state.db).await?,
    ))
}

/// Approved Queue
///
/// Get the approved moderation queue.
#[utoipa::path(
    get,
    path = "/queue/approved",
    tag = "Moderation",
    responses(
        (status = 200, description = "Fetched the queue!", body = Vec<ModerationQueueItem>),
        (status = INTERNAL_SERVER_ERROR, description = "An internal error occured!"),
    ),
    security(
        ("api_auth_token" = []),
    ),
)]
#[debug_handler]
pub async fn list_queue_approved(
    jar: CookieJar,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<Vec<ModerationQueueItem>>> {
    let user = get_user_from_req(&jar, &headers, &state.db).await?;

    if !user.admin || !user.moderator {
        return Err(AppError::NoAccess);
    }

    Ok(Json(
        get_queue_by_status(ModerationStatusEnum::Approved, &state.db).await?,
    ))
}

/// Under Review Queue
///
/// Get the under review moderation queue.
#[utoipa::path(
    get,
    path = "/queue/under_review",
    tag = "Moderation",
    responses(
        (status = 200, description = "Fetched the queue!", body = Vec<ModerationQueueItem>),
        (status = INTERNAL_SERVER_ERROR, description = "An internal error occured!"),
    ),
    security(
        ("api_auth_token" = []),
    ),
)]
#[debug_handler]
pub async fn list_queue_under_review(
    jar: CookieJar,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<Vec<ModerationQueueItem>>> {
    let user = get_user_from_req(&jar, &headers, &state.db).await?;

    if !user.admin || !user.moderator {
        return Err(AppError::NoAccess);
    }

    Ok(Json(
        get_queue_by_status(ModerationStatusEnum::UnderReview, &state.db).await?,
    ))
}

/// Denied Queue
///
/// Get the denied moderation queue.
#[utoipa::path(
    get,
    path = "/queue/denied",
    tag = "Moderation",
    responses(
        (status = 200, description = "Fetched the queue!", body = Vec<ModerationQueueItem>),
        (status = INTERNAL_SERVER_ERROR, description = "An internal error occured!"),
    ),
    security(
        ("api_auth_token" = []),
    ),
)]
#[debug_handler]
pub async fn list_queue_denied(
    jar: CookieJar,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<Vec<ModerationQueueItem>>> {
    let user = get_user_from_req(&jar, &headers, &state.db).await?;

    if !user.admin || !user.moderator {
        return Err(AppError::NoAccess);
    }

    Ok(Json(
        get_queue_by_status(ModerationStatusEnum::Denied, &state.db).await?,
    ))
}
