use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(
            m,
            "videos",
            &[
                ("id", ColType::StringUniq),
                ("title", ColType::StringNull),
                ("thumbnail_url", ColType::StringNull),
                ("duration_seconds", ColType::IntegerNull),
                ("local_video_path", ColType::StringNull),
            ],
            &[],
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "videos").await
    }
}
