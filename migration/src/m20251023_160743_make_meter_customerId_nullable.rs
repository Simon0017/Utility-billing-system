use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                .table(Meters::Table)
                .modify_column(ColumnDef::new(Meters::CustomerId).string().null())
                .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // rollback
        manager
            .alter_table(
                Table::alter()
                .table(Meters::Table)
                .modify_column(ColumnDef::new(Meters::CustomerId).string().not_null())
                .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Meters {
    Table,
    Id,
    CustomerId,
    Amount,
    CreatedAt,
}
