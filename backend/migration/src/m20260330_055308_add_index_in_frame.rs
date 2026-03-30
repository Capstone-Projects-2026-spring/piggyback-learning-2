use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.create_index(
            Index::create()
                .if_not_exists()
                .name("idx-timestamp")
                .table(Frames::Table)
                .col(Frames::TimestampSeconds)
                .unique()
                .to_owned(),
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.drop_index(
            Index::drop()
                .name("idx-timestamp")
                .table(Frames::Table)
                .to_owned(),
        )
        .await
    }
}

#[derive(Iden)]
enum Frames {
    Table,
    _Id,
    TimestampSeconds,
}
