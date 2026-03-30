use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(m, "frames",
            &[
            
            ("id", ColType::PkAuto),
            
            ("video_id", ColType::String),
            ("frame_number", ColType::Integer),
            ("timestamp_seconds", ColType::Integer),
            ("timestamp_formatted", ColType::String),
            ("filename", ColType::String),
            ("file_path", ColType::String),
            ("subtitle_text", ColType::StringNull),
            ("is_keyframe", ColType::Boolean),
            ],
            &[
            ]
        ).await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "frames").await
    }
}
