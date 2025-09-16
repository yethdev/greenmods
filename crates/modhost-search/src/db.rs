use crate::MeiliProject;
use itertools::Itertools;
use modhost_core::Result;
use modhost_db::{
    DbConn,
    prelude::{ProjectVersions, Projects, Users},
};
use sea_orm::EntityTrait;

/// Get all projects in the database as [`MeiliProject`]s.
pub async fn get_all_projects(conn: &DbConn) -> Result<Vec<MeiliProject>> {
    Ok(Projects::find()
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
        .collect_vec())
}
