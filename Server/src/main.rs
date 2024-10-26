use std::sync::Arc;

use signal_hook::consts::signal::{SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use tokio::task;

use graphdb::lib::api::rest::run_server;
use graphdb::lib::storage::StorageManager;
use graphdb::lib::utils::logger::log_info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  let storage_manager = Arc::new(StorageManager::new());

  task::spawn_blocking(move || {
    let mut signals = Signals::new(&[SIGINT, SIGTERM]).unwrap();
    for sig in signals.forever() {
      match sig {
        SIGINT | SIGTERM => {
          log_info("Received SIGINT or SIGTERM, graceful shutdown initiated...");
          std::process::exit(0);
        }
        _ => unreachable!(),
      }
    }
  });

  run_server(storage_manager).await?;

  Ok(())
}
