//! Moderation data utilities.

use modhost_core::{AppError, Result};
use modhost_db::{
    DbConn, ModerationComment, ModerationQueueItem, ModerationQueueStatus, Project, User,
    moderation_comment, moderation_queue,
    prelude::{ModerationComment as ModerationComments, ModerationQueue},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait,
    QueryFilter,
};

/// Get the entire queue of moderation items with a specific status.
pub async fn get_queue_by_status(
    status: ModerationQueueStatus,
    conn: &DbConn,
) -> Result<Vec<ModerationQueueItem>> {
    Ok(ModerationQueue::find()
        .filter(moderation_queue::Column::Status.eq(status))
        .all(conn)
        .await?)
}

/// Get the entire moderation queue.
pub async fn get_moderation_queue(conn: &DbConn) -> Result<Vec<ModerationQueueItem>> {
    Ok(ModerationQueue::find().all(conn).await?)
}

/// Get the moderation queue item for a project.
pub async fn get_moderation_queue_item(
    project: &Project,
    conn: &DbConn,
) -> Result<ModerationQueueItem> {
    project
        .find_related(ModerationQueue)
        .one(conn)
        .await?
        .ok_or(AppError::NotFound)
}

/// Get or create the moderation queue item for a project.
pub async fn get_or_create_moderation_queue_item(
    project: &Project,
    conn: &DbConn,
) -> Result<ModerationQueueItem> {
    let existing = get_moderation_queue_item(project, conn).await;

    match existing.ok() {
        Some(it) => Ok(it),
        None => Ok(ModerationQueue::insert(moderation_queue::ActiveModel {
            project_id: Set(project.id),
            ..Default::default()
        })
        .exec_with_returning(conn)
        .await?),
    }
}

/// Set the moderation status for a project.
pub async fn set_moderation_status(
    project: &Project,
    status: ModerationQueueStatus,
    conn: &DbConn,
) -> Result<ModerationQueueItem> {
    let item = get_moderation_queue_item(project, conn).await?;
    let mut item = item.into_active_model();

    item.status = Set(status);

    Ok(item.update(conn).await?)
}

/// Set the assigned moderator for a project.
pub async fn set_assigned_moderator(
    project: &Project,
    assigned: i32,
    conn: &DbConn,
) -> Result<ModerationQueueItem> {
    let item = get_moderation_queue_item(project, conn).await?;
    let mut item = item.into_active_model();

    item.assigned_id = Set(assigned);

    Ok(item.update(conn).await?)
}

/// Get the moderation comments for a project.
pub async fn get_moderation_comments(
    project: &Project,
    conn: &DbConn,
) -> Result<Vec<ModerationComment>> {
    Ok(project.find_related(ModerationComments).all(conn).await?)
}

/// Create a new moderation comment on a project.
pub async fn create_moderation_comment(
    project: &Project,
    user: &User,
    comment: String,
    conn: &DbConn,
) -> Result<ModerationComment> {
    Ok(ModerationComments::insert(moderation_comment::ActiveModel {
        user_id: Set(user.id),
        project_id: Set(project.id),
        is_moderator: Set(user.moderator || user.admin),
        is_system: Set(user.id == -1),
        comment: Set(comment),
        ..Default::default()
    })
    .exec_with_returning(conn)
    .await?)
}
