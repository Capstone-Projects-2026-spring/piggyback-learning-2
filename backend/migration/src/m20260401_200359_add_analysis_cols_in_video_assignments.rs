use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.alter_table(
            Table::alter()
                .table(VideoAssignments::Table)
                .add_column(ColumnDef::new(VideoAssignments::Answers).json().null())
                .to_owned(),
        )
        .await?;

        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.alter_table(
            Table::alter()
                .table(VideoAssignments::Table)
                .drop_column(VideoAssignments::Answers)
                .to_owned(),
        )
        .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum VideoAssignments {
    Table,
    Answers,
}
