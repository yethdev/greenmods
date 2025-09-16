use sea_orm_migration::{prelude::*, schema::*};

use crate::m20250915_155846_initial_setup::Users;

#[derive(DeriveIden, Clone, Copy)]
pub enum UsersAdded {
    Admin,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(boolean(UsersAdded::Admin).not_null().default(false))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(UsersAdded::Admin)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
