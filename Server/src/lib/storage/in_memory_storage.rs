use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use crate::lib::{
  errors::graph_error::GraphError,
  graph::{edge::Edge, node::Node, Graph},
};

use super::id_generator::IdGenerator;

pub struct InMemoryStorage {
  graphs: RwLock<HashMap<String, Graph>>,
  graphs_id_generators: RwLock<HashMap<String, Arc<IdGenerator>>>,
}

impl InMemoryStorage {
  pub fn new() -> Self {
    Self {
      graphs: RwLock::new(HashMap::new()),
      graphs_id_generators: RwLock::new(HashMap::new()),
    }
  }

  pub async fn list_graph_names(&self) -> Vec<String> {
    let graphs = self.graphs.read().await;
    graphs.keys().cloned().collect()
  }

  pub async fn create_graph(&self, graph_name: String) -> Result<Graph, ()> {
    let id_generator = Arc::new(IdGenerator::new());
    let graph = Graph::new(graph_name.clone(), Arc::clone(&id_generator));

    let mut graphs = self.graphs.write().await;
    graphs.insert(graph_name.clone(), graph.clone());

    self
      .graphs_id_generators
      .write()
      .await
      .insert(graph_name, id_generator);

    Ok(graph)
  }

  pub async fn get_graph(&self, graph_name: &str) -> Option<Graph> {
    let graphs = self.graphs.read().await;
    graphs.get(graph_name).cloned()
  }

  pub async fn add_node(&self, graph_name: &str, node: Node) -> Result<(), GraphError> {
    let mut graphs = self.graphs.write().await;
    if let Some(graph) = graphs.get_mut(graph_name) {
      graph.add_full_node(node);
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
      graph.add_full_edge(edge);
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
