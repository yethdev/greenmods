//! Utilities for working with users.

use itertools::Itertools;
use modhost_core::Result;
use modhost_db::{
    AsProjectData, DbConn, Project, ProjectAuthor, ProjectData, ProjectVisibility, User,
    prelude::{ProjectAuthors, Projects, Users},
    projects, users,
};
use sea_orm::{ColumnTrait, EntityTrait, ExprTrait, QueryFilter};

/// Get a list of projects for a user.
pub async fn get_user_projects(
    authed_user: Option<User>,
    user: i32,
    conn: &DbConn,
) -> Result<Vec<ProjectData>> {
    let mut query = Projects::find()
        .find_also_related(ProjectAuthors)
        .and_also_related(Users)
        .filter(users::Column::Id.eq(user));

    if let Some(authed_user) = authed_user {
        if !authed_user.admin && !authed_user.moderator {
            query = query.filter(
                projects::Column::Visibility
                    .eq(ProjectVisibility::Public)
                    .or(users::Column::Id.eq(authed_user.id)),
            );
        }
    } else {
        query = query.filter(projects::Column::Visibility.eq(ProjectVisibility::Public));
    }

    Ok(
        (query.all(conn).await? as Vec<(Project, Option<ProjectAuthor>, Option<User>)>)
            .into_iter()
            .filter_map(|(a, _, c)| if let Some(c) = c { Some((a, c)) } else { None })
            .into_group_map()
            .into_iter()
            .map(|v| v.0.with_authors(v.1))
            .collect_vec(),
    )
}
