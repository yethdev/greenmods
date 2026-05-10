//! Project GitHub sync routes and helpers.

use super::bad_request;
use axum::{
    Json, Router,
    body::{Body, Bytes},
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::{get, patch, post},
};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use hmac::{Hmac, Mac};
use modhost_auth::get_user_from_req;
use modhost_core::{AppError, Result};
use modhost_db::{
    Project, ProjectRepoSync, ProjectVersion,
    prelude::{ProjectAuthors, ProjectRepoSyncs, ProjectVersions},
    project_repo_syncs, project_versions, version_files,
};
use modhost_db_util::{projects::get_project, vers::get_latest_version};
use modhost_server_core::state::AppState;
use object_store::{ObjectStore, PutPayload};
use random_string::generate;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait,
    QueryFilter,
};
use semver::Version;
use sha1::{Digest, Sha1};
use sha2::Sha256;
use tokio::task;
use url::Url;

type HmacSha256 = Hmac<Sha256>;

/// Register GitHub sync routes.
/// Should be nested at `/api/v1/projects/{id}/github`.
pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(info_handler))
        .route("/", patch(update_handler))
        .route("/webhook", post(webhook_handler))
        .with_state(state)
}

/// Author-visible GitHub sync details for a project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, ToResponse)]
pub struct ProjectRepoSyncAdminData {
    /// The tracked GitHub repository owner.
    pub repo_owner: String,

    /// The tracked GitHub repository name.
    pub repo_name: String,

    /// The tracked default branch.
    pub default_branch: Option<String>,

    /// Whether README syncing is enabled.
    pub sync_readme: bool,

    /// Whether release syncing is enabled.
    pub sync_releases: bool,

    /// Whether FAQ syncing is enabled.
    pub sync_faq: bool,

    /// Whether repo links syncing is enabled.
    pub sync_links: bool,

    /// The most recent successful push sync time.
    pub last_push_sync_at: Option<chrono::NaiveDateTime>,

    /// The most recent successful release sync time.
    pub last_release_sync_at: Option<chrono::NaiveDateTime>,

    /// The last sync error, if any.
    pub last_error: Option<String>,

    /// Relative path to the project's webhook endpoint.
    pub webhook_path: String,

    /// The GitHub webhook secret.
    pub webhook_secret: String,
}

impl ProjectRepoSyncAdminData {
    fn from_models(project: &Project, sync: &ProjectRepoSync) -> Self {
        Self {
            repo_owner: sync.repo_owner.clone(),
            repo_name: sync.repo_name.clone(),
            default_branch: sync.default_branch.clone(),
            sync_readme: sync.sync_readme,
            sync_releases: sync.sync_releases,
            sync_faq: sync.sync_faq,
            sync_links: sync.sync_links,
            last_push_sync_at: sync.last_push_sync_at,
            last_release_sync_at: sync.last_release_sync_at,
            last_error: sync.last_error.clone(),
            webhook_path: format!("/api/v1/projects/{}/github/webhook", project.slug),
            webhook_secret: sync.webhook_secret.clone(),
        }
    }
}

/// A partial GitHub sync configuration update.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, ToResponse)]
pub struct PartialProjectRepoSync {
    /// Override the tracked default branch.
    #[serde(default)]
    pub default_branch: Option<String>,

    /// Whether README syncing is enabled.
    #[serde(default)]
    pub sync_readme: Option<bool>,

    /// Whether release syncing is enabled.
    #[serde(default)]
    pub sync_releases: Option<bool>,

    /// Whether FAQ syncing is enabled.
    #[serde(default)]
    pub sync_faq: Option<bool>,

    /// Whether repo links syncing is enabled.
    #[serde(default)]
    pub sync_links: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct InstallManifest {
    #[serde(default)]
    loaders: Vec<String>,
    #[serde(default, alias = "gameVersions", alias = "game_versions")]
    game_versions: Vec<String>,
    #[serde(default)]
    version_number: Option<String>,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct GitHubRepositoryOwner {
    login: String,
}

#[derive(Debug, Clone, Deserialize)]
struct GitHubRepositoryPayload {
    name: String,
    default_branch: Option<String>,
    owner: GitHubRepositoryOwner,
}

#[derive(Debug, Clone, Deserialize)]
struct PushPayload {
    #[serde(rename = "ref")]
    git_ref: String,
    repository: GitHubRepositoryPayload,
}

#[derive(Debug, Clone, Deserialize)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ReleasePayloadData {
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
    draft: bool,
    assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Clone, Deserialize)]
