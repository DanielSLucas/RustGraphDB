use graphdb::lib::api::rest::run_server;
use graphdb::lib::storage::StorageManager;
use graphdb::lib::utils::logger::log_info;
use std::env;
use std::sync::{Arc, Mutex};
use ctrlc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the storage manager
    let storage_dir = "graph_storage";
    let storage_manager = Arc::new(Mutex::new(StorageManager::new(storage_dir)));

    // Set up signal handling for graceful shutdown
    let storage_manager_clone = Arc::clone(&storage_manager);
    ctrlc::set_handler(move || {
        log_info("\nGraceful shutdown initiated. Saving graphs...");
        let storage_manager = storage_manager_clone.lock().unwrap();
        storage_manager.save_all_graphs();
        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");

    run_server(storage_manager).await?;

    Ok(())
}
