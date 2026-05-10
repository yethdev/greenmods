//! Routes concerning project collections.

pub mod create;
pub mod delete;
pub mod info;
pub mod list;
pub mod update;

use axum::{
    Router,
    body::Body,
    http::StatusCode,
    response::Response,
    routing::{delete, get, patch, put},
};
use modhost_db::{ProjectVisibility, User};
use modhost_db_util::projects::{ProjectUtils, get_full_project};
use modhost_server_core::state::AppState;
use std::collections::HashSet;

const MAX_COLLECTION_PROJECTS: usize = 128;

/// Register collection-related routes.
/// This should be nested at `/api/v1/collections`.
pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list::list_handler))
        .route("/", put(create::create_handler))
        .route("/{id}", get(info::info_handler))
        .route("/{id}", patch(update::update_handler))
        .route("/{id}", delete(delete::delete_handler))
        .with_state(state)
}

fn bad_request(msg: impl Into<String>) -> modhost_core::Result<Response> {
    Ok(Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::new(msg.into()))?)
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

fn validate_collection_name(name: &str) -> Option<&'static str> {
    if !(3..=80).contains(&name.chars().count()) {
        return Some("Collection name must be 3 to 80 characters.");
    }

    None
}

fn validate_collection_description(description: &str) -> Option<&'static str> {
    if !(10..=240).contains(&description.chars().count()) {
        return Some("Collection summary must be 10 to 240 characters.");
    }

    None
}

fn validate_collection_readme(readme: &str) -> Option<&'static str> {
    let len = readme.trim().chars().count();

    if !(20..=100_000).contains(&len) {
        return Some("Collection readme must be 20 to 100000 characters.");
    }

    None
}

async fn resolve_project_ids(
    project_refs: &[String],
    user: &User,
    state: &AppState,
) -> std::result::Result<Vec<i32>, String> {
    if project_refs.is_empty() {
        return Err("Add at least one project to the collection.".into());
    }

    if project_refs.len() > MAX_COLLECTION_PROJECTS {
        return Err(format!(
            "Collections can include at most {MAX_COLLECTION_PROJECTS} projects."
        ));
    }

    let mut seen = HashSet::new();
    let mut project_ids = Vec::with_capacity(project_refs.len());

    for project_ref in project_refs {
        let project_ref = project_ref.trim();

        if project_ref.is_empty() {
            return Err("Project references cannot be empty.".into());
        }

        if !seen.insert(project_ref.to_string()) {
            return Err(format!("Duplicate project: {project_ref}"));
        }

        let project = get_full_project(project_ref, &state.db)
            .await
            .map_err(|_| format!("Unknown project: {project_ref}"))?;

        if !user.admin
            && !project
                .is_visible_to(user, &state.db)
                .await
                .map_err(|_| format!("Failed to validate project: {project_ref}"))?
        {
            return Err(format!("You do not have access to project: {project_ref}"));
        }

        project_ids.push(project.id);
    }

    Ok(project_ids)
}

fn is_visible_to_viewer(collection_visibility: ProjectVisibility, owner_id: i32, user: Option<&User>) -> bool {
    matches!(collection_visibility, ProjectVisibility::Public)
        || user
            .map(|user| user.admin || user.id == owner_id)
            .unwrap_or(false)
}

/// The spec for the collections API.
/// Should be nested at `/api/v1/collections`.
#[derive(OpenApi)]
#[openapi(paths(
    create::create_handler,
    delete::delete_handler,
    info::info_handler,
    list::list_handler,
    update::update_handler,
))]
pub struct CollectionsApi;