use sea_orm_migration::{prelude::*, schema::*};

use crate::m20250915_155846_initial_setup::Packages;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Packages::Table)
                    .drop_column(Packages::Views)
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
                    .add_column(integer(Packages::Views).not_null().default(0))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
