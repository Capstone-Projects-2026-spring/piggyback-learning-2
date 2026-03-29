#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;

mod m20260329_190420_parents;
mod m20260329_201606_kids;
mod m20260329_211415_tags;
mod m20260329_212214_kid_tags;
mod m20260329_220403_videos;
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
            // inject-above (do not remove this comment)
        ]
    }
}