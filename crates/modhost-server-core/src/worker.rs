//! ModHost's worker, providing token invalidation.

use chrono::Utc;
use jsglue::abort::ABORT_HANDLES;
use modhost_core::Result;
use modhost_db::{DbConn, DbPool, prelude::UserTokens};
use sea_orm::{EntityTrait, ModelTrait};
use tokio::task::JoinHandle;

/// Start the worker service and get a handle to its thread.
pub fn run_worker(pool: DbPool) -> JoinHandle<Result<()>> {
    info!("Starting worker...");

    let handle = tokio::spawn(async move { worker_loop(pool).await });
    let abort = handle.abort_handle();

    // Hook into Glue's exit handler.
    ABORT_HANDLES.lock().unwrap().push(abort);

    handle
}

/// The internal worker loop.
/// This function will never return unless an error occurs.
pub async fn worker_loop(db: DbConn) -> Result<()> {
    loop {
        let tkns = UserTokens::find().all(&db).await?;

        for token in tkns {
            let time = Utc::now().timestamp_millis();

            if time >= token.expires.and_utc().timestamp_millis() {
                info!("Found expired token (id: {}). Deleting...", token.id);

                token.delete(&db).await?;
            }
        }
    }
}
