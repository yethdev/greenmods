//! The create gallery image route.

use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use modhost_auth::get_user_from_req;
use modhost_core::{AppError, Result};
use modhost_db::{PublicGalleryImage, gallery_images, prelude::ProjectAuthors};
use modhost_db_util::{gallery::transform_gallery_image, projects::get_project};
use modhost_server_core::state::AppState;
use object_store::{ObjectStore, PutPayload};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, IntoActiveModel, ModelTrait};
use sha1::{Digest, Sha1};

/// The data for uploading a gallery image.
/// This should be formatted as "multipart/form-data".
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema, ToResponse,
)]
pub struct GalleryImageUpload {
    /// The project ID.
    pub project: i32,

    /// The display name of the image.
    pub name: String,

    /// An optional markdown-formatted description.
    pub description: Option<String>,

    /// The order of this image.
    pub ordering: i32,

    /// The image file data itself.
    #[schema(content_media_type = "application/octet-stream")]
    pub file: Vec<u8>,
}

/// Upload Gallery Image
///
/// Upload a gallery image
#[utoipa::path(
    put,
    path = "/",
    tag = "Gallery",
    responses(
        (status = 200, description = "Created gallery image!", body = PublicGalleryImage),
        (status = INTERNAL_SERVER_ERROR, description = "An internal error occured!"),
    ),
    request_body(content = inline(GalleryImageUpload), description = "The gallery image metadata", content_type = "multipart/form-data"),
    security(
        ("api_auth_token" = []),
    ),
)]
#[debug_handler]
pub async fn create_handler(
    jar: CookieJar,
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<AppState>,
    mut data: Multipart,
) -> Result<Response> {
    let user = get_user_from_req(&jar, &headers, &state.db).await?;
    let pkg = get_project(id, &state.db).await?;
    let authors = pkg.find_related(ProjectAuthors).all(&state.db).await?;

    if !authors.iter().any(|v| v.user_id == user.id) && !user.admin {
        return Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::empty())?);
    }

    let mut name = None;
    let mut description = None;
    let mut ordering = None;
    let mut file = None;

    while let Ok(Some(field)) = data.next_field().await {
        match field.name().ok_or(AppError::MissingFieldName)? {
            "name" => name = Some(field.text().await?),
            "description" => description = Some(field.text().await?),
            "ordering" => ordering = Some(field.text().await?),
            "file" => file = Some(field.bytes().await?),
            _ => {}
        }
    }

    if name.is_none() {
        Err(AppError::MissingField("name".into()))?;
    }

    if file.is_none() {
        Err(AppError::MissingField("file".into()))?;
    }

    let name = name.unwrap();
    let ordering = ordering.unwrap_or("-1".into()).parse()?;
    let file = file.unwrap();
    let file_format = imghdr::from_bytes(&file).ok_or(AppError::InvalidImageFile)?;
    let mut hasher = Sha1::new();

    hasher.update(&file);

    let file_id = format!("{:x}", hasher.finalize());
    let file_name = format!("{}.{}", file_id, file_format);

    state
        .buckets
        .gallery
        .put(
            &format!("/{}", file_name).into(),
            PutPayload::from_bytes(file),
        )
        .await?;

    let data = gallery_images::ActiveModel {
        project: Set(pkg.id),
        name: Set(name),
        description: Set(description),
        ordering: Set(ordering),
        s3_id: Set(file_name),
        ..Default::default()
    };

    let mut pkg = pkg.into_active_model();

    pkg.updated_at = Set(Utc::now().naive_utc());
    pkg.update(&state.db).await?;

    let image = data.insert(&state.db).await?;

    Ok(Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::new(serde_json::to_string(&transform_gallery_image(
            image,
        ))?))?)
}
