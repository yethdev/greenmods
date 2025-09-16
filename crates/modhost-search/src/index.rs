//! Utilities for indexing projects.

use crate::{MeiliProject, MeilisearchService, get_all_projects};
use itertools::Itertools;
use meilisearch_sdk::documents::DocumentDeletionQuery;
use modhost_core::Result;
use modhost_db::{
    DbConn,
    prelude::{ProjectVersions, Projects, Users},
    projects,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

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
        // Abomination #2! It's so beautiful! I make Rust programmers worldwide upset!
        let data = Projects::find()
            .filter(projects::Column::Id.eq(project))
            .find_also_related(Users)
            .find_also_related(ProjectVersions)
            .all(conn)
            .await?
            .into_iter()
            .filter_map(|(a, b, c)| {
                if let Some(b) = b {
                    Some((a, b, c))
                } else {
                    None
                }
            })
            .filter_map(|(a, b, c)| {
                if let Some(c) = c {
                    Some((a, b, c))
                } else {
                    None
                }
            })
            .into_group_map_by(|v| v.0.clone())
            .into_iter()
            .map(|v| (v.0, v.1.into_iter().map(|v| (v.1, v.2)).unzip()))
            .map(|v| MeiliProject::from_data(v.0, v.1.0, v.1.1))
            .next();

        if let Some(data) = data {
            self.projects()
                .add_or_replace(&[data], Some("id"))
                .await?
                .wait_for_completion(&self.client, None, None)
                .await?;
        }

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
