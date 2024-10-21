use signal_hook::consts::signal::{SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use tokio::sync::mpsc::channel;
use tokio::sync::RwLock;
use tokio::task;

use graphdb::lib::api::rest::run_server;
use graphdb::lib::storage::disk_storage::DiskStorageManager;
use graphdb::lib::storage::{StorageManager, WriteTask};
use graphdb::lib::utils::logger::log_info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  let (sender, mut receiver) = channel::<WriteTask>(32);
  let storage_manager = RwLock::new(StorageManager::new(sender));

  // Worker para processar as escritas em disco
  tokio::spawn(async move {
    let disk_storage_manager = DiskStorageManager::new();

    while let Some(task) = receiver.recv().await {
      match task {
        WriteTask::CreateGraph(graph_name) => {
          let _ = disk_storage_manager.create_graph_dir(&graph_name).await;
        }
        WriteTask::AddNode { graph_name, node } => {
          let _ = disk_storage_manager
            .add_node_to_file(&graph_name, &node)
            .await;
        }
        WriteTask::AddEdge { graph_name, edge } => {
          let _ = disk_storage_manager
            .add_edge_to_file(&graph_name, &edge)
            .await;
        }
      }
    }
  });

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