struct ReleasePayload {
    action: String,
    release: ReleasePayloadData,
    repository: GitHubRepositoryPayload,
}

struct RepoContentSync {
    readme: Option<String>,
    faq: Option<String>,
    links: Option<String>,
    install_json: Option<String>,
}

/// Get Project GitHub Sync
///
/// Get GitHub sync information for a project that is linked to a GitHub repository.
#[utoipa::path(
    get,
    path = "/",
    tag = "Projects",
    responses(
        (status = 200, description = "Method returned ok", body = ProjectRepoSyncAdminData),
        (status = INTERNAL_SERVER_ERROR, description = "An internal error occured!"),
    ),
    security(
        ("api_auth_token" = []),
    ),
)]
#[debug_handler]
pub async fn info_handler(
    jar: CookieJar,
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ProjectRepoSyncAdminData>> {
    let user = get_user_from_req(&jar, &headers, &state.db).await?;
    let project = get_project(id, &state.db).await?;

    ensure_author_access(&project, &user, &state).await?;

    let sync = ensure_repo_sync(&project, &state).await?;

    Ok(Json(ProjectRepoSyncAdminData::from_models(&project, &sync)))
}

/// Update Project GitHub Sync
///
/// Update GitHub sync settings for a project.
#[utoipa::path(
    patch,
    path = "/",
    tag = "Projects",
    request_body(content = PartialProjectRepoSync, description = "GitHub sync settings to update"),
    responses(
        (status = 200, description = "Method returned ok", body = ProjectRepoSyncAdminData),
        (status = INTERNAL_SERVER_ERROR, description = "An internal error occured!"),
    ),
    security(
        ("api_auth_token" = []),
    ),
)]
#[debug_handler]
pub async fn update_handler(
    jar: CookieJar,
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(data): Json<PartialProjectRepoSync>,
) -> Result<Response> {
    let user = get_user_from_req(&jar, &headers, &state.db).await?;
    let project = get_project(id, &state.db).await?;

    ensure_author_access(&project, &user, &state).await?;

    let mut sync = ensure_repo_sync(&project, &state).await?.into_active_model();

    if let Some(default_branch) = data.default_branch {
        let default_branch = default_branch.trim().to_string();

        if !default_branch.is_empty() {
            if default_branch.len() > 120 || default_branch.contains(' ') {
                return bad_request("Default branch must be 1 to 120 characters with no spaces.");
            }

            sync.default_branch = Set(Some(default_branch));
        }
    }

    if let Some(sync_readme) = data.sync_readme {
        sync.sync_readme = Set(sync_readme);
    }

    if let Some(sync_releases) = data.sync_releases {
        sync.sync_releases = Set(sync_releases);
    }

    if let Some(sync_faq) = data.sync_faq {
        sync.sync_faq = Set(sync_faq);
    }

    if let Some(sync_links) = data.sync_links {
        sync.sync_links = Set(sync_links);
    }

    let sync = sync.update(&state.db).await?;

    Ok(Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::new(serde_json::to_string(
            &ProjectRepoSyncAdminData::from_models(&project, &sync),
        )?))?)
}

