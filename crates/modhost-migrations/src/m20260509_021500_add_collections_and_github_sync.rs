use sea_orm_migration::{prelude::*, schema::*};

use crate::m20250915_155908_add_license_and_visibility::VisibilityEnum;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden, Clone, Copy)]
enum Projects {
    Table,
    Id,
    Faq,
    RepoLinks,
    InstallJson,
}

#[derive(DeriveIden, Clone, Copy)]
enum Users {
    Table,
    Id,
}

#[derive(DeriveIden, Clone, Copy)]
enum ProjectCollections {
    Table,
    Id,
    OwnerId,
    Slug,
    Name,
    Description,
    Readme,
    ProjectIds,
    Visibility,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden, Clone, Copy)]
enum ProjectRepoSyncs {
    Table,
    ProjectId,
    RepoOwner,
    RepoName,
    DefaultBranch,
    WebhookSecret,
    SyncReadme,
    SyncReleases,
    SyncFaq,
    SyncLinks,
    LastPushSyncAt,
    LastReleaseSyncAt,
    LastError,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Projects::Table)
                    .add_column(text_null(Projects::Faq))
                    .add_column(text_null(Projects::RepoLinks))
                    .add_column(text_null(Projects::InstallJson))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ProjectCollections::Table)
                    .if_not_exists()
                    .col(pk_auto(ProjectCollections::Id))
                    .col(integer(ProjectCollections::OwnerId).not_null())
                    .col(text(ProjectCollections::Slug).not_null().unique_key())
                    .col(text(ProjectCollections::Name).not_null())
                    .col(text(ProjectCollections::Description).not_null())
                    .col(text(ProjectCollections::Readme).not_null())
                    .col(
                        array(ProjectCollections::ProjectIds, ColumnType::Integer)
                            .not_null()
                            .default("{}"),
                    )
                    .col(
                        custom(ProjectCollections::Visibility, VisibilityEnum)
                            .not_null()
                            .default("public"),
                    )
                    .col(
                        timestamp(ProjectCollections::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp(ProjectCollections::UpdatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-project-collections-owner")
                            .from(ProjectCollections::Table, ProjectCollections::OwnerId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ProjectRepoSyncs::Table)
                    .if_not_exists()
                    .col(integer(ProjectRepoSyncs::ProjectId).not_null().primary_key())
                    .col(text(ProjectRepoSyncs::RepoOwner).not_null())
                    .col(text(ProjectRepoSyncs::RepoName).not_null())
                    .col(text_null(ProjectRepoSyncs::DefaultBranch))
                    .col(text(ProjectRepoSyncs::WebhookSecret).not_null())
                    .col(boolean(ProjectRepoSyncs::SyncReadme).not_null().default(true))
                    .col(boolean(ProjectRepoSyncs::SyncReleases).not_null().default(true))
                    .col(boolean(ProjectRepoSyncs::SyncFaq).not_null().default(true))
                    .col(boolean(ProjectRepoSyncs::SyncLinks).not_null().default(true))
                    .col(timestamp_null(ProjectRepoSyncs::LastPushSyncAt))
                    .col(timestamp_null(ProjectRepoSyncs::LastReleaseSyncAt))
                    .col(text_null(ProjectRepoSyncs::LastError))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-project-repo-syncs-project")
                            .from(ProjectRepoSyncs::Table, ProjectRepoSyncs::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ProjectRepoSyncs::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ProjectCollections::Table).to_owned())
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Projects::Table)
                    .drop_column(Projects::Faq)
                    .drop_column(Projects::RepoLinks)
                    .drop_column(Projects::InstallJson)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}