use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1️⃣ Create Customers Table
        manager
            .create_table(
                Table::create()
                    .table(Customers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Customers::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Customers::Name).string().not_null())
                    .col(ColumnDef::new(Customers::Email).string().unique_key())
                    .col(ColumnDef::new(Customers::Password).string().not_null())
                    .col(
                        ColumnDef::new(Customers::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // 2️⃣ Create Meters Table
        manager
            .create_table(
                Table::create()
                    .table(Meters::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Meters::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Meters::CustomerId).string().not_null())
                    .col(ColumnDef::new(Meters::Amount).decimal_len(10, 2).default(0))
                    .col(
                        ColumnDef::new(Meters::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Meters::Table, Meters::CustomerId)
                            .to(Customers::Table, Customers::Id),
                    )
                    .to_owned(),
            )
            .await?;
        

        // 3️⃣ Create Invoices Table
        manager
            .create_table(
                Table::create()
                    .table(Invoices::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Invoices::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Invoices::CustomerId).string().not_null())
                    .col(ColumnDef::new(Invoices::Amount).decimal_len(10, 2).not_null())
                    .col(
                        ColumnDef::new(Invoices::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Invoices::Table, Invoices::CustomerId)
                            .to(Customers::Table, Customers::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // 4️⃣ Create Payments Table
        manager
            .create_table(
                Table::create()
                    .table(Payments::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Payments::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Payments::InvoiceId).string().not_null())
                    .col(ColumnDef::new(Payments::CustomerId).string().not_null())
                    .col(ColumnDef::new(Payments::Amount).decimal_len(10, 2).not_null())
                    .col(ColumnDef::new(Payments::BalAmount).decimal_len(10, 2).default(0))
                    .col(
                        ColumnDef::new(Payments::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Payments::Table, Payments::InvoiceId)
                            .to(Invoices::Table, Invoices::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Payments::Table, Payments::CustomerId)
                            .to(Customers::Table, Customers::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // 5️⃣ Create Readings Table
        manager
            .create_table(
                Table::create()
                    .table(Readings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Readings::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Readings::MeterId).string().not_null())
                    .col(ColumnDef::new(Readings::Units).integer().not_null())
                    .col(ColumnDef::new(Readings::Timestamp).timestamp().not_null())
                    .col(ColumnDef::new(Readings::Period).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Readings::Table, Readings::MeterId)
                            .to(Meters::Table, Meters::Id),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop in reverse order of creation to satisfy FK constraints
        manager.drop_table(Table::drop().table(Readings::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Payments::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Invoices::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Meters::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Customers::Table).to_owned()).await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Customers {
    Table,
    Id,
    Name,
    Email,
    Password,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Meters {
    Table,
    Id,
    CustomerId,
    Amount,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Invoices {
    Table,
    Id,
    CustomerId,
    Amount,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Payments {
    Table,
    Id,
    InvoiceId,
    CustomerId,
    Amount,
    BalAmount,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Readings {
    Table,
    Id,
    MeterId,
    Units,
    Timestamp,
    Period,
}