/// GitHub Webhook
///
/// Receive GitHub push and release webhooks for a project.
#[utoipa::path(
    post,
    path = "/webhook",
    tag = "Projects",
    responses(
        (status = 200, description = "Webhook processed successfully!"),
        (status = INTERNAL_SERVER_ERROR, description = "An internal error occured!"),
    ),
)]
#[debug_handler]
pub async fn webhook_handler(
    Path(id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response> {
    let project = get_project(id, &state.db).await?;
    let sync = ProjectRepoSyncs::find_by_id(project.id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    verify_signature(&sync, &headers, &body)?;

    let event = headers
        .get("X-GitHub-Event")
        .and_then(|value| value.to_str().ok())
        .ok_or(AppError::MissingField("X-GitHub-Event".into()))?;

    let result = match event {
        "ping" => Ok("GitHub webhook connected.".to_string()),
        "push" => {
            let payload: PushPayload = serde_json::from_slice(&body)?;
            ensure_payload_repo(&sync, &payload.repository)?;
            handle_push_event(&project, &sync, payload, &state).await
        }
        "release" => {
            let payload: ReleasePayload = serde_json::from_slice(&body)?;
            ensure_payload_repo(&sync, &payload.repository)?;
            handle_release_event(&project, &sync, payload, &state).await
        }
        _ => Ok(format!("Ignored unsupported GitHub event: {event}")),
    };

    match result {
        Ok(message) => Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Body::new(message))?),
        Err(err) => {
            mark_sync_error(sync.project_id, err.to_string(), &state).await?;

            Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::new(err.to_string()))?)
        }
    }
}

pub(crate) async fn sync_source_config(
    project: &Project,
    source: Option<&str>,
    state: &AppState,
) -> Result<()> {
    let Some((repo_owner, repo_name)) = source.and_then(parse_github_repo) else {
        if let Some(sync) = ProjectRepoSyncs::find_by_id(project.id).one(&state.db).await? {
            sync.delete(&state.db).await?;
        }

        return Ok(());
    };

    let default_branch = fetch_default_branch(&repo_owner, &repo_name).await.ok().flatten();

    if let Some(sync) = ProjectRepoSyncs::find_by_id(project.id).one(&state.db).await? {
        let mut sync = sync.into_active_model();

        sync.repo_owner = Set(repo_owner);
        sync.repo_name = Set(repo_name);
        sync.default_branch = Set(default_branch);
        sync.last_error = Set(None);
        sync.update(&state.db).await?;
    } else {
        project_repo_syncs::ActiveModel {
            project_id: Set(project.id),
            repo_owner: Set(repo_owner),
            repo_name: Set(repo_name),
            default_branch: Set(default_branch),
            webhook_secret: Set(generate(48, "abcdefghijklmnopqrstuvwxyz0123456789")),
            sync_readme: Set(true),
            sync_releases: Set(true),
            sync_faq: Set(true),
            sync_links: Set(true),
            ..Default::default()
        }
        .insert(&state.db)
        .await?;
    }

    Ok(())
}

async fn ensure_author_access(project: &Project, user: &modhost_db::User, state: &AppState) -> Result<()> {
    let authors = project.find_related(ProjectAuthors).all(&state.db).await?;

    if !authors.iter().any(|author| author.user_id == user.id) && !user.admin {
        return Err(AppError::NoAccess);
    }

    Ok(())
}

async fn ensure_repo_sync(project: &Project, state: &AppState) -> Result<ProjectRepoSync> {
    if let Some(sync) = ProjectRepoSyncs::find_by_id(project.id).one(&state.db).await? {
        return Ok(sync);
    }

    sync_source_config(project, project.source.as_deref(), state).await?;

    ProjectRepoSyncs::find_by_id(project.id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)
}

fn parse_github_repo(source: &str) -> Option<(String, String)> {
    let parsed = Url::parse(source).ok()?;

    if parsed.host_str()? != "github.com" {
        return None;
    }

    let mut segments = parsed.path_segments()?.filter(|segment| !segment.is_empty());
    let owner = segments.next()?.to_string();
    let repo = segments
        .next()?
        .trim_end_matches(".git")
        .trim_end_matches('/')
        .to_string();

    if owner.is_empty() || repo.is_empty() {
        return None;
    }

    Some((owner, repo))
}

async fn fetch_default_branch(owner: &str, repo: &str) -> Result<Option<String>> {
    #[derive(Deserialize)]
    struct RepoInfo {
        default_branch: String,
    }

    let response = reqwest::Client::new()
        .get(format!("https://api.github.com/repos/{owner}/{repo}"))
        .header("User-Agent", "greenmods")
        .send()
        .await?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }

    Ok(Some(response.error_for_status()?.json::<RepoInfo>().await?.default_branch))
}

