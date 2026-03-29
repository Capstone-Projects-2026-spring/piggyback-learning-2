#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;

mod m20260329_190420_parents;
mod m20260329_201606_kids;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260329_190420_parents::Migration),
            Box::new(m20260329_201606_kids::Migration),
            // inject-above (do not remove this comment)
        ]
    }
}