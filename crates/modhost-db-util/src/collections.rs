//! Utilities for working with project collections.

use crate::projects::{ProjectUtils, get_full_project};
use modhost_core::{AppError, Result};
use modhost_db::{
    DbConn, ProjectCollection, ProjectCollectionData, ProjectVisibility, User,
    prelude::{ProjectCollections, Users},
    project_collections,
};
use sea_orm::{ColumnTrait, Condition, EntityTrait, ExprTrait, ModelTrait, QueryFilter, QueryOrder};

/// Get a collection by its ID or slug.
pub async fn get_collection(id: impl AsRef<str>, conn: &DbConn) -> Result<ProjectCollection> {
    let id = id.as_ref();

    let mut query = ProjectCollections::find();

    if let Ok(num) = id.parse::<i32>() {
        query = query.filter(
            project_collections::Column::Id
                .eq(num)
                .or(project_collections::Column::Slug.eq(id)),
        );
    } else {
        query = query.filter(project_collections::Column::Slug.eq(id));
    }

    query.one(conn).await?.ok_or(AppError::NotFound)
}

/// Get a fully hydrated collection.
pub async fn get_full_collection(
    id: impl AsRef<str>,
    viewer: Option<&User>,
    conn: &DbConn,
) -> Result<ProjectCollectionData> {
    let collection = get_collection(id, conn).await?;
    let owner = collection
        .find_related(Users)
        .one(conn)
        .await?
        .ok_or(AppError::UnknownUser)?;
    let can_view_hidden = viewer
        .map(|user| user.admin || user.id == owner.id)
        .unwrap_or(false);
    let mut projects = Vec::new();
    let mut project_ids = Vec::new();

    for project_id in &collection.project_ids {
        let Ok(project) = get_full_project(project_id.to_string(), conn).await else {
            continue;
        };

        let is_visible = if can_view_hidden {
            true
        } else if let Some(viewer) = viewer {
            project.is_visible_to(viewer, conn).await?
        } else {
            project.is_visible(conn).await?
        };

        if is_visible {
            project_ids.push(*project_id);
            projects.push(project);
        }
    }

    Ok(ProjectCollectionData {
        id: collection.id,
        owner,
        slug: collection.slug,
        name: collection.name,
        description: collection.description,
        readme: collection.readme,
        project_ids,
        projects,
        visibility: collection.visibility,
        created_at: collection.created_at,
        updated_at: collection.updated_at,
    })
}

/// List all collections visible to a viewer.
pub async fn list_collections(
    viewer: Option<&User>,
    conn: &DbConn,
) -> Result<Vec<ProjectCollectionData>> {
    let mut query = ProjectCollections::find().order_by_desc(project_collections::Column::UpdatedAt);

    if let Some(viewer) = viewer {
        if !viewer.admin {
            query = query.filter(
                Condition::any()
                    .add(project_collections::Column::Visibility.eq(ProjectVisibility::Public))
                    .add(project_collections::Column::OwnerId.eq(viewer.id)),
            );
        }
    } else {
        query = query.filter(project_collections::Column::Visibility.eq(ProjectVisibility::Public));
    }

    let collections = query.all(conn).await?;
    let mut hydrated = Vec::with_capacity(collections.len());

    for collection in collections {
        hydrated.push(get_full_collection(collection.id.to_string(), viewer, conn).await?);
    }

    Ok(hydrated)
}