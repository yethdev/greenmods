use sea_orm_migration::{prelude::*, schema::*};

use crate::util::SchemaManagerExt;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden, Clone, Copy)]
pub enum Users {
    Table,
    Id,
    Username,
    GithubId,
}

#[derive(DeriveIden, Clone, Copy)]
pub enum UserTokens {
    Table,
    Id,
    UserId,
    Value,
}

#[derive(DeriveIden, Clone, Copy)]
pub enum Packages {
    Table,
    Id,
    Name,
    Slug,
    Readme,
    Description,
    Views,
    Source,
    Issues,
    Wiki,
    CreatedAt,

    #[sea_orm(iden = "updated_at")]
    UpdatedAt,
}

#[derive(DeriveIden, Clone, Copy)]
pub enum PackageVersions {
    Table,
    Id,
    Package,
    Name,
    VersionNumber,
    FileId,
    Changelog,
    Kubejs,
    Loaders,
    Minecraft,
    Downloads,
    CreatedAt,

    #[sea_orm(iden = "updated_at")]
    UpdatedAt,
}

#[derive(DeriveIden, Clone, Copy)]
pub enum PackageVersionRefs {
    Table,
    Value,
}

#[derive(DeriveIden, Clone, Copy)]
pub enum PackageRelations {
    Table,
    Package,
    Dependency,
    Kind,
}

#[derive(DeriveIden, Clone, Copy)]
pub enum PackageAuthors {
    Table,
    Package,
    UserId,
}

mod fk {
    use super::*;
    use crate::fk;

    fk!(USER_TOKENS_USER_ID: UserTokens::UserId => Users::Id);
    fk!(PKG_VERS_PACKAGE: PackageVersions::Package => Packages::Id);
    fk!(PKG_VER_REFS_VALUE: PackageVersionRefs::Value => PackageVersions::Id);
    fk!(PKG_RELS_PACKAGE: PackageRelations::Package => PackageVersions::Id);
    fk!(PKG_RELS_DEPENDENCY: PackageRelations::Dependency => PackageVersionRefs::Value);
    fk!(PKG_AUTHORS_PACKAGE: PackageAuthors::Package => Packages::Id);
    fk!(PKG_AUTHORS_USER_ID: PackageAuthors::UserId => Users::Id);
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(pk_auto(Users::Id))
                    .col(string(Users::Username).not_null())
                    .col(integer(Users::GithubId).not_null().unique_key())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(UserTokens::Table)
                    .if_not_exists()
                    .col(pk_auto(UserTokens::Id))
                    .col(integer(UserTokens::UserId).not_null())
                    .col(text(UserTokens::Value).not_null())
                    .foreign_key(&mut fk::USER_TOKENS_USER_ID.create())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Packages::Table)
                    .if_not_exists()
                    .col(pk_auto(Packages::Id))
                    .col(text(Packages::Name).not_null())
                    .col(text(Packages::Slug).not_null().unique_key())
                    .col(text(Packages::Readme).not_null())
                    .col(text(Packages::Description).not_null())
                    .col(integer(Packages::Views).not_null().default(0))
                    .col(text_null(Packages::Source))
                    .col(text_null(Packages::Issues))
                    .col(text_null(Packages::Wiki))
                    .col(
                        timestamp(Packages::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp(Packages::UpdatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PackageVersions::Table)
                    .if_not_exists()
                    .col(pk_auto(PackageVersions::Id))
                    .col(integer(PackageVersions::Package).not_null())
                    .col(text(PackageVersions::Name).not_null())
                    .col(text(PackageVersions::VersionNumber).not_null())
                    .col(text(PackageVersions::FileId).not_null())
                    .col(text_null(PackageVersions::Changelog))
                    .col(
                        array(PackageVersions::Kubejs, ColumnType::Text)
                            .not_null()
                            .default("{}"),
                    )
                    .col(
                        array(PackageVersions::Loaders, ColumnType::Text)
                            .not_null()
                            .default("{}"),
                    )
                    .col(
                        array(PackageVersions::Minecraft, ColumnType::Text)
                            .not_null()
                            .default("{}"),
                    )
                    .col(integer(PackageVersions::Downloads).not_null().default(0))
                    .col(
                        timestamp(PackageVersions::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp(PackageVersions::UpdatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(&mut fk::PKG_VERS_PACKAGE.create())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PackageVersionRefs::Table)
                    .if_not_exists()
                    .col(integer(PackageVersionRefs::Value).not_null())
                    .foreign_key(&mut fk::PKG_VER_REFS_VALUE.create())
                    .primary_key(Index::create().col(PackageVersionRefs::Value))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PackageRelations::Table)
                    .if_not_exists()
                    .col(integer(PackageRelations::Package).not_null())
                    .col(integer(PackageRelations::Dependency).not_null())
                    // The relation kind. 0 = dependency, 1 = incompatibility
                    .col(integer(PackageRelations::Kind).not_null())
                    .foreign_key(&mut fk::PKG_RELS_PACKAGE.create())
                    .foreign_key(&mut fk::PKG_RELS_DEPENDENCY.create())
                    .primary_key(
                        Index::create()
                            .col(PackageRelations::Package)
                            .col(PackageRelations::Dependency)
                            .col(PackageRelations::Kind),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PackageAuthors::Table)
                    .if_not_exists()
                    .col(integer(PackageAuthors::Package).not_null())
                    .col(integer(PackageAuthors::UserId).not_null())
                    .foreign_key(&mut fk::PKG_AUTHORS_PACKAGE.create())
                    .foreign_key(&mut fk::PKG_AUTHORS_USER_ID.create())
                    .primary_key(
                        Index::create()
                            .col(PackageAuthors::Package)
                            .col(PackageAuthors::UserId),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        fk::USER_TOKENS_USER_ID.auto_drop(&manager).await?;
        fk::PKG_VERS_PACKAGE.auto_drop(&manager).await?;
        fk::PKG_VER_REFS_VALUE.auto_drop(&manager).await?;
        fk::PKG_RELS_PACKAGE.auto_drop(&manager).await?;
        fk::PKG_RELS_DEPENDENCY.auto_drop(&manager).await?;
        fk::PKG_AUTHORS_PACKAGE.auto_drop(&manager).await?;
        fk::PKG_AUTHORS_USER_ID.auto_drop(&manager).await?;

        manager.drop(UserTokens::Table).await?;
        manager.drop(PackageAuthors::Table).await?;
        manager.drop(Users::Table).await?;
        manager.drop(PackageRelations::Table).await?;
        manager.drop(PackageVersionRefs::Table).await?;
        manager.drop(PackageVersions::Table).await?;
        manager.drop(Packages::Table).await?;

        Ok(())
    }
}
