use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(
            m,
            "segments",
            &[
                ("id", ColType::PkAuto),
                ("start_seconds", ColType::Integer),
                ("end_seconds", ColType::Integer),
                ("best_question", ColType::StringNull),
            ],
            &[("video", "")],
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "segments").await
    }
}
