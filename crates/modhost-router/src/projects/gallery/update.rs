//! The gallery image update route.

use axum::{
    Json,
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use modhost_auth::get_user_from_req;
use modhost_core::Result;
use modhost_db::{PublicGalleryImage, get_gallery_image, prelude::ProjectAuthors};
use modhost_db_util::{gallery::transform_gallery_image, projects::get_project};
use modhost_server_core::state::AppState;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, IntoActiveModel, ModelTrait};

/// Data for updating a gallery image.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema, ToResponse,
)]
pub struct PartialGalleryImage {
    /// The display name of the image.
    #[serde(default)]
    pub name: Option<String>,

    /// An optional markdown-formatted description.
    #[serde(default)]
    pub description: Option<String>,

    /// The order of this image.
    #[serde(default)]
    pub ordering: Option<i32>,
}

/// Update Gallery Image
///
/// Update gallery image metadata
#[utoipa::path(
    patch,
    path = "/{image}",
    tag = "Gallery",
    responses(
        (status = 200, description = "Updated gallery image!", body = PublicGalleryImage),
        (status = INTERNAL_SERVER_ERROR, description = "An internal error occured!"),
    ),
    params(
        ("image" = String, Path, description = "The gallery image ID."),
    ),
    request_body(content = PartialGalleryImage, description = "The information to update"),
    security(
        ("api_auth_token" = []),
    ),
)]
#[debug_handler]
pub async fn update_handler(
    jar: CookieJar,
    headers: HeaderMap,
    Path((project, image)): Path<(String, String)>,
    State(state): State<AppState>,
    Json(data): Json<PartialGalleryImage>,
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

    let mut img = img.into_active_model();

    if let Some(name) = data.name {
        img.name = Set(name);
    }

    if let Some(ordering) = data.ordering {
        img.ordering = Set(ordering);
    }

    if let Some(desc) = data.description {
        img.description = Set(Some(desc));
    }

    img.updated_at = Set(Utc::now().naive_utc());

    let img = img.update(&state.db).await?;

    Ok(Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::new(serde_json::to_string(&transform_gallery_image(
            img,
        ))?))?)
}
