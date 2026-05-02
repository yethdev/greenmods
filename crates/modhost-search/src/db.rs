use crate::MeiliProject;
use modhost_core::Result;
use modhost_db::{
    DbConn,
    prelude::{ProjectVersions, Projects, Users},
};
use sea_orm::{EntityTrait, ModelTrait};

/// Get all projects in the database as [`MeiliProject`]s.
pub async fn get_all_projects(conn: &DbConn) -> Result<Vec<MeiliProject>> {
    let mut indexed = Vec::new();

    for project in Projects::find().all(conn).await? {
        let authors = project.find_related(Users).all(conn).await?;

        if authors.is_empty() {
            continue;
        }

        let versions = project.find_related(ProjectVersions).all(conn).await?;

        indexed.push(MeiliProject::from_data(project, authors, versions));
    }

    Ok(indexed)
}
