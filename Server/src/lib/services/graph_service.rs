use crate::lib::graph::{edge::Edge, node::Node, Graph};
use crate::lib::storage::StorageManager;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Tamanho de grafo para rodar usando multi-threads
const TAM_MIN_GRPAH: usize = 10;

pub use super::graph_error::GraphError;

pub type GraphResult<T> = Result<T, GraphError>;

pub struct GraphService {
  storage_manager: Arc<StorageManager>,
}

impl Clone for GraphService {
  fn clone(&self) -> Self {
    Self {
      storage_manager: Arc::clone(&self.storage_manager),
    }
  }
}

impl GraphService {
  pub fn new(storage_manager: Arc<StorageManager>) -> Self {
    Self { storage_manager }
  }

  pub async fn create_graph(&self, name: String) -> GraphResult<()> {
    if self.storage_manager.get_graph(&name).await.is_some() {
      return Err(GraphError::GraphAlreadyExists(name));
    }

    let graph = Graph::new(name.clone());
    self
      .storage_manager
      .add_graph(graph)
      .await
      .map_err(|e| GraphError::StorageError(format!("Failed to add graph '{}': {}", name, e)))?;
    Ok(())
  }

  pub fn list_graphs(&self) -> GraphResult<Vec<String>> {
    Ok(self.storage_manager.get_graph_names())
  }

  pub async fn add_node(
    &self,
    graph_name: String,
    node_id: usize,
    label: String,
    properties: HashMap<String, String>,
  ) -> GraphResult<()> {
    let graph_arc = self.get_graph_locked(&graph_name).await?;
    {
      let mut graph = graph_arc.lock().unwrap();
      if graph.get_node(node_id).is_some() {
        return Err(GraphError::NodeAlreadyExists(node_id));
      }
      let node = Node::new(node_id, label, properties);
      graph.add_node(node);
    }

    self.save_graph_changes(&graph_name).await?;
    Ok(())
  }

  pub async fn add_edge(
    &self,
    graph_name: String,
    edge_id: usize,
    from: usize,
    to: usize,
    label: String,
    properties: HashMap<String, String>,
  ) -> GraphResult<()> {
    let graph_arc = self.get_graph_locked(&graph_name).await?;
    {
      let mut graph = graph_arc.lock().unwrap();

      if graph.get_edge(edge_id).is_some() {
        return Err(GraphError::EdgeAlreadyExists(edge_id));
      }
      if graph.get_node(from).is_none() {
        return Err(GraphError::NodeNotFound(from));
      }
      if graph.get_node(to).is_none() {
        return Err(GraphError::NodeNotFound(to));
      }

      let edge = Edge::new(edge_id, label, from, to, properties);
      graph.add_edge(edge);
    }

    self.save_graph_changes(&graph_name).await?;
    Ok(())
  }

  pub async fn get_graph_adjacency(
    &self,
    graph_name: String,
  ) -> GraphResult<HashMap<usize, Vec<usize>>> {
    let graph_arc = self.get_graph_locked(&graph_name).await?;
    let graph = graph_arc.lock().unwrap();
    Ok(graph.adjacency_list().clone())
  }

  pub async fn get_graph_relations(
    &self,
    graph_name: String,
  ) -> GraphResult<Vec<(usize, String, String, usize, String)>> {
    let graph_arc = self.get_graph_locked(&graph_name).await?;
    let graph = graph_arc.lock().unwrap();
    let mut relations = Vec::new();

    for edge in graph.edges().values() {
      let from_node = graph.get_node(edge.from).unwrap();
      let to_node = graph.get_node(edge.to).unwrap();

      relations.push((
        from_node.id,
        from_node.label.clone(),
        edge.label.clone(),
        to_node.id,
        to_node.label.clone(),
      ));
    }

    Ok(relations)
  }

  pub async fn search_path(
    &self,
    graph_name: String,
    method: String,
    origin: usize,
    goal: usize,
  ) -> GraphResult<Vec<usize>> {
    match method.as_str() {
      "bfs" => self.bfs_path(graph_name, origin, goal).await,
      "dfs" => self.dfs_path(graph_name, origin, goal).await,
      "dijkstra" => self.dijkstra_path(graph_name, origin, goal).await,
      _ => Err(GraphError::MethodNotSupported(method)),
    }
  }

  pub async fn bfs_path(
    &self,
    graph_name: String,
    origin: usize,
    goal: usize,
  ) -> GraphResult<Vec<usize>> {
    let graph_arc = self.get_graph_locked(&graph_name).await?;
    let graph = graph_arc.lock().unwrap();
    let path = graph.bfs(origin, goal, TAM_MIN_GRPAH);
    Ok(path)
  }

  pub async fn dfs_path(
    &self,
    graph_name: String,
    origin: usize,
    goal: usize,
  ) -> GraphResult<Vec<usize>> {
    let graph_arc = self.get_graph_locked(&graph_name).await?;
    let graph = graph_arc.lock().unwrap();
    let path = graph.dfs(origin, goal);
    Ok(path)
  }

  pub async fn dijkstra_path(
    &self,
    _graph_name: String,
    _origin: usize,
    _goal: usize,
  ) -> GraphResult<Vec<usize>> {
    unimplemented!();
  }

  async fn get_graph_locked(&self, graph_name: &str) -> GraphResult<Arc<Mutex<Graph>>> {
    self
      .storage_manager
      .get_graph(graph_name)
      .await
      .ok_or_else(|| GraphError::GraphNotFound(graph_name.to_string()))
  }

  async fn save_graph_changes(&self, graph_name: &str) -> GraphResult<()> {
    self
      .storage_manager
      .save_graph(graph_name)
      .await
      .map_err(|e| {
        GraphError::StorageError(format!("Failed to save graph '{}': {}", graph_name, e))
      })
  }
}
