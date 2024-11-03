use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::lib::graph::{edge::Edge, node::Node, Graph};

use super::{disk_storage::DiskStorage, in_memory_storage::InMemoryStorage};

pub struct StorageManager {
  disk_storage: DiskStorage,
  in_memory_storage: InMemoryStorage,
  write_queue_disk: Sender<WriteOperation>,
}

pub enum WriteOperation {
  CreateGraph(String, Graph),
  AddNode(String, Node),
  AddEdge(String, Edge),
  UpdateNode(String, Node),
  UpdateEdge(String, Edge),
  DeleteGraph(String),
  DeleteNode(String, usize),
  DeleteEdge(String, usize),
}

impl StorageManager {
  pub fn new() -> Self {
    let (write_queue_disk, write_queue_disk_rx) = mpsc::channel(100);

    let manager = Self {
      disk_storage: DiskStorage::new().expect("Failed to initialize disk storage"),
      in_memory_storage: InMemoryStorage::new(),
      write_queue_disk,
    };

    manager.start_write_workers(write_queue_disk_rx);

    manager
  }

  fn start_write_workers(&self, mut write_queue_disk_rx: Receiver<WriteOperation>) {
    tokio::spawn(async move {
      while let Some(operation) = write_queue_disk_rx.recv().await {
        DiskStorage::new()
          .expect("Failed to initialize disk storage")
          .process_write_operation(operation)
          .await;
      }
    });
  }

  pub async fn list_graph_names(&self) -> Vec<String> {
    let graph_names = self.in_memory_storage.list_graph_names().await;

    if graph_names.is_empty() {
      return match self.disk_storage.list_graph_names() {
        Ok(graphs) => graphs,
        Err(_) => Vec::new(),
      };
    }

    graph_names
  }

  pub async fn get_graph(&self, graph_name: &str) -> Option<Graph> {
    if let Some(graph) = self.in_memory_storage.get_graph(graph_name).await {
      Some(graph)
    } else {
      match self.disk_storage.get_graph(graph_name) {
        Ok(graph_option) => graph_option,
        Err(_) => None,
      }
    }
  }

  pub async fn create_graph(&self, graph_name: String) {
    let graph = self
      .in_memory_storage
      .create_graph(graph_name.clone())
      .await
      .unwrap();

    self
      .write_queue_disk
      .send(WriteOperation::CreateGraph(graph_name.clone(), graph))
      .await
      .unwrap();
  }

  pub async fn add_node(&self, graph_name: String, node: Node) {
    self
      .in_memory_storage
      .add_node(&graph_name, node.clone())
      .await
      .unwrap();

    self
      .write_queue_disk
      .send(WriteOperation::AddNode(graph_name, node))
      .await
      .unwrap();
  }

  pub async fn add_edge(&self, graph_name: String, edge: Edge) {
    self
      .in_memory_storage
      .add_edge(&graph_name, edge.clone())
      .await
      .unwrap();

    self
      .write_queue_disk
      .send(WriteOperation::AddEdge(graph_name, edge))
      .await
      .unwrap();
  }

  pub async fn delete_graph(&self, graph_name: String) {
    self
      .in_memory_storage
      .delete_graph(&graph_name)
      .await
      .unwrap();

    self
      .write_queue_disk
      .send(WriteOperation::DeleteGraph(graph_name))
      .await
      .unwrap();
  }

  pub async fn delete_node(&self, graph_name: String, node_id: usize) {
    self
      .in_memory_storage
      .delete_node(&graph_name, node_id)
      .await
      .unwrap();
    self
      .write_queue_disk
      .send(WriteOperation::DeleteNode(graph_name, node_id))
      .await
      .unwrap();
  }

  pub async fn delete_edge(&self, graph_name: String, edge_id: usize) {
    self
      .in_memory_storage
      .delete_edge(&graph_name, edge_id)
      .await
      .unwrap();
    self
      .write_queue_disk
      .send(WriteOperation::DeleteEdge(graph_name, edge_id))
      .await
      .unwrap();
  }

  pub async fn update_node(&self, graph_name: String, node: Node) {
    self
      .in_memory_storage
      .update_node(&graph_name, node.clone())
      .await
      .unwrap();
    self
      .write_queue_disk
      .send(WriteOperation::UpdateNode(graph_name, node))
      .await
      .unwrap();
  }

  pub async fn update_edge(&self, graph_name: String, edge: Edge) {
    self
      .in_memory_storage
      .update_edge(&graph_name, edge.clone())
      .await
      .unwrap();
    self
      .write_queue_disk
      .send(WriteOperation::UpdateEdge(graph_name, edge))
      .await
      .unwrap();
  }
}
