use sea_orm_migration::{
    prelude::{extension::postgres::Type, *},
    schema::*,
    sea_orm::{EnumIter, Iterable},
};

use crate::util::SchemaManagerExt;

#[derive(DeriveIden, Clone, Copy)]
pub struct ModerationStatusEnum;

#[derive(DeriveIden, Clone, Copy, EnumIter)]
pub enum ModerationStatus {
    Pending,
    Denied,
    Approved,
    UnderReview,
}

#[derive(DeriveIden, Clone, Copy)]
pub enum ModerationQueue {
    Table,
    Id,
    ProjectId,
    AssignedId,
    Status,
}

#[derive(DeriveIden, Clone, Copy)]
pub enum ModerationComment {
    Table,
    Id,
    ProjectId,
    UserId,
    IsSystem,
    IsModerator,
    Comment,
}

mod fk {
    use super::*;
    use crate::{
        fk,
        m20250915_155846_initial_setup::{Packages, Users},
    };

    fk!(MODERATION_QUEUE_PROJECT_ID: ModerationQueue::ProjectId => Packages::Id);
    fk!(MODERATION_QUEUE_ASSIGNED_ID: ModerationQueue::AssignedId => Users::Id);

    fk!(MODERATION_COMMENT_PROJECT_ID: ModerationComment::ProjectId => Packages::Id);
    fk!(MODERATION_COMMENT_USER_ID: ModerationComment::UserId => Users::Id);
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(ModerationStatusEnum)
                    .values(ModerationStatus::iter())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ModerationQueue::Table)
                    .if_not_exists()
                    .col(pk_auto(ModerationQueue::Id))
                    .col(integer(ModerationQueue::ProjectId).not_null())
                    .col(integer(ModerationQueue::AssignedId).not_null())
                    .col(
                        custom(ModerationQueue::Status, ModerationStatusEnum)
                            .not_null()
                            .default(ModerationStatus::Pending.to_string()),
                    )
                    .foreign_key(&mut fk::MODERATION_QUEUE_PROJECT_ID.create())
                    .foreign_key(&mut fk::MODERATION_QUEUE_ASSIGNED_ID.create())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ModerationComment::Table)
                    .if_not_exists()
                    .col(pk_auto(ModerationComment::Id))
                    .col(integer(ModerationComment::ProjectId).not_null())
                    .col(integer(ModerationComment::UserId).not_null())
                    .col(
                        boolean(ModerationComment::IsSystem)
                            .not_null()
                            .default(false),
                    )
                    .col(
                        boolean(ModerationComment::IsModerator)
                            .not_null()
                            .default(false),
                    )
                    .col(text(ModerationComment::Comment).not_null())
                    .foreign_key(&mut fk::MODERATION_COMMENT_PROJECT_ID.create())
                    .foreign_key(&mut fk::MODERATION_COMMENT_USER_ID.create())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        fk::MODERATION_QUEUE_PROJECT_ID.auto_drop(&manager).await?;
        fk::MODERATION_QUEUE_ASSIGNED_ID.auto_drop(&manager).await?;

        fk::MODERATION_COMMENT_PROJECT_ID
            .auto_drop(&manager)
            .await?;

        fk::MODERATION_COMMENT_USER_ID.auto_drop(&manager).await?;

        manager.drop(ModerationQueue::Table).await?;
        manager.drop(ModerationComment::Table).await?;

        manager
            .drop_type(
                Type::drop()
                    .if_exists()
                    .name(ModerationStatusEnum)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
