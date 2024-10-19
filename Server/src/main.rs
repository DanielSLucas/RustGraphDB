use signal_hook::consts::signal::{SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use std::sync::Arc;
use tokio::task;

use graphdb::lib::api::rest::run_server;
use graphdb::lib::storage::StorageManager;
use graphdb::lib::utils::logger::log_info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  let storage_dir = "storage";
  let storage_manager = Arc::new(StorageManager::new(storage_dir));

  let storage_manager_clone = Arc::clone(&storage_manager);
  task::spawn_blocking(move || {
    let mut signals = Signals::new(&[SIGINT, SIGTERM]).unwrap();
    for sig in signals.forever() {
      match sig {
        SIGINT | SIGTERM => {
          println!("Received SIGINT or SIGTERM, shutting down...");
          log_info("\nGraceful shutdown initiated. Saving graphs...");

          storage_manager_clone.save_all_graphs_sync()
            .expect("Failed to save graphs");

          std::process::exit(0);
        }
        _ => unreachable!(),
      }
    }
  });

  run_server(storage_manager).await?;

  Ok(())
}
