use std::sync::{Arc, Mutex};

use graphdb::lib::api::rest::run_server;
use graphdb::lib::storage::StorageManager;
use graphdb::lib::utils::logger::log_info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the storage manager
    let storage_dir = "storage";
    let storage_manager = Arc::new(Mutex::new(StorageManager::new(storage_dir)));
    
    let storage_manager_clone: Arc<Mutex<StorageManager>> = Arc::clone(&storage_manager);
    std::thread::spawn(move || {
        use signal_hook::consts::signal::{SIGTERM, SIGINT};
        use signal_hook::iterator::Signals;

        let mut signals = Signals::new(&[SIGINT, SIGTERM]).unwrap();
        for sig in signals.forever(){
            match sig {
                SIGINT | SIGTERM => {
                    println!("Received SIGINT, shutting down...");
                    log_info("\nGraceful shutdown initiated. Saving graphs...");
                    // Set up signal handling for graceful shutdown
                    let storage_manager = storage_manager_clone.lock().unwrap();
                    storage_manager.save_all_graphs();
                    std::process::exit(0);
                },
                _ => unreachable!()
            }
        }
    });
    
    run_server(storage_manager).await?;

    Ok(())
}
