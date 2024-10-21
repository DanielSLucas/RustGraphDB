use std::collections::HashMap;
use tokio::sync::mpsc::Sender;

use super::disk_storage::DiskStorageManager;
use crate::lib::graph::edge::Edge;
use crate::lib::graph::node::Node;
use crate::lib::graph::Graph;

pub enum WriteTask {
  CreateGraph(String),
  AddNode { graph_name: String, node: Node },
  AddEdge { graph_name: String, edge: Edge },
}

pub struct StorageManager {
  graphs: HashMap<String, Graph>,
  disk_storage_manager: DiskStorageManager,
  queue_sender: Sender<WriteTask>, // Adição da fila
}

impl StorageManager {
  pub fn new(sender: Sender<WriteTask>) -> Self {
    Self {
      graphs: HashMap::new(),
      disk_storage_manager: DiskStorageManager::new(),
      queue_sender: sender,
    }
  }

  pub fn get_graph_names(&self) -> Vec<String> {
    return self.disk_storage_manager.get_graph_names();
  }

  pub async fn get_graph(&mut self, name: &str) -> Option<Graph> {
    {
      // First, try to read from the in-memory cache
      if let Some(graph) = self.graphs.get(name) {
        return Some(graph.clone());
      }
    }

    // If not in memory, try to load it from disk
    match self.disk_storage_manager.load_graph_from_file(name).await {
      Ok(graph) => {
        self.graphs.insert(name.to_string(), graph.clone());
        Some(graph)
      }
      Err(_) => None,
    }
  }

  pub async fn add_node(&self, graph_name: String, node: Node) -> std::io::Result<()> {
    let task = WriteTask::AddNode { graph_name, node };
    self.queue_sender.send(task).await.unwrap(); // Enviando tarefa para a fila
    Ok(())
  }

  pub async fn add_edge(&self, graph_name: String, edge: Edge) -> std::io::Result<()> {
    let task = WriteTask::AddEdge { graph_name, edge };
    self.queue_sender.send(task).await.unwrap(); // Enviando tarefa para a fila
    Ok(())
  }

  pub async fn save_graph(&mut self, graph: Graph) -> std::io::Result<()> {
    let name = graph.name().clone();

    let task = WriteTask::CreateGraph(name.clone());

    self.queue_sender.send(task).await.unwrap();

    for (_key, value) in graph.nodes().iter() {
      self.add_node(name.clone(), value.clone()).await?;
    }

    for (_key, value) in graph.edges().iter() {
      self.add_edge(name.clone(), value.clone()).await?;
    }

    self.graphs.insert(name.clone(), graph);

    Ok(())
  }
}
