//! The version download route.

use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, header},
    response::Response,
};
use axum_extra::extract::CookieJar;
use modhost_auth::get_user_from_req;
use modhost_core::AppError;
use modhost_core::Result;
use modhost_db::{ProjectVisibility, get_version};
use modhost_db_util::{projects::get_full_project, vers::get_version_file};
use modhost_server_core::state::AppState;
use object_store::ObjectStore;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, IntoActiveModel};
use std::{
    io::{Cursor, Read},
    path::Path as FsPath,
};
use zip::ZipArchive;

/// Download Project Version
///
/// Download a specific project version
#[utoipa::path(
    get,
    path = "/{version}/download/{file}",
    tag = "Versions",
    responses(
        (status = 307, description = "Redirecting to download"),
        (status = INTERNAL_SERVER_ERROR, description = "An internal error occured!"),
    ),
    params(
        ("version" = String, Path, description = "The version ID/name/number."),
        ("file" = String, Path, description = "The file ID/name."),
    ),
)]
#[debug_handler]
pub async fn download_handler(
    jar: CookieJar,
    headers: HeaderMap,
    Path((project, version, file)): Path<(String, String, String)>,
    State(state): State<AppState>,
) -> Result<Vec<u8>> {
    download_file_bytes(&jar, &headers, project, version, file, &state).await
}

/// Download only the mod payload from a single-file ZIP upload.
#[utoipa::path(
    get,
    path = "/{version}/download/{file}/mod-only",
    tag = "Versions",
    responses(
        (status = 200, description = "Downloading the only real file from the uploaded ZIP"),
        (status = 400, description = "The upload is not a ZIP with exactly one real file"),
        (status = INTERNAL_SERVER_ERROR, description = "An internal error occured!"),
    ),
    params(
        ("version" = String, Path, description = "The version ID/name/number."),
        ("file" = String, Path, description = "The file ID/name."),
    ),
)]
#[debug_handler]
pub async fn mod_only_download_handler(
    jar: CookieJar,
    headers: HeaderMap,
    Path((project, version, file)): Path<(String, String, String)>,
    State(state): State<AppState>,
) -> Result<Response> {
    let bytes = download_file_bytes(&jar, &headers, project, version, file, &state).await?;

    let Some((name, payload)) = single_real_zip_file(&bytes)? else {
        return super::bad_request("Mod-only download requires a ZIP with exactly one real file.");
    };

    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", header_safe_filename(&name)),
        )
        .body(Body::from(payload))?)
}

async fn download_file_bytes(
    jar: &CookieJar,
    headers: &HeaderMap,
    project: String,
    version: String,
    file: String,
    state: &AppState,
) -> Result<Vec<u8>> {
    let pkg = get_full_project(project, &state.db).await?;

    if pkg.visibility == ProjectVisibility::Private {
        match get_user_from_req(&jar, &headers, &state.db).await {
            Ok(user) => {
                if !pkg.authors.iter().any(|v| v.github_id == user.github_id) && !user.admin {
                    return Err(AppError::NotFound);
                }
            }

            Err(_) => return Err(AppError::NotFound),
        }
    }

    let ver = get_version(pkg.id, version, &state.db).await?;
    let file = get_version_file(ver.id, file, &state.db).await?;

    let pkg_dl = pkg.downloads;
    let ver_dl = ver.downloads;
    let mut pkg = pkg.into_project().into_active_model();

    pkg.downloads = Set(pkg_dl + 1);

    let mut ver = ver.into_active_model();

    ver.downloads = Set(ver_dl + 1);

    let pkg = pkg.update(&state.db).await?;

    ver.update(&state.db).await?;

    state.search.update_project(pkg.id, &state.db).await?;

    let bytes = state
        .buckets
        .projects
        .get(&format!("/{}", file.s3_id).into())
        .await?
        .bytes()
        .await?
        .to_vec();

    Ok(bytes)
}

fn single_real_zip_file(bytes: &[u8]) -> Result<Option<(String, Vec<u8>)>> {
    let mut archive = match ZipArchive::new(Cursor::new(bytes)) {
        Ok(archive) => archive,
        Err(_) => return Ok(None),
    };

    let mut found = None;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;

        if entry.is_dir() {
            continue;
        }

        let Some(path) = entry.enclosed_name() else {
            return Ok(None);
        };

        if ignored_archive_file(&path) {
            continue;
        }

        if found.is_some() {
            return Ok(None);
        }

        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("mod-file")
            .to_string();

        let mut payload = Vec::new();
        entry.read_to_end(&mut payload)?;

        found = Some((name, payload));
    }

    Ok(found)
}

fn ignored_archive_file(path: &FsPath) -> bool {
    let normalized = path.to_string_lossy().replace('\\', "/").to_ascii_lowercase();
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    normalized.starts_with("__macosx/")
        || matches!(
            file_name.as_str(),
            ".ds_store" | "thumbs.db" | "desktop.ini" | ".gitkeep"
        )
}

fn header_safe_filename(name: &str) -> String {
    name.chars()
        .map(|ch| match ch {
            '"' | '\\' | '\r' | '\n' => '_',
            _ => ch,
        })
        .collect()
}