fn verify_signature(sync: &ProjectRepoSync, headers: &HeaderMap, body: &[u8]) -> Result<()> {
    let signature = headers
        .get("X-Hub-Signature-256")
        .and_then(|value| value.to_str().ok())
        .ok_or(AppError::MissingField("X-Hub-Signature-256".into()))?;

    let mut mac = HmacSha256::new_from_slice(sync.webhook_secret.as_bytes())
        .map_err(|_| AppError::Unknown)?;
    mac.update(body);

    let expected = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));

    if expected != signature {
        return Err(AppError::NoAccess);
    }

    Ok(())
}

fn ensure_payload_repo(sync: &ProjectRepoSync, repository: &GitHubRepositoryPayload) -> Result<()> {
    if repository.owner.login != sync.repo_owner || repository.name != sync.repo_name {
        return Err(AppError::NoAccess);
    }

    Ok(())
}

async fn handle_push_event(
    project: &Project,
    sync: &ProjectRepoSync,
    payload: PushPayload,
    state: &AppState,
) -> Result<String> {
    let project_id = project.id;
    let branch = payload.git_ref.trim_start_matches("refs/heads/").to_string();
    let tracked_branch = payload
        .repository
        .default_branch
        .clone()
        .or(sync.default_branch.clone())
        .unwrap_or_else(|| branch.clone());

    if branch != tracked_branch {
        mark_push_success(sync.project_id, Some(tracked_branch), state).await?;
        return Ok(format!("Ignored push to non-default branch: {branch}"));
    }

    let content = fetch_repo_content(&sync.repo_owner, &sync.repo_name, &tracked_branch).await?;
    let mut project = project.clone().into_active_model();

    if sync.sync_readme {
        if let Some(readme) = content.readme {
            project.readme = Set(readme);
        }
    }

    if sync.sync_faq {
        project.faq = Set(content.faq);
    }

    if sync.sync_links {
        project.repo_links = Set(content.links);
    }

    project.install_json = Set(content.install_json);
    project.updated_at = Set(Utc::now().naive_utc());
    project.update(&state.db).await?;

    state.search.update_project(project_id, &state.db).await?;
    mark_push_success(sync.project_id, Some(tracked_branch), state).await?;

    Ok("Synced GitHub repository metadata.".into())
}

async fn handle_release_event(
    project: &Project,
    sync: &ProjectRepoSync,
    payload: ReleasePayload,
    state: &AppState,
) -> Result<String> {
    if !sync.sync_releases {
        return Ok("Release syncing is disabled for this project.".into());
    }

    if payload.release.draft
        || !matches!(payload.action.as_str(), "published" | "released" | "prereleased")
    {
        return Ok(format!("Ignored GitHub release action: {}", payload.action));
    }

    let branch = payload
        .repository
        .default_branch
        .clone()
        .or(sync.default_branch.clone())
        .unwrap_or_else(|| "main".to_string());
    let install_manifest = fetch_install_manifest(&sync.repo_owner, &sync.repo_name, &branch).await?;
    let version_number = release_version_number(&payload.release, install_manifest.as_ref())?;

    if ProjectVersions::find()
        .filter(project_versions::Column::Project.eq(project.id))
        .filter(project_versions::Column::VersionNumber.eq(version_number.clone()))
        .one(&state.db)
        .await?
        .is_some()
    {
        mark_release_success(sync.project_id, Some(branch), state).await?;
        return Ok(format!("Version {version_number} already exists."));
    }

    let assets = payload
        .release
        .assets
        .into_iter()
        .filter(|asset| validate_release_file_name(&asset.name, state).is_none())
        .collect::<Vec<_>>();

    if assets.is_empty() {
        return Err(AppError::MissingField("release assets".into()));
    }

    let (loaders, game_versions) =
        resolve_release_matrix(project, install_manifest.as_ref(), state).await?;

    validate_known_values("loader", &loaders, &state.loaders.iter().map(|item| item.id.as_str()).collect::<Vec<_>>())?;
    validate_known_values(
        "game version",
        &game_versions,
        &state
            .game_versions
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>(),
    )?;

    let version_name = payload
        .release
        .name
        .clone()
        .or_else(|| install_manifest.as_ref().and_then(|manifest| manifest.name.clone()))
        .unwrap_or_else(|| format!("{} {}", project.name, version_number));

    let mut project_model = project.clone().into_active_model();
    project_model.updated_at = Set(Utc::now().naive_utc());
    project_model.update(&state.db).await?;

    let version = project_versions::ActiveModel {
        project: Set(project.id),
        name: Set(version_name),
        version_number: Set(version_number),
        changelog: Set(payload.release.body.clone()),
        loaders: Set(loaders),
        game_versions: Set(game_versions),
        downloads: Set(0),
        ..Default::default()
    }
    .insert(&state.db)
    .await?;

    for asset in assets {
        upload_release_asset(&version, asset, state).await?;
    }

    state.search.update_project(project.id, &state.db).await?;
    mark_release_success(sync.project_id, Some(branch), state).await?;

    Ok(format!("Synced GitHub release {}.", version.version_number))
}

