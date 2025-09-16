use crate::m20250915_155846_initial_setup::Packages;
use sea_orm_migration::{
    prelude::{extension::postgres::Type, *},
    schema::*,
    sea_orm::{EnumIter, Iterable},
};

#[derive(DeriveIden, Clone, Copy)]
pub struct VisibilityEnum;

#[derive(DeriveIden, Clone, Copy, EnumIter)]
pub enum Visibility {
    Public,
    Private,
    Unlisted,
}

#[derive(DeriveIden, Clone, Copy)]
pub enum PackagesAdded {
    License,
    Visibility,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(VisibilityEnum)
                    .values(Visibility::iter())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Packages::Table)
                    .add_column(text_null(PackagesAdded::License))
                    .add_column(
                        custom(PackagesAdded::Visibility, VisibilityEnum)
                            .not_null()
                            .default(Visibility::Public.to_string()),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Packages::Table)
                    .drop_column(PackagesAdded::License)
                    .drop_column(PackagesAdded::Visibility)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_type(Type::drop().if_exists().name(VisibilityEnum).to_owned())
            .await?;

        Ok(())
    }
}
