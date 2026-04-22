use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.alter_table(
            Table::alter()
                .table(Questions::Table)
                .add_column(ColumnDef::new(Questions::FollowupEnabled).boolean().null())
                .to_owned(),
        )
        .await?;
        m.alter_table(
            Table::alter()
                .table(Questions::Table)
                .add_column(ColumnDef::new(Questions::FollowupCorrectQuestion).string().null())
                .to_owned(),
        )
        .await?;
        m.alter_table(
            Table::alter()
                .table(Questions::Table)
                .add_column(ColumnDef::new(Questions::FollowupCorrectAnswer).string().null())
                .to_owned(),
        )
        .await?;
        m.alter_table(
            Table::alter()
                .table(Questions::Table)
                .add_column(ColumnDef::new(Questions::FollowupWrongQuestion).string().null())
                .to_owned(),
        )
        .await?;
        m.alter_table(
            Table::alter()
                .table(Questions::Table)
                .add_column(ColumnDef::new(Questions::FollowupWrongAnswer).string().null())
                .to_owned(),
        )
        .await?;
        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.alter_table(
            Table::alter()
                .table(Questions::Table)
                .drop_column(Questions::FollowupEnabled)
                .drop_column(Questions::FollowupCorrectQuestion)
                .drop_column(Questions::FollowupCorrectAnswer)
                .drop_column(Questions::FollowupWrongQuestion)
                .drop_column(Questions::FollowupWrongAnswer)
                .to_owned(),
        )
        .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum Questions {
    Table,
    FollowupEnabled,
    FollowupCorrectQuestion,
    FollowupCorrectAnswer,
    FollowupWrongQuestion,
    FollowupWrongAnswer,
}