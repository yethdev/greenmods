//! Utilities for gallery images.

use migration::sea_orm::EntityTrait;
use modhost_core::{AppError, Result};
use modhost_entities::{gallery_images, prelude::GalleryImages};

use crate::DbConn;

/// Get a gallery image from the databasse by its ID.
pub async fn get_gallery_image(
    id: impl AsRef<str>,
    conn: &DbConn,
) -> Result<gallery_images::Model> {
    let id = id.as_ref().parse::<i32>()?;

    GalleryImages::find_by_id(id)
        .one(conn)
        .await?
        .ok_or(AppError::NotFound)
}
