pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20251021_043820_create_tables;
mod m20251023_160743_make_meter_customerId_nullable;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20251021_043820_create_tables::Migration),
            Box::new(m20251023_160743_make_meter_customerId_nullable::Migration),
        ]
    }
}
