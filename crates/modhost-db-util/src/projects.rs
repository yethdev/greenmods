//! Utilities for working with projects.

use crate::moderation::get_moderation_queue_item;
use itertools::Itertools;
use modhost_core::{AppError, Result};
use modhost_db::{
    AsProjectData, DbConn, GalleryImage, ModerationQueueStatus, Project, ProjectData,
    ProjectVisibility, User,
    prelude::{GalleryImages, ProjectAuthors, Projects, Users},
    projects,
};
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter};

/// Utilities for working with a project.
pub trait ProjectUtils {
    /// Check if this project is publicly visible in search.
    async fn is_visible_in_search(&self, conn: &DbConn) -> Result<bool>;

    /// Check if this project is visible (public or unlisted).
    async fn is_visible(&self, conn: &DbConn) -> Result<bool>;

    /// Check if this project is visible to a user.
    async fn is_visible_to(&self, user: &User, conn: &DbConn) -> Result<bool>;
}

impl ProjectUtils for Project {
    async fn is_visible_in_search(&self, conn: &DbConn) -> Result<bool> {
        Ok(self.visibility == ProjectVisibility::Public
            && get_moderation_queue_item(self, conn).await?.status
                == ModerationQueueStatus::Approved)
    }

    async fn is_visible(&self, conn: &DbConn) -> Result<bool> {
        Ok((self.visibility == ProjectVisibility::Public
            || self.visibility == ProjectVisibility::Unlisted)
            && get_moderation_queue_item(self, conn).await?.status
                == ModerationQueueStatus::Approved)
    }

    async fn is_visible_to(&self, user: &User, conn: &DbConn) -> Result<bool> {
        Ok((self.visibility == ProjectVisibility::Public
            && get_moderation_queue_item(self, conn).await?.status
                == ModerationQueueStatus::Approved)
            || get_full_project(self.id.to_string(), conn)
                .await?
                .authors
                .contains(user))
    }
}

impl ProjectUtils for ProjectData {
    async fn is_visible_in_search(&self, conn: &DbConn) -> Result<bool> {
        self.clone().into_project().is_visible_in_search(conn).await
    }

    async fn is_visible(&self, conn: &DbConn) -> Result<bool> {
        self.clone().into_project().is_visible(conn).await
    }

    async fn is_visible_to(&self, user: &User, conn: &DbConn) -> Result<bool> {
        Ok((self.visibility == ProjectVisibility::Public
            && get_moderation_queue_item(&self.clone().into_project(), conn)
                .await?
                .status
                == ModerationQueueStatus::Approved)
            || self.authors.contains(user))
    }
}

/// Get a project by its ID or slug.
pub async fn get_project(id: impl AsRef<str>, conn: &DbConn) -> Result<Project> {
    let id = id.as_ref();

    if let Ok(id) = id.parse::<i32>() {
        let proj = Projects::find_by_id(id).one(conn).await?;

        if let Some(proj) = proj {
            return Ok(proj);
        }
    }

    Projects::find()
        .filter(projects::Column::Slug.like(id))
        .one(conn)
        .await?
        .ok_or(AppError::NotFound)
}

/// Get the full data for a project by its ID or slug.
pub async fn get_full_project(id: impl AsRef<str>, conn: &DbConn) -> Result<ProjectData> {
    let proj = get_project(id, conn).await?;

    let authors = proj
        .find_related(ProjectAuthors)
        .find_also_related(Users)
        .all(conn)
        .await?
        .into_iter()
        .filter_map(|(_, it)| it)
        .collect_vec();

    Ok(proj.with_authors(authors))
}

/// Get the gallery images for a project.
pub async fn get_gallery(pkg_id: impl AsRef<str>, conn: &DbConn) -> Result<Vec<GalleryImage>> {
    Ok(get_project(pkg_id, conn)
        .await?
        .find_related(GalleryImages)
        .all(conn)
        .await?)
}
