use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(
            m,
            "questions",
            &[
                ("id", ColType::PkAuto),
                ("qtype", ColType::String),
                ("question", ColType::String),
                ("answer", ColType::String),
                ("rank", ColType::IntegerNull),
            ],
            &[("segment", "")],
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "questions").await
    }
}
