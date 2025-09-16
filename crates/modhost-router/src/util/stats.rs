//! Statistics utilities.

use futures::StreamExt;
use modhost_core::{
    Result,
    info::{SysInfo, get_sys_info},
    uptime_secs,
};
use modhost_db::{
    DbConn,
    prelude::{GalleryImages, ProjectVersions, Projects, Users, VersionFiles},
};
use modhost_search::{Index, MeiliProject};
use modhost_server_core::state::AppState;
use object_store::{ObjectStore, aws::AmazonS3};
use once_cell::sync::Lazy;
use sea_orm::{EntityTrait, PaginatorTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::{
    sync::broadcast::{self, Receiver, Sender},
    task::JoinHandle,
    time::interval,
};

/// Whether to stop the stats thread.
pub const STOP_STATS_THREAD: AtomicBool = AtomicBool::new(false);

/// The stats channel.
pub static STATS_CHANNEL: Lazy<(Sender<AdminStats>, Receiver<AdminStats>)> =
    Lazy::new(|| broadcast::channel(1));

/// Stats for admins.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, ToResponse)]
pub struct AdminStats {
    /// The number of projects created.
    pub projects: u64,

    /// The number of versions created.
    pub versions: u64,

    /// The number of uploaded version files.
    pub files: u64,

    /// The number of uploaded gallery images.
    pub images: u64,

    /// The number of indexed projects in search.
    pub indexed_projects: u64,

    /// The number of users.
    pub users: u64,

    /// The instance uptime in seconds.
    pub uptime_secs: u64,

    /// The size of the projects bucket in bytes.
    pub projects_size_bytes: usize,

    /// The size of the gallery bucket in bytes.
    pub gallery_size_bytes: usize,

    /// Host system information.
    pub sys_info: SysInfo,
}

/// Start the admin stats fetcher thread.
pub fn start_stats_thread(state: &AppState) -> JoinHandle<Result<()>> {
    let projects_bucket = state.buckets.projects.clone();
    let gallery_bucket = state.buckets.gallery.clone();
    let search_projects = state.search.projects();
    let db = state.db.clone();
    let mut interval = interval(state.config.admin.stats_interval);

    tokio::spawn(async move {
        loop {
            if AtomicBool::load(&STOP_STATS_THREAD, Ordering::Relaxed) {
                break;
            }

            let stats =
                fetch_stats(&projects_bucket, &gallery_bucket, &search_projects, &db).await?;

            let _ = STATS_CHANNEL.0.send(stats);

            interval.tick().await;
        }

        info!("Exited stats thread");

        Ok(()) as Result<()>
    })
}

/// Fetch admin statistics.
pub async fn fetch_stats(
    projects_bucket: &AmazonS3,
    gallery_bucket: &AmazonS3,
    search_projects: &Index,
    conn: &DbConn,
) -> Result<AdminStats> {
    let mut stream = projects_bucket.list(None);
    let mut projects_size_bytes = 0;

    while let Some(obj) = stream.next().await {
        if let Ok(obj) = obj {
            projects_size_bytes += obj.size as usize;
        }
    }

    let mut stream = gallery_bucket.list(None);
    let mut gallery_size_bytes = 0;

    while let Some(obj) = stream.next().await {
        if let Ok(obj) = obj {
            gallery_size_bytes += obj.size as usize;
        }
    }

    Ok(AdminStats {
        projects: Projects::find().count(conn).await?,
        versions: ProjectVersions::find().count(conn).await?,
        files: VersionFiles::find().count(conn).await?,
        images: GalleryImages::find().count(conn).await?,
        indexed_projects: search_projects.get_documents::<MeiliProject>().await?.total as u64,
        users: Users::find().count(conn).await?,
        uptime_secs: uptime_secs(),
        projects_size_bytes,
        gallery_size_bytes,
        sys_info: get_sys_info(),
    })
}
