use sea_orm_migration::{
    DbErr, SchemaManager,
    async_trait::async_trait,
    prelude::{
        ForeignKey, ForeignKeyAction, ForeignKeyCreateStatement, ForeignKeyDropStatement, IdenList,
        IntoTableRef, Table,
    },
    sea_orm::DbBackend,
};

pub struct ForeignKeyInfo<A: IntoTableRef + IdenList + Copy, B: IntoTableRef + IdenList + Copy> {
    pub from: (A, A),
    pub to: (B, B),
}

pub const fn fkey<F: IntoTableRef + IdenList + Copy, T: IntoTableRef + IdenList + Copy>(
    from: (F, F),
    to: (T, T),
) -> ForeignKeyInfo<F, T> {
    ForeignKeyInfo::<F, T> { from, to }
}

impl<A: IntoTableRef + IdenList + Copy, B: IntoTableRef + IdenList + Copy> ForeignKeyInfo<A, B> {
    fn name(&self) -> String {
        let from_table = self.from.0.into_table_ref().sea_orm_table().inner();

        let from_cols = self
            .from
            .1
            .into_iter()
            .map(|it| it.inner().to_string())
            .collect::<Vec<_>>()
            .join("_");

        let to_table = self.to.0.into_table_ref().sea_orm_table().inner();

        let to_cols = self
            .to
            .1
            .into_iter()
            .map(|it| it.inner().to_string())
            .collect::<Vec<_>>()
            .join("_");

        let name = format!("fk-{from_table}-{from_cols}-{to_table}-{to_cols}");

        name
    }

    pub fn create(&self) -> ForeignKeyCreateStatement {
        ForeignKey::create()
            .from(self.from.0, self.from.1)
            .to(self.to.0, self.to.1)
            .on_delete(ForeignKeyAction::Cascade)
            .on_update(ForeignKeyAction::Cascade)
            .name(self.name())
            .to_owned()
    }

    pub fn drop(&self) -> ForeignKeyDropStatement {
        ForeignKey::drop()
            .table(self.from.0)
            .name(self.name())
            .to_owned()
    }

    pub async fn auto_drop(&self, mgr: &SchemaManager<'_>) -> Result<(), DbErr> {
        if mgr.get_database_backend() != DbBackend::Sqlite {
            mgr.drop_foreign_key(self.drop()).await?;
        }

        Ok(())
    }
}

#[async_trait]
pub trait SchemaManagerExt {
    async fn drop(&self, table: impl IntoTableRef + Send) -> Result<(), DbErr>;
}

#[async_trait]
impl SchemaManagerExt for SchemaManager<'_> {
    async fn drop(&self, table: impl IntoTableRef + Send) -> Result<(), DbErr> {
        self.drop_table(Table::drop().if_exists().table(table).to_owned())
            .await
    }
}

#[macro_export]
macro_rules! fk {
    ($name: ident: $e1: ident::$f1: ident => $e2: ident::$f2: ident) => {
        pub const $name: $crate::util::ForeignKeyInfo<$e1, $e2> =
            $crate::util::fkey(($e1::Table, $e1::$f1), ($e2::Table, $e2::$f2));
    };
}
