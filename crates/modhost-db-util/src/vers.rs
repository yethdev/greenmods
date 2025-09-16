//! Utilities for working with project versions.

use modhost_core::{AppError, Result};
use modhost_db::{
    AsVersionData, DbConn, Project, ProjectFile, ProjectVersion, ProjectVersionData,
    prelude::{ProjectVersions, VersionFiles},
    project_versions, version_files,
};
use sea_orm::{ColumnTrait, EntityTrait, ExprTrait, ModelTrait, QueryFilter};
use semver::Version;

/// Get a list of versions for a project.
pub async fn get_versions(project: &Project, conn: &DbConn) -> Result<Vec<ProjectVersionData>> {
    Ok(project
        .find_related(ProjectVersions)
        .find_with_related(VersionFiles)
        .all(conn)
        .await?
        .into_iter()
        .map(|v| v.0.with_files(v.1))
        .collect())
}

/// Get the full version data for a project.
pub async fn get_full_version(
    project: &Project,
    ver: impl AsRef<str>,
    conn: &DbConn,
) -> Result<ProjectVersionData> {
    let ver = ver.as_ref();

    let mut query = project
        .find_related(ProjectVersions)
        .find_with_related(VersionFiles);

    if let Ok(ver_num) = ver.parse::<i32>() {
        query = query.filter(
            project_versions::Column::Id
                .eq(ver_num)
                .or(project_versions::Column::Name
                    .eq(ver)
                    .or(project_versions::Column::VersionNumber.eq(ver))),
        );
    } else {
        query = query.filter(
            project_versions::Column::Name
                .eq(ver)
                .or(project_versions::Column::VersionNumber.eq(ver)),
        );
    }

    query
        .all(conn)
        .await?
        .into_iter()
        .map(|v| v.0.with_files(v.1))
        .next()
        .ok_or(AppError::NotFound)
}

/// Get a version file.
pub async fn get_version_file(
    ver: i32,
    file: impl AsRef<str>,
    conn: &DbConn,
) -> Result<ProjectFile> {
    let file = file.as_ref();

    let mut query = VersionFiles::find().filter(version_files::Column::VersionId.eq(ver));

    if let Ok(file_id) = file.parse::<i32>() {
        query = query.filter(
            version_files::Column::Id
                .eq(file_id)
                .or(version_files::Column::FileName.eq(file)),
        );
    } else {
        query = query.filter(version_files::Column::FileName.eq(file));
    }

    query.one(conn).await?.ok_or(AppError::NotFound)
}

/// Get a project's latest version.
pub async fn get_latest_version(project: &Project, conn: &DbConn) -> Result<ProjectVersion> {
    let mut versions = project.find_related(ProjectVersions).all(conn).await?;

    versions.sort_by(|a, b| {
        Version::parse(&a.version_number)
            .unwrap()
            .cmp(&Version::parse(&b.version_number).unwrap())
    });

    versions.last().cloned().ok_or(AppError::NoVersions)
}

/// Get a project's latest version.
pub async fn get_latest_full_version(
    project: &Project,
    conn: &DbConn,
) -> Result<ProjectVersionData> {
    let mut versions = get_versions(project, conn).await?;

    versions.sort_by(|a, b| {
        Version::parse(&a.version_number)
            .unwrap()
            .cmp(&Version::parse(&b.version_number).unwrap())
    });

    versions.last().cloned().ok_or(AppError::NoVersions)
}
