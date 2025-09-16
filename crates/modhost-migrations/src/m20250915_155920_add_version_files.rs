use sea_orm_migration::{prelude::*, schema::*};

use crate::{m20250915_155846_initial_setup::PackageVersions, util::SchemaManagerExt};

#[derive(DeriveIden, Clone, Copy)]
pub enum VersionFiles {
    Table,
    Id,
    FileName,
    Sha1,
    S3Id,
    Size,
    VersionId,
    UploadedAt,
}

mod fk {
    use super::*;
    use crate::{fk, m20250915_155846_initial_setup::PackageVersions};

    fk!(VERSION_FILES_VERSION_ID: VersionFiles::VersionId => PackageVersions::Id);
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(PackageVersions::Table)
                    .drop_column(PackageVersions::FileId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(VersionFiles::Table)
                    .if_not_exists()
                    .col(pk_auto(VersionFiles::Id))
                    .col(text(VersionFiles::FileName).not_null())
                    .col(text(VersionFiles::Sha1).not_null())
                    .col(text(VersionFiles::S3Id).not_null())
                    .col(big_integer(VersionFiles::Size).not_null())
                    .col(integer(VersionFiles::VersionId).not_null())
                    .col(
                        timestamp(VersionFiles::UploadedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(&mut fk::VERSION_FILES_VERSION_ID.create())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        fk::VERSION_FILES_VERSION_ID.auto_drop(manager).await?;

        manager.drop(VersionFiles::Table).await?;

        manager
            .alter_table(
                Table::alter()
                    .table(PackageVersions::Table)
                    .add_column(text(PackageVersions::FileId).not_null().default("")) // <-- FIXME: this is invalid data
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
