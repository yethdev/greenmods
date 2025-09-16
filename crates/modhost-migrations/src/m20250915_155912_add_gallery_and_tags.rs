use sea_orm_migration::{prelude::*, schema::*};

use crate::{m20250915_155846_initial_setup::Packages, util::SchemaManagerExt};

#[derive(DeriveIden, Clone, Copy)]
pub enum GalleryImages {
    Table,
    Id,
    Name,
    Description,
    Ordering,
    S3Id,
    Package,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden, Clone, Copy)]
pub enum PackagesAdded {
    Tags,
}

mod fk {
    use super::*;
    use crate::{fk, m20250915_155846_initial_setup::Packages};

    fk!(GALLERY_IMGS_PACKAGE: GalleryImages::Package => Packages::Id);
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GalleryImages::Table)
                    .if_not_exists()
                    .col(pk_auto(GalleryImages::Id))
                    .col(text(GalleryImages::Name).not_null())
                    .col(text_null(GalleryImages::Description))
                    .col(integer(GalleryImages::Ordering).not_null().default(-1))
                    .col(text(GalleryImages::S3Id).not_null())
                    .col(integer(GalleryImages::Package).not_null())
                    .col(
                        timestamp(GalleryImages::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp(GalleryImages::UpdatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(&mut fk::GALLERY_IMGS_PACKAGE.create())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Packages::Table)
                    .add_column(
                        array(PackagesAdded::Tags, ColumnType::Text)
                            .not_null()
                            .default("{}"),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        fk::GALLERY_IMGS_PACKAGE.auto_drop(&manager).await?;

        manager.drop(GalleryImages::Table).await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Packages::Table)
                    .drop_column(PackagesAdded::Tags)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
