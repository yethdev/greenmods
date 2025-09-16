use sea_orm_migration::{prelude::*, schema::*};

use crate::m20250915_155846_initial_setup::UserTokens;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden, Clone, Copy)]
pub enum UserTokensAdded {
    Expires,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(UserTokens::Table)
                    .add_column(
                        timestamp(UserTokensAdded::Expires)
                            .not_null()
                            .default(Expr::current_timestamp()),
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
                    .table(UserTokens::Table)
                    .drop_column(UserTokensAdded::Expires)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