async fn fetch_repo_content(owner: &str, repo: &str, branch: &str) -> Result<RepoContentSync> {
    let readme = fetch_optional_text(owner, repo, branch, &["README.md", "readme.md"]).await?;
    let faq = fetch_optional_text(owner, repo, branch, &[".openmods/FAQ.md", "FAQ.md"]).await?;
    let links =
        fetch_optional_text(owner, repo, branch, &[".openmods/LINKS.md", "LINKS.md"]).await?;
    let install_json = fetch_optional_text(
        owner,
        repo,
        branch,
        &[".openmods/install.json", "install.json"],
    )
    .await?;

    Ok(RepoContentSync {
        readme,
        faq,
        links,
        install_json,
    })
}

async fn fetch_install_manifest(
    owner: &str,
    repo: &str,
    branch: &str,
) -> Result<Option<InstallManifest>> {
    let Some(raw) = fetch_optional_text(owner, repo, branch, &[".openmods/install.json", "install.json"]).await? else {
        return Ok(None);
    };

    Ok(serde_json::from_str(&raw).ok())
}

async fn fetch_optional_text(
    owner: &str,
    repo: &str,
    branch: &str,
    candidates: &[&str],
) -> Result<Option<String>> {
    let client = reqwest::Client::new();

    for path in candidates {
        let response = client
            .get(format!(
                "https://raw.githubusercontent.com/{owner}/{repo}/{branch}/{path}"
            ))
            .header("User-Agent", "greenmods")
            .send()
            .await?;

        if response.status().is_success() {
            return Ok(Some(response.text().await?));
        }

        if response.status() != reqwest::StatusCode::NOT_FOUND {
            response.error_for_status()?;
        }
    }

    Ok(None)
}

fn release_version_number(
    release: &ReleasePayloadData,
    manifest: Option<&InstallManifest>,
) -> Result<String> {
    let version_number = manifest
        .and_then(|manifest| manifest.version_number.clone())
        .unwrap_or_else(|| release.tag_name.trim().trim_start_matches('v').to_string());

    Version::parse(&version_number)?;

    Ok(version_number)
}

async fn resolve_release_matrix(
    project: &Project,
    manifest: Option<&InstallManifest>,
    state: &AppState,
) -> Result<(Vec<String>, Vec<String>)> {
    let mut loaders = manifest
        .map(|manifest| clean_release_values(&manifest.loaders))
        .unwrap_or_default();
    let mut game_versions = manifest
        .map(|manifest| clean_release_values(&manifest.game_versions))
        .unwrap_or_default();

    if loaders.is_empty() || game_versions.is_empty() {
        if let Ok(latest) = get_latest_version(project, &state.db).await {
            if loaders.is_empty() {
                loaders = latest.loaders;
            }

            if game_versions.is_empty() {
                game_versions = latest.game_versions;
            }
        }
    }

    if loaders.is_empty() {
        return Err(AppError::MissingField("install.json loaders".into()));
    }

    if game_versions.is_empty() {
        return Err(AppError::MissingField("install.json game_versions".into()));
    }

    Ok((loaders, game_versions))
}

