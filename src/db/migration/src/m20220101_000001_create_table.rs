use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MovieServings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(MovieServings::Path)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(MovieServings::LastTimestamp).big_integer().not_null())
                    .col(ColumnDef::new(MovieServings::LastFilePosition).big_integer().not_null())
                    .col(ColumnDef::new(MovieServings::FileLength).big_integer().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MovieServings::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum MovieServings {
    Table,
    Path,
    LastTimestamp,
    LastFilePosition,
    FileLength,
}

