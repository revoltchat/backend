//! Semi-important background task management

use crate::Database;

use async_std::task;
use std::time::Instant;

const WORKER_COUNT: usize = 5;

pub mod ack;
pub mod last_message_id;
pub mod process_embeds;
pub mod web_push;

/// Spawn background workers
pub async fn start_workers(db: Database) {
    for _ in 0..WORKER_COUNT {
        task::spawn(ack::worker(db.clone()));
        task::spawn(last_message_id::worker(db.clone()));
        task::spawn(process_embeds::worker(db.clone()));
        task::spawn(web_push::worker(db.clone().into()));
    }
}

/// Task with additional information on when it should run
pub struct DelayedTask<T> {
    pub data: T,
    last_updated: Instant,
    first_seen: Instant,
}

/// Commit to database every 30 seconds if the task is particularly active.
static EXPIRE_CONSTANT: u64 = 30;

/// Otherwise, commit to database after 5 seconds.
static SAVE_CONSTANT: u64 = 5;

impl<T> DelayedTask<T> {
    /// Create a new delayed task
    pub fn new(data: T) -> Self {
        DelayedTask {
            data,
            last_updated: Instant::now(),
            first_seen: Instant::now(),
        }
    }

    /// Push a task further back in time
    pub fn delay(&mut self) {
        self.last_updated = Instant::now()
    }

    /// Check if a task should run yet
    pub fn should_run(&self) -> bool {
        self.first_seen.elapsed().as_secs() > EXPIRE_CONSTANT
            || self.last_updated.elapsed().as_secs() > SAVE_CONSTANT
    }
}
