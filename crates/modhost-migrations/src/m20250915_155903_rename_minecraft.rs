use crate::m20250915_155846_initial_setup::PackageVersions;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveIden, Clone, Copy)]
pub enum PackageVersRenamed {
    GameVersions,
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
                    .rename_column(PackageVersions::Minecraft, PackageVersRenamed::GameVersions)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(PackageVersions::Table)
                    .drop_column(PackageVersions::Kubejs)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(PackageVersions::Table)
                    .rename_column(PackageVersRenamed::GameVersions, PackageVersions::Minecraft)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(PackageVersions::Table)
                    .add_column(
                        array(PackageVersions::Kubejs, ColumnType::Text)
                            .not_null()
                            .default("{}"),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
