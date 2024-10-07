use graphdb::lib::api::rest::run_server;
use graphdb::lib::storage::StorageManager;
use graphdb::lib::utils::logger::log_info;
use signal_hook::iterator::Signals;
use std::sync::atomic::AtomicBool;
use std::thread;
use std::sync::{Arc, Mutex};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the storage manager
    let storage_dir = "graph_storage";
    let storage_manager = Arc::new(Mutex::new(StorageManager::new(storage_dir)));

    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term))?;

    //let signals = Signals::new(&[signal_hook::SIGINT, signal_hook::SIGTERM]).unwrap();
    let signals_handle = signals.handle();

    thread::spawn(move || {
        for sig in signals.forever() {
            match sig {
                signal_hook::SIGINT | signal_hook::SIGTERM => {
                    println!("Received SIGINT, shutting down...");
                    log_info("\nGraceful shutdown initiated. Saving graphs...");
                    // Set up signal handling for graceful shutdown
                    let storage_manager_clone = Arc::clone(&storage_manager);
                    let storage_manager = storage_manager_clone.lock().unwrap();
                    storage_manager.save_all_graphs();
                    std::process::exit(0);
                    break;
                },
                _ => unreachable!(),
            }
        }
    });

    run_server(storage_manager).await?;

    Ok(())
}
