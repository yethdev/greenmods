//! The download gallery image route.

use axum::{
    extract::{Path, State},
    http::header::CONTENT_TYPE,
    response::Response,
};
use modhost_core::Result;
use modhost_db::get_gallery_image;
use modhost_db_util::gallery::get_image;
use modhost_server_core::state::AppState;

/// Get Gallery Image Data
///
/// Get a gallery image file data from S3.
/// A URL to this endpoint should be returned by any other gallery endpoints.
#[utoipa::path(
    get,
    path = "/{image}/download",
    tag = "Gallery",
    responses(
        (status = 200, description = "The gallery image.", body = Vec<u8>),
        (status = INTERNAL_SERVER_ERROR, description = "Error: image might not exist, or another error occured!"),
    ),
    params(
        ("image" = String, Path, description = "The gallery image's ID."),
    ),
)]
#[debug_handler]
pub async fn download_handler(
    Path((_project, id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Response> {
    let img = get_gallery_image(id, &state.db).await?;
    let bytes = get_image(img.s3_id, &state.buckets.gallery).await?;
    let ty = imghdr::from_bytes(&bytes);

    let mime = match ty {
        Some(it) => format!("image/{}", it.ext()),
        None => "application/octet-stream".into(),
    };

    Ok(Response::builder()
        .header(CONTENT_TYPE, mime)
        .body(bytes.into())?)
}
