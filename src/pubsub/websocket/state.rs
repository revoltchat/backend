use crate::pubsub::hive;

use many_to_many::ManyToMany;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use log::{error, info};
use ws::Sender;

lazy_static! {
    static ref CONNECTIONS: Arc<RwLock<HashMap<String, Sender>>> =
        Arc::new(RwLock::new(HashMap::new()));
    static ref CLIENTS: Arc<RwLock<ManyToMany<String, String>>> =
        Arc::new(RwLock::new(ManyToMany::new()));
}

pub fn accept(id: String, user_id: String, sender: Sender) -> Result<(), String> {
    let mut conns = CONNECTIONS
        .write()
        .map_err(|_| "Failed to lock connections for writing.")?;
    
    conns.insert(id.clone(), sender);

    let mut clients = CLIENTS
        .write()
        .map_err(|_| "Failed to lock clients for writing.")?;
    
    clients.insert(user_id.clone(), id.clone());

    info!("Accepted user [{}] for connection {}.", user_id, id);
    Ok(())
}

pub fn drop(id: &String) -> Result<(), String> {
    let mut conns = CONNECTIONS
        .write()
        .map_err(|_| "Failed to lock connections for writing.")?;
    
    conns.remove(id);

    let mut clients = CLIENTS
        .write()
        .map_err(|_| "Failed to lock clients for writing.")?;
    
    let uid = if let Some(ids) = clients.get_right(id) {
        let user_id: String = ids.into_iter().next().unwrap();
        info!("Dropped user [{}] for connection {}.", user_id, id);
        Some(user_id)
    } else {
        None
    };

    clients.remove_right(id);

    if let Some(user_id) = &uid {
        if let None = clients.get_left(user_id) {
            if let Err(error) = hive::drop_user(user_id) {
                error!("Failed to drop user from hive! {}", error);
            } else {
                info!("User [{}] has completed disconnected from node.", user_id);
            }
        }
    }

    Ok(())
}

pub fn publish(clients: Vec<String>, data: String) -> Result<(), String> {
    let conns = CONNECTIONS
        .read()
        .map_err(|_| "Failed to lock connections for reading.")?;

    let client_map = CLIENTS
        .read()
        .map_err(|_| "Failed to lock clients for reading.")?;
    
    for client in clients {
        if let Some(targets) = client_map.get_left(&client) {
            for target in &targets {
                if let Some(connection) = conns.get(target) {
                    if let Err(err) = connection.send(data.clone()) {
                        error!("Failed to publish notification to client [{}]! {}", target, err);
                    }
                }
            }
        }
    }

    Ok(())
}