fn clean_release_values(values: &[String]) -> Vec<String> {
    values
        .iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect()
}

fn validate_known_values(field: &str, values: &[String], known: &[&str]) -> Result<()> {
    if values.len() > 32 {
        return Err(AppError::MissingField(format!("Too many {field}s")));
    }

    for value in values {
        if !known.is_empty() && !known.contains(&value.as_str()) {
            return Err(AppError::MissingField(format!("Unknown {field}: {value}")));
        }
    }

    Ok(())
}

fn validate_release_file_name(file_name: &str, state: &AppState) -> Option<String> {
    let name = file_name.trim();

    if name.is_empty() || name.len() > 180 {
        return Some("File name must be 1 to 180 characters.".into());
    }

    if name.contains('/') || name.contains('\\') || name.contains(':') {
        return Some("File name cannot contain path separators.".into());
    }

    let lowered = name.to_ascii_lowercase();

    if !state
        .config
        .ui
        .project_file_formats
        .iter()
        .any(|fmt| lowered.ends_with(&fmt.to_ascii_lowercase()))
    {
        return Some(format!(
            "File must use one of: {}",
            state.config.ui.project_file_formats.join(", ")
        ));
    }

    None
}

async fn upload_release_asset(
    version: &ProjectVersion,
    asset: ReleaseAsset,
    state: &AppState,
) -> Result<()> {
    let response = reqwest::Client::new()
        .get(&asset.browser_download_url)
        .header("User-Agent", "greenmods")
        .send()
        .await?
        .error_for_status()?;
    let bytes = response.bytes().await?;
    let verifier = state.verifier.clone();
    let verified = task::spawn_blocking({
        let bytes = bytes.clone();
        move || (*verifier)(bytes)
    })
    .await
    .map_err(|err| AppError::Io(std::io::Error::other(err.to_string())))?;

    if !verified {
        return Err(AppError::InvalidImageFile);
    }

    let mut hasher = Sha1::new();
    hasher.update(&bytes);

    let file_id = format!("{:x}", hasher.finalize());

    state
        .buckets
        .projects
        .put(
            &format!("/{file_id}").into(),
            PutPayload::from_bytes(bytes.clone()),
        )
        .await?;

    version_files::ActiveModel {
        file_name: Set(asset.name),
        sha1: Set(file_id.clone()),
        s3_id: Set(file_id),
        version_id: Set(version.id),
        size: Set(bytes.len() as i64),
        ..Default::default()
    }
    .insert(&state.db)
    .await?;

    Ok(())
}

async fn mark_push_success(project_id: i32, branch: Option<String>, state: &AppState) -> Result<()> {
    let sync = ProjectRepoSyncs::find_by_id(project_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;
    let mut sync = sync.into_active_model();
    sync.default_branch = Set(branch);
    sync.last_push_sync_at = Set(Some(Utc::now().naive_utc()));
    sync.last_error = Set(None);
    sync.update(&state.db).await?;
    Ok(())
}

async fn mark_release_success(project_id: i32, branch: Option<String>, state: &AppState) -> Result<()> {
    let sync = ProjectRepoSyncs::find_by_id(project_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;
    let mut sync = sync.into_active_model();
    sync.default_branch = Set(branch);
    sync.last_release_sync_at = Set(Some(Utc::now().naive_utc()));
    sync.last_error = Set(None);
    sync.update(&state.db).await?;
    Ok(())
}

async fn mark_sync_error(project_id: i32, message: String, state: &AppState) -> Result<()> {
    let sync = ProjectRepoSyncs::find_by_id(project_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;
    let mut sync = sync.into_active_model();
    sync.last_error = Set(Some(message));
    sync.update(&state.db).await?;
    Ok(())
}

/// The spec for the project GitHub sync API.
/// Should be nested at `/api/v1/projects/{id}/github`.
#[derive(OpenApi)]
#[openapi(paths(info_handler, update_handler, webhook_handler))]
pub struct ProjectGitHubApi;