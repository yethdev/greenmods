//! The delete gallery image route.

use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};
use axum_extra::extract::CookieJar;
use modhost_auth::get_user_from_req;
use modhost_core::Result;
use modhost_db::{
    gallery_images, get_gallery_image,
    prelude::{GalleryImages, ProjectAuthors},
};
use modhost_db_util::projects::get_project;
use modhost_server_core::state::AppState;
use object_store::ObjectStore;
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter};

/// Delete Gallery Image
///
/// Delete a gallery image
#[utoipa::path(
    delete,
    path = "/{image}",
    tag = "Gallery",
    responses(
        (status = 200, description = "Deleted gallery image!", body = String),
        (status = INTERNAL_SERVER_ERROR, description = "An internal error occured!"),
    ),
    params(
        ("image" = String, Path, description = "The gallery image ID number."),
    ),
    security(
        ("api_auth_token" = []),
    ),
)]
#[debug_handler]
pub async fn delete_handler(
    jar: CookieJar,
    headers: HeaderMap,
    Path((project, image)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Response> {
    let user = get_user_from_req(&jar, &headers, &state.db).await?;
    let pkg = get_project(project, &state.db).await?;
    let img = get_gallery_image(image, &state.db).await?;
    let authors = pkg.find_related(ProjectAuthors).all(&state.db).await?;

    if !authors.iter().any(|v| v.user_id == user.id) && !user.admin {
        return Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::empty())?);
    }

    let all_referencing = GalleryImages::find()
        .filter(gallery_images::Column::S3Id.eq(img.s3_id.clone()))
        .all(&state.db)
        .await?;

    if all_referencing.len() <= 1 {
        state
            .buckets
            .gallery
            .delete(&format!("/{}", img.s3_id).into())
            .await?;
    }

    img.delete(&state.db).await?;

    Ok(Response::builder().body(Body::new("Deleted gallery image successfully!".to_string()))?)
}
