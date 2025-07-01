use std::{future::Future, time::Duration};

use revolt_config::{configure, capture_error};
use revolt_database::{Database, DatabaseInfo};
use revolt_result::Result;
use tasks::{file_deletion, prune_dangling_files};
use tokio::{join, time::sleep};

pub mod tasks;

pub async fn cron_task_wrapper<Fut: Future<Output = Result<()>>>(func: fn(Database) -> Fut, db: Database) {
    loop {
        if let Err(error) = func(db.clone()).await {
            log::error!("cron task failed unexpectidly: {error:?}\nRetrying after 60s");
            capture_error(&error);
        }

        sleep(Duration::from_secs(60)).await;
    }
}

#[tokio::main]
async fn main() {
    configure!(crond);

    let db = DatabaseInfo::Auto.connect().await.expect("database");

    join!(
        cron_task_wrapper(file_deletion::task, db.clone()),
        cron_task_wrapper(prune_dangling_files::task, db.clone()),
    );
}
