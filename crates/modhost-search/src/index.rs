//! Utilities for indexing projects.

use crate::{MeiliProject, MeilisearchService, get_all_projects};
use meilisearch_sdk::documents::DocumentDeletionQuery;
use modhost_core::Result;
use modhost_db::{
    DbConn,
    prelude::{ProjectVersions, Projects, Users},
    projects,
};
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter};

impl MeilisearchService {
    /// Index all project present in the database.
    /// THIS IS A DESTRUCTIVE ACTION! IT WILL DELETE ALL EXISTING DATA
    /// IN THE MEILISEARCH INDEX!
    pub async fn index_projects(&self, conn: &DbConn) -> Result<()> {
        // This is my baby abomination and I am so proud of it.
        let projects = get_all_projects(conn).await?;
        let index = self.projects();

        index.delete_all_documents().await?;
        index.add_documents(projects.as_slice(), Some("id")).await?;

        Ok(())
    }

    /// Update a project in the Meilisearch index.
    pub async fn update_project(&self, project: i32, conn: &DbConn) -> Result<()> {
        let Some(project) = Projects::find()
            .filter(projects::Column::Id.eq(project))
            .one(conn)
            .await?
        else {
            return Ok(());
        };

        let authors = project.find_related(Users).all(conn).await?;

        if authors.is_empty() {
            return Ok(());
        }

        let versions = project.find_related(ProjectVersions).all(conn).await?;
        let data = MeiliProject::from_data(project, authors, versions);

        self.projects()
            .add_or_replace(&[data], Some("id"))
            .await?
            .wait_for_completion(&self.client, None, None)
            .await?;

        Ok(())
    }

    /// Delete a project from the Meilisearch index.
    pub async fn delete_project(&self, project: i32) -> Result<()> {
        let index = self.projects();
        let mut query = DocumentDeletionQuery::new(&index);
        let filter = format!("id = {}", project);

        query.with_filter(&filter);
        index.delete_documents_with(&query).await?;

        Ok(())
    }
}
