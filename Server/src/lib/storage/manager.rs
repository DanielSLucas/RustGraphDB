use std::sync::Arc;

use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::lib::graph::{edge::Edge, node::Node, Graph};

use super::{disk_storage::DiskStorage, in_memory_storage::InMemoryStorage};

pub struct StorageManager {
  disk_storage: DiskStorage,
  in_memory_storage: Arc<InMemoryStorage>,
  write_queue_mem: Sender<WriteOperation>,
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
    let (write_queue_mem, write_queue_mem_rx) = mpsc::channel(100);
    let (write_queue_disk, write_queue_disk_rx) = mpsc::channel(100);

    let manager = Self {
      disk_storage: DiskStorage::new().expect("Failed to initialize disk storage"),
      in_memory_storage: Arc::new(InMemoryStorage::new()),
      write_queue_mem,
      write_queue_disk,
    };

    manager.start_write_workers(write_queue_mem_rx, write_queue_disk_rx);

    manager
  }

  fn start_write_workers(
    &self,
    mut write_queue_mem_rx: Receiver<WriteOperation>,
    mut write_queue_disk_rx: Receiver<WriteOperation>,
  ) {
    let in_memory_storage_clone = Arc::clone(&self.in_memory_storage);

    tokio::spawn(async move {
      while let Some(operation) = write_queue_mem_rx.recv().await {
        in_memory_storage_clone
          .process_write_operation(operation)
          .await;
      }
    });

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

  pub async fn create_graph(&self, graph_name: String, graph: Graph) {
    self
      .write_queue_mem
      .send(WriteOperation::CreateGraph(
        graph_name.clone(),
        graph.clone(),
      ))
      .await
      .unwrap();
    self
      .write_queue_disk
      .send(WriteOperation::CreateGraph(
        graph_name.clone(),
        graph.clone(),
      ))
      .await
      .unwrap();
  }

  pub async fn add_node(&self, graph_name: String, node: Node) {
    self
      .write_queue_mem
      .send(WriteOperation::AddNode(graph_name.clone(), node.clone()))
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
      .write_queue_mem
      .send(WriteOperation::AddEdge(graph_name.clone(), edge.clone()))
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
      .write_queue_mem
      .send(WriteOperation::DeleteGraph(graph_name.clone()))
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
      .write_queue_mem
      .send(WriteOperation::DeleteNode(graph_name.clone(), node_id))
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
      .write_queue_mem
      .send(WriteOperation::DeleteEdge(graph_name.clone(), edge_id))
      .await
      .unwrap();
    self
      .write_queue_disk
      .send(WriteOperation::DeleteEdge(graph_name, edge_id))
      .await
      .unwrap();
  }
}
