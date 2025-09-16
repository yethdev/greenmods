pub use sea_orm_migration::prelude::*;

pub mod util;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250915_155846_initial_setup::Migration),
            Box::new(m20250915_155850_add_token_invalidation::Migration),
            Box::new(m20250915_155857_add_downloads_to_packages::Migration),
            Box::new(m20250915_155903_rename_minecraft::Migration),
            Box::new(m20250915_155908_add_license_and_visibility::Migration),
            Box::new(m20250915_155912_add_gallery_and_tags::Migration),
            Box::new(m20250915_155914_remove_views::Migration),
            Box::new(m20250915_155917_add_admin::Migration),
            Box::new(m20250915_155920_add_version_files::Migration),
            Box::new(m20250915_155922_add_moderation::Migration),
            Box::new(m20250915_155925_rename_all_to_project::Migration),
            Box::new(m20250915_155928_add_moderators::Migration),
        ]
    }
}

mod m20250915_155846_initial_setup;
mod m20250915_155850_add_token_invalidation;
mod m20250915_155857_add_downloads_to_packages;
mod m20250915_155903_rename_minecraft;
mod m20250915_155908_add_license_and_visibility;
mod m20250915_155912_add_gallery_and_tags;
mod m20250915_155914_remove_views;
mod m20250915_155917_add_admin;
mod m20250915_155920_add_version_files;
mod m20250915_155922_add_moderation;
mod m20250915_155925_rename_all_to_project;
mod m20250915_155928_add_moderators;
