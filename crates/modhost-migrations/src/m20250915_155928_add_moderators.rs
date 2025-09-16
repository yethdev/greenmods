use crate::m20250915_155846_initial_setup::Users;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveIden, Clone, Copy)]
pub enum UsersAdded {
    Moderator,
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
                    .add_column(boolean(UsersAdded::Moderator).not_null().default(false))
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
                    .drop_column(UsersAdded::Moderator)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
