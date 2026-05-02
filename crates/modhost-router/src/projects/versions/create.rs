//! The version create route.

use super::{bad_request, split_csv, validate_file_name, validate_game_versions, validate_loaders};
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
use modhost_db::{ProjectVersion, prelude::ProjectAuthors, project_versions, version_files};
use modhost_db_util::projects::get_project;
use modhost_server_core::state::AppState;
use object_store::{ObjectStore, PutPayload};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, IntoActiveModel, ModelTrait};
use semver::Version;
use sha1::{Digest, Sha1};

/// The initial data for creating a new project version.
/// This should be formatted as "multipart/form-data".
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, ToSchema, ToResponse)]
pub struct ProjectVersionInit {
    /// The name of the project version.
    pub name: String,

    /// The version number.
    pub version_number: String,

    /// An optional changelog.
    pub changelog: Option<String>,

    /// A list of loaders this version works on.
    /// This should be a comma-separated list in the request.
    pub loaders: String,

    /// A list of game versions this works on.
    /// This should be a comma-separated list in the request.
    pub game_versions: String,

    /// The file name.
    pub file_name: String,

    /// The file content.
    #[schema(content_media_type = "application/octet-stream")]
    pub file: Vec<u8>,
}

/// Upload Project Version
///
/// Upload a new project version
#[utoipa::path(
    put,
    path = "/",
    tag = "Versions",
    responses(
        (status = 200, description = "Created project version!", body = ProjectVersion),
        (status = INTERNAL_SERVER_ERROR, description = "An internal error occured!"),
    ),
    request_body(content = inline(ProjectVersionInit), description = "The version data", content_type = "multipart/form-data"),
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
    let mut version_number = None;
    let mut changelog = None;
    let mut loaders = None;
    let mut game_versions = None;
    let mut file = None;
    let mut file_name = None;

    while let Ok(Some(field)) = data.next_field().await {
        match field.name().ok_or(AppError::MissingFieldName)? {
            "name" => name = Some(field.text().await?),
            "version_number" => version_number = Some(field.text().await?),
            "changelog" => changelog = Some(field.text().await?),
            "loaders" => loaders = Some(split_csv(field.text().await?)),
            "game_versions" => game_versions = Some(split_csv(field.text().await?)),
            "file" => file = Some(field.bytes().await?),
            "file_name" => file_name = Some(field.text().await?),
            _ => {}
        }
    }

    if name.is_none() {
        Err(AppError::MissingField("name".into()))?;
    }

    if version_number.is_none() {
        Err(AppError::MissingField("version_number".into()))?;
    }

    if loaders.is_none() {
        Err(AppError::MissingField("loaders".into()))?;
    }

    if game_versions.is_none() {
        Err(AppError::MissingField("game_versions".into()))?;
    }

    if file.is_none() {
        Err(AppError::MissingField("file".into()))?;
    }

    if file_name.is_none() {
        Err(AppError::MissingField("file_name".into()))?;
    }

    let name = name.unwrap();
    let version_number = version_number.unwrap();
    let loaders = loaders.unwrap();
    let game_versions = game_versions.unwrap();
    let file = file.unwrap();
    let file_name = file_name.unwrap().trim().to_string();

    Version::parse(&version_number)?;

    if let Some(err) = validate_loaders(&loaders, &state) {
        return bad_request(err);
    }

    if let Some(err) = validate_game_versions(&game_versions, &state) {
        return bad_request(err);
    }

    if let Some(err) = validate_file_name(&file_name, &state) {
        return bad_request(err);
    }

    if !(state.verifier)(file.clone()) {
        return bad_request("File did not pass greenmods upload validation.");
    }

    let mut hasher = Sha1::new();

    hasher.update(&file);

    let file_id = format!("{:x}", hasher.finalize());
    let file_size = file.len() as i64;

    state
        .buckets
        .projects
        .put(
            &format!("/{}", file_id).into(),
            PutPayload::from_bytes(file),
        )
        .await?;

    let data = project_versions::ActiveModel {
        project: Set(pkg.id),
        name: Set(name),
        version_number: Set(version_number),
        changelog: Set(changelog),
        loaders: Set(loaders),
        game_versions: Set(game_versions),
        downloads: Set(0),
        ..Default::default()
    };

    let mut pkg = pkg.into_active_model();

    pkg.updated_at = Set(Utc::now().naive_utc());

    let pkg = pkg.update(&state.db).await?;
    let ver = data.insert(&state.db).await?;

    let file = version_files::ActiveModel {
        file_name: Set(file_name),
        sha1: Set(file_id.clone()),
        s3_id: Set(file_id),
        version_id: Set(ver.id),
        size: Set(file_size),
        ..Default::default()
    };

    file.insert(&state.db).await?;

    state.search.update_project(pkg.id, &state.db).await?;

    Ok(Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::new(serde_json::to_string(&ver)?))?)
}
