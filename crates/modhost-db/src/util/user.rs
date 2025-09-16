//! Utilities for users.

use migration::sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use modhost_core::{AppError, Result};
use modhost_entities::{prelude::Users, users};

use crate::DbConn;

/// Get a user by their ID, GitHub ID, or their username.
pub async fn get_user(id: impl AsRef<str>, conn: &DbConn) -> Result<users::Model> {
    let id = id.as_ref();

    if let Ok(id) = id.parse::<i32>() {
        if let Some(user) = Users::find_by_id(id).one(conn).await? {
            return Ok(user);
        }

        if let Some(user) = Users::find()
            .filter(users::Column::GithubId.eq(id))
            .one(conn)
            .await?
        {
            return Ok(user);
        }
    }

    Users::find()
        .filter(users::Column::Username.eq(id))
        .one(conn)
        .await?
        .ok_or(AppError::NotFound)
}

/// Search for users with the specified string in their username.
pub async fn search_users(name: impl AsRef<str>, conn: &mut DbConn) -> Result<Vec<users::Model>> {
    let name = name.as_ref();

    Ok(Users::find()
        .filter(users::Column::Username.like(format!("%{}%", name)))
        .all(conn)
        .await?)
}
