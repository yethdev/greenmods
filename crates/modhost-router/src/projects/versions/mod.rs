//! Routes concerning project versions.

use axum::{
    Router,
    body::Body,
    http::StatusCode,
    response::Response,
    routing::{delete, get, patch, put},
};
use modhost_core::Result;
use modhost_server_core::state::AppState;
use std::collections::HashSet;

pub mod create;
pub mod delete;
pub mod download;
pub mod info;
pub mod latest;
pub mod list;
pub mod update;

/// Register project versions API routes.
/// Should be nested at `/api/v1/projects/{id}/versions`.
pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list::list_handler))
        .route("/", put(create::create_handler))
        .route("/latest", get(latest::latest_handler))
        .route("/{version}", get(info::info_handler))
        .route("/{version}", patch(update::update_handler))
        .route("/{version}", delete(delete::delete_handler))
        .route(
            "/{version}/download/{file}",
            get(download::download_handler),
        )
        .route(
            "/{version}/download/{file}/mod-only",
            get(download::mod_only_download_handler),
        )
        .with_state(state)
}

fn bad_request(msg: impl Into<String>) -> Result<Response> {
    Ok(Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::new(msg.into()))?)
}

fn split_csv(value: String) -> Vec<String> {
    value
        .split(',')
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect()
}

fn validate_file_name(file_name: &str, state: &AppState) -> Option<String> {
    let name = file_name.trim();

    if name.is_empty() || name.len() > 180 {
        return Some("File name must be 1 to 180 characters.".into());
    }

    if name.contains('/') || name.contains('\\') || name.contains(':') {
        return Some("File name cannot contain path separators.".into());
    }

    let allowed = &state.config.ui.project_file_formats;
    let lowered = name.to_ascii_lowercase();

    if !allowed
        .iter()
        .any(|fmt| lowered.ends_with(&fmt.to_ascii_lowercase()))
    {
        return Some(format!("File must use one of: {}", allowed.join(", ")));
    }

    None
}

fn validate_loaders(loaders: &[String], state: &AppState) -> Option<String> {
    let known = state
        .loaders
        .iter()
        .map(|loader| loader.id.as_str())
        .collect::<HashSet<_>>();

    validate_known_values("loader", loaders, &known)
}

fn validate_game_versions(versions: &[String], state: &AppState) -> Option<String> {
    let known = state
        .game_versions
        .iter()
        .map(|version| version.id.as_str())
        .collect::<HashSet<_>>();

    validate_known_values("game version", versions, &known)
}

fn validate_known_values(field: &str, values: &[String], known: &HashSet<&str>) -> Option<String> {
    if values.is_empty() {
        return Some(format!("At least one {field} is required."));
    }

    if values.len() > 32 {
        return Some(format!("Choose at most 32 {field}s."));
    }

    let mut seen = HashSet::new();

    for value in values {
        if value.trim() != value || value.is_empty() {
            return Some(format!("Invalid {field}: {value}"));
        }

        if !seen.insert(value) {
            return Some(format!("Duplicate {field}: {value}"));
        }

        if !known.is_empty() && !known.contains(value.as_str()) {
            return Some(format!("Unknown {field}: {value}"));
        }
    }

    None
}

/// The spec for the project versions API.
/// Should be nested at `/api/v1/projects/{id}/versions`.
#[derive(OpenApi)]
#[openapi(paths(
    create::create_handler,
    delete::delete_handler,
    download::download_handler,
    download::mod_only_download_handler,
    info::info_handler,
    list::list_handler,
    update::update_handler,
    latest::latest_handler,
))]
pub struct ProjectVersionsApi;
