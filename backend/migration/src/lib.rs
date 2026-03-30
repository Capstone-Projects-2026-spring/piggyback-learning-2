#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;

mod m20260329_190420_parents;
mod m20260329_201606_kids;
mod m20260329_211415_tags;
mod m20260329_212214_kid_tags;
mod m20260329_220403_videos;
mod m20260330_034450_video_tags;
mod m20260330_053736_frames;
mod m20260330_055308_add_index_in_frame;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260329_190420_parents::Migration),
            Box::new(m20260329_201606_kids::Migration),
            Box::new(m20260329_211415_tags::Migration),
            Box::new(m20260329_212214_kid_tags::Migration),
            Box::new(m20260329_220403_videos::Migration),
            Box::new(m20260330_034450_video_tags::Migration),
            Box::new(m20260330_053736_frames::Migration),
            Box::new(m20260330_055308_add_index_in_frame::Migration),
            // inject-above (do not remove this comment)
        ]
    }
}