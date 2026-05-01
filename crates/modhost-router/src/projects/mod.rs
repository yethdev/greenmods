//! Routes concerning projects.

pub mod authors;
pub mod create;
pub mod delete;
pub mod gallery;
pub mod info;
pub mod search;
pub mod update;
pub mod versions;

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
use url::Url;

const MAX_TAGS: usize = 12;
const REQUIRED_TEST_TAG: &str = "tested";

/// Register project-related routes onto the router.
/// This should be nested at `/api/v1/projects`.
pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", put(create::create_handler))
        .route("/search", get(search::search_handler))
        .route("/{id}", get(info::info_handler))
        .route("/{id}", patch(update::update_handler))
        .route("/{id}", delete(delete::delete_handler))
        .nest("/{id}/authors", authors::router(state.clone()))
        .nest("/{id}/gallery", gallery::router(state.clone()))
        .nest("/{id}/versions", versions::router(state.clone()))
        .with_state(state)
}

fn bad_request(msg: impl Into<String>) -> Result<Response> {
    Ok(Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::new(msg.into()))?)
}

fn clean_tags(tags: &[String]) -> Vec<String> {
    tags.iter()
        .map(|tag| tag.trim().to_ascii_lowercase())
        .filter(|tag| !tag.is_empty())
        .collect()
}

fn clean_link(field: &str, raw: Option<String>) -> std::result::Result<Option<String>, String> {
    let Some(value) = raw
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };

    if value.len() > 300 {
        return Err(format!("{field} URL is too long."));
    }

    let parsed = Url::parse(&value).map_err(|_| format!("{field} must be a valid URL."))?;

    match parsed.scheme() {
        "https" | "http" => Ok(Some(value)),
        _ => Err(format!("{field} must use http or https.")),
    }
}

fn validate_slug(slug: &str) -> Option<&'static str> {
    if !(3..=64).contains(&slug.len()) {
        return Some("Slug must be 3 to 64 characters.");
    }

    if slug.starts_with('-') || slug.ends_with('-') {
        return Some("Slug cannot start or end with a dash.");
    }

    if !slug
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        return Some("Slug can only use lowercase letters, numbers, and dashes.");
    }

    None
}

fn validate_project_name(name: &str) -> Option<&'static str> {
    if !(3..=80).contains(&name.chars().count()) {
        return Some("Project name must be 3 to 80 characters.");
    }

    None
}

fn validate_project_description(description: &str) -> Option<&'static str> {
    if !(10..=240).contains(&description.chars().count()) {
        return Some("Summary must be 10 to 240 characters.");
    }

    None
}

fn validate_project_readme(readme: &str) -> Option<&'static str> {
    let len = readme.trim().chars().count();

    if !(20..=100_000).contains(&len) {
        return Some("Readme must be 20 to 100000 characters.");
    }

    None
}

fn validate_project_tags(tags: &[String], state: &AppState) -> Option<String> {
    if tags.is_empty() {
        return Some("At least one tag is required.".into());
    }

    if tags.len() > MAX_TAGS {
        return Some(format!("Choose at most {MAX_TAGS} tags."));
    }

    if !tags.iter().any(|tag| tag == REQUIRED_TEST_TAG) {
        return Some("The tested tag is required.".into());
    }

    let mut seen = HashSet::new();

    for tag in tags {
        if !seen.insert(tag) {
            return Some(format!("Duplicate tag: {tag}"));
        }
    }

    if !state.tags.is_empty() {
        let known = state
            .tags
            .iter()
            .map(|tag| tag.id.as_str())
            .collect::<HashSet<_>>();

        for tag in tags {
            if !known.contains(tag.as_str()) {
                return Some(format!("Unknown tag: {tag}"));
            }
        }
    }

    None
}

/// The spec for the projects API.
/// Should be nested at `/api/v1/projects`.
#[derive(OpenApi)]
#[openapi(
    paths(
        create::create_handler,
        delete::delete_handler,
        info::info_handler,
        search::search_handler,
        update::update_handler,
    ),
    nest(
        (path = "/{id}/authors", api = authors::ProjectAuthorsApi),
        (path = "/{id}/gallery", api = gallery::ProjectGalleryApi),
        (path = "/{id}/versions", api = versions::ProjectVersionsApi),
    ),
)]
pub struct ProjectsApi;
