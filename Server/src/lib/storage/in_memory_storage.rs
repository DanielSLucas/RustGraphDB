use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::lib::{
  errors::graph_error::GraphError,
  graph::{edge::Edge, node::Node, Graph},
};

use super::manager::WriteOperation;

pub struct InMemoryStorage {
  graphs: RwLock<HashMap<String, Graph>>,
}

impl InMemoryStorage {
  pub fn new() -> Self {
    Self {
      graphs: RwLock::new(HashMap::new()),
    }
  }

  pub async fn process_write_operation(&self, operation: WriteOperation) {
    match operation {
      WriteOperation::CreateGraph(graph_name, graph) => {
        let _ = self.create_graph(graph_name, graph).await;
      }
      WriteOperation::AddNode(graph_name, node) => {
        let _ = self.add_node(&graph_name, node).await;
      }
      WriteOperation::AddEdge(graph_name, edge) => {
        let _ = self.add_edge(&graph_name, edge).await;
      }
      WriteOperation::UpdateNode(graph_name, node) => {
        let _ = self.update_node(&graph_name, node).await;
      }
      WriteOperation::UpdateEdge(graph_name, edge) => {
        let _ = self.update_edge(&graph_name, edge).await;
      }
      WriteOperation::DeleteGraph(graph_name) => {
        let _ = self.delete_graph(&graph_name).await;
      }
      WriteOperation::DeleteNode(graph_name, node_id) => {
        let _ = self.delete_node(&graph_name, node_id).await;
      }
      WriteOperation::DeleteEdge(graph_name, edge_id) => {
        let _ = self.delete_edge(&graph_name, edge_id).await;
      }
    }
  }

  pub async fn list_graph_names(&self) -> Vec<String> {
    let graphs = self.graphs.read().await;
    graphs.keys().cloned().collect()
  }

  pub async fn create_graph(&self, graph_name: String, grafo: Graph) -> Result<(), ()> {
    let mut graphs = self.graphs.write().await;
    graphs.insert(graph_name, grafo);
    Ok(())
  }

  pub async fn get_graph(&self, graph_name: &str) -> Option<Graph> {
    let graphs = self.graphs.read().await;
    graphs.get(graph_name).cloned()
  }

  pub async fn add_node(&self, graph_name: &str, node: Node) -> Result<(), GraphError> {
    let mut graphs = self.graphs.write().await;
    if let Some(graph) = graphs.get_mut(graph_name) {
      graph.add_node(node);
      Ok(())
    } else {
      Err(GraphError::StorageError(format!(
        "Graph '{}' not found.",
        graph_name
      )))
    }
  }

  pub async fn add_edge(&self, graph_name: &str, edge: Edge) -> Result<(), GraphError> {
    let mut graphs = self.graphs.write().await;
    if let Some(graph) = graphs.get_mut(graph_name) {
      graph.add_edge(edge);
      Ok(())
    } else {
      Err(GraphError::StorageError(format!(
        "Graph '{}' not found.",
        graph_name
      )))
    }
  }

  pub async fn delete_graph(&self, graph_name: &str) -> Result<(), GraphError> {
    let mut graphs = self.graphs.write().await;
    if graphs.remove(graph_name).is_some() {
      Ok(())
    } else {
      Err(GraphError::StorageError(format!(
        "Graph '{}' not found.",
        graph_name
      )))
    }
  }

  pub async fn delete_node(&self, graph_name: &str, node_id: usize) -> Result<(), GraphError> {
    let mut graphs = self.graphs.write().await;
    if let Some(graph) = graphs.get_mut(graph_name) {
      graph.delete_node(node_id);
      Ok(())
    } else {
      Err(GraphError::StorageError(format!(
        "Graph '{}' not found.",
        graph_name
      )))
    }
  }

  pub async fn delete_edge(&self, graph_name: &str, edge_id: usize) -> Result<(), GraphError> {
    let mut graphs = self.graphs.write().await;
    if let Some(graph) = graphs.get_mut(graph_name) {
      graph.delete_edge(edge_id);
      Ok(())
    } else {
      Err(GraphError::StorageError(format!(
        "Graph '{}' not found.",
        graph_name
      )))
    }
  }

  pub async fn update_node(&self, graph_name: &str, new_node: Node) -> Result<(), GraphError> {
    let mut graphs = self.graphs.write().await;
    if let Some(graph) = graphs.get_mut(graph_name) {
      graph.update_node(new_node);
      Ok(())
    } else {
      Err(GraphError::StorageError(format!(
        "Graph '{}' not found.",
        graph_name
      )))
    }
  }

  pub async fn update_edge(&self, graph_name: &str, new_edge: Edge) -> Result<(), GraphError> {
    let mut graphs = self.graphs.write().await;
    if let Some(graph) = graphs.get_mut(graph_name) {
      graph.update_edge(new_edge);
      Ok(())
    } else {
      Err(GraphError::StorageError(format!(
        "Graph '{}' not found.",
        graph_name
      )))
    }
  }
}
