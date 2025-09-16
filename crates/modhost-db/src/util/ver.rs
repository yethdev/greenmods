//! Utilities for project versions.

use migration::sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use modhost_core::{AppError, Result};
use modhost_entities::{prelude::ProjectVersions, project_versions};

use crate::DbConn;

/// Get a version by its ID, name, or version number.
pub async fn get_version(
    project: i32,
    id: impl AsRef<str>,
    conn: &DbConn,
) -> Result<project_versions::Model> {
    let id = id.as_ref();

    if let Ok(id) = id.parse::<i32>() {
        let ver = ProjectVersions::find_by_id(id).one(conn).await?;

        if let Some(ver) = ver {
            return Ok(ver);
        }
    }

    if let Some(ver) = ProjectVersions::find()
        .filter(project_versions::Column::VersionNumber.eq(id))
        .filter(project_versions::Column::Project.eq(project))
        .one(conn)
        .await?
    {
        return Ok(ver);
    }

    ProjectVersions::find()
        .filter(project_versions::Column::Name.eq(id))
        .filter(project_versions::Column::Project.eq(project))
        .one(conn)
        .await?
        .ok_or(AppError::NotFound)
}
