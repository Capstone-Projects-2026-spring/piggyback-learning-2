use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.create_index(
            Index::create()
                .name("idx-unique-segment")
                .table(Segments::Table)
                .col(Segments::VideoId)
                .col(Segments::StartSeconds)
                .col(Segments::EndSeconds)
                .unique()
                .to_owned(),
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.drop_index(
            Index::drop()
                .name("idx-unique-segment")
                .table(Segments::Table)
                .to_owned(),
        )
        .await
    }
}

#[derive(Iden)]
enum Segments {
    Table,
    VideoId,
    StartSeconds,
    EndSeconds,
}
