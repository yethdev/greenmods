#![warn(missing_docs)]
//! ModHost's database module, containing models and utilities.

mod models;
mod util;

pub use models::*;
pub use util::*;

use migration::{
    Migrator, MigratorTrait,
    sea_orm::{Database, DatabaseConnection},
};
use modhost_core::{Result, utoipa_types};
use std::env;

/// The async database pool type.
pub type DbPool = DatabaseConnection;

/// The async database connection type.
pub type DbConn = DatabaseConnection;

/// Create an async connection to a database.
pub async fn create_connection(db_url: Option<String>) -> Result<DbPool> {
    let embedded_db_url = option_env!("DATABASE_URL").map(|v| v.to_string());

    let db_url = db_url.map(Ok).unwrap_or_else(|| {
        embedded_db_url
            .map(Ok)
            .unwrap_or_else(|| env::var("DATABASE_URL"))
    })?;

    Ok(Database::connect(db_url).await?)
}

/// Run the migrations on an async database connection via its pool.
pub async fn run_migrations(pool: &DbPool) -> Result<()> {
    Migrator::up(pool, None).await?;

    Ok(())
}

/// Reset the database, then run the migrations on an async database connection via its pool.
pub async fn fresh_migrations(pool: &DbPool) -> Result<()> {
    Migrator::reset(pool).await?;
    Migrator::up(pool, None).await?;

    Ok(())
}

utoipa_types![
    User,
    UserToken,
    ProjectManifest,
    Project,
    ProjectAuthor,
    ProjectRelation,
    ProjectFile,
    ProjectVersionRef,
    // RelationKind,
    ProjectData,
    ProjectVisibility,
    GalleryImage,
    PublicGalleryImage,
    ProjectVersionData,
    ModerationComment,
    ModerationQueueItem,
    ModerationQueueStatus,
];
