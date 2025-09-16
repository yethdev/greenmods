//! Utilities for tokens.

use crate::DbConn;
use chrono::{DateTime, Utc};
use migration::sea_orm::{ActiveValue::Set, ColumnTrait, EntityTrait, ModelTrait, QueryFilter};
use modhost_core::Result;
use modhost_entities::{
    prelude::{UserTokens, Users},
    user_tokens, users,
};
use random_string::{charsets::ALPHANUMERIC, generate};

/// The time until a token expires in milliseconds.
/// Calculation: 1 * SECS_PER_WEEK * MILLIS_PER_SEC
#[allow(clippy::identity_op)]
pub const TOKEN_EXPIRE_TIME: i64 = 1 * 604800 * 1000;

/// The length of a token string.
pub const TOKEN_LENGTH: usize = 64;

/// Generate a token to insert into the database.
pub fn generate_token(user_id: i32) -> user_tokens::ActiveModel {
    user_tokens::ActiveModel {
        user_id: Set(user_id),
        value: Set(generate(TOKEN_LENGTH, ALPHANUMERIC)),
        expires: Set(DateTime::from_timestamp_millis(
            Utc::now().timestamp_millis() + TOKEN_EXPIRE_TIME,
        )
        .unwrap()
        .naive_utc()),
        ..Default::default()
    }
}

/// Create a token and insert it into the database.
pub async fn create_token(user_id: i32, pool: &DbConn) -> Result<user_tokens::Model> {
    Ok(UserTokens::insert(generate_token(user_id))
        .exec_with_returning(pool)
        .await?)
}

/// Get the user the provided token belongs to.
pub async fn get_user_for_token(
    token: impl AsRef<str>,
    conn: &DbConn,
) -> Result<Option<users::Model>> {
    let token = UserTokens::find()
        .filter(user_tokens::Column::Value.eq(token.as_ref().to_string()))
        .one(conn)
        .await?;

    if let Some(token) = token {
        Ok(token.find_related(Users).one(conn).await?)
    } else {
        Ok(None)
    }
}
