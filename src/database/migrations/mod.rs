use super::get_connection;

pub mod init;
pub mod scripts;

pub async fn run_migrations() {
    let client = get_connection();

    let list = client
        .list_database_names(None, None)
        .await
        .expect("Failed to fetch database names.");
    
    if list.iter().position(|x| x == "revolt").is_none() {
        init::create_database().await;
    } else {
        scripts::migrate_database().await;
    }
    
    // panic!("https://pbs.twimg.com/media/EDTpB5JWwAUvyxd.jpg");
    rauth::entities::sync_models(&super::get_db()).await;
}
