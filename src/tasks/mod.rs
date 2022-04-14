//! Semi-important background task management

use async_std::task;
use revolt_quark::Database;

const WORKER_COUNT: usize = 10;

pub mod process_embeds;

/// Spawn background workers
pub async fn start_workers(db: Database) {
    for _ in 0..WORKER_COUNT {
        task::spawn(process_embeds::worker(db.clone()));
    }
}
