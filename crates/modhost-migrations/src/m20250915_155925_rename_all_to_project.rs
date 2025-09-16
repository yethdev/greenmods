use sea_orm_migration::prelude::*;

use crate::{
    m20250915_155846_initial_setup::{
        PackageAuthors, PackageRelations, PackageVersionRefs, PackageVersions, Packages,
    },
    m20250915_155912_add_gallery_and_tags::GalleryImages,
};

#[derive(DeriveIden, Clone, Copy)]
pub enum Renamed {
    Projects,
    ProjectVersions,
    ProjectVersionRefs,
    ProjectRelations,
    ProjectAuthors,
    Project,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .rename_table(
                Table::rename()
                    .table(Packages::Table, Renamed::Projects)
                    .to_owned(),
            )
            .await?;

        manager
            .rename_table(
                Table::rename()
                    .table(PackageVersions::Table, Renamed::ProjectVersions)
                    .to_owned(),
            )
            .await?;

        manager
            .rename_table(
                Table::rename()
                    .table(PackageVersionRefs::Table, Renamed::ProjectVersionRefs)
                    .to_owned(),
            )
            .await?;

        manager
            .rename_table(
                Table::rename()
                    .table(PackageRelations::Table, Renamed::ProjectRelations)
                    .to_owned(),
            )
            .await?;

        manager
            .rename_table(
                Table::rename()
                    .table(PackageAuthors::Table, Renamed::ProjectAuthors)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Renamed::ProjectVersions)
                    .rename_column(PackageVersions::Package, Renamed::Project)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Renamed::ProjectAuthors)
                    .rename_column(PackageAuthors::Package, Renamed::Project)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Renamed::ProjectRelations)
                    .rename_column(PackageRelations::Package, Renamed::Project)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(GalleryImages::Table)
                    .rename_column(GalleryImages::Package, Renamed::Project)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .rename_table(
                Table::rename()
                    .table(Renamed::Projects, Packages::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .rename_table(
                Table::rename()
                    .table(Renamed::ProjectVersions, PackageVersions::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .rename_table(
                Table::rename()
                    .table(Renamed::ProjectVersionRefs, PackageVersionRefs::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .rename_table(
                Table::rename()
                    .table(Renamed::ProjectRelations, PackageRelations::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .rename_table(
                Table::rename()
                    .table(Renamed::ProjectAuthors, PackageAuthors::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(PackageVersions::Table)
                    .rename_column(Renamed::Project, PackageVersions::Package)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(PackageAuthors::Table)
                    .rename_column(Renamed::Project, PackageAuthors::Package)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(PackageRelations::Table)
                    .rename_column(Renamed::Project, PackageRelations::Package)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(GalleryImages::Table)
                    .rename_column(Renamed::Project, GalleryImages::Package)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
