use crate::lib::errors::graph_error::GraphError;
use crate::lib::graph::{edge::Edge, node::Node, Graph};
use crate::lib::storage::StorageManager;
use std::collections::HashMap;
use std::sync::Arc;

// Tamanho de grafo para rodar usando multi-threads
const TAM_MIN_GRPAH: usize = 10;

pub type GraphResult<T> = Result<T, GraphError>;

pub struct GraphService {
  storage_manager: Arc<StorageManager>,
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

    let _ = self.storage_manager.create_graph(name, graph).await;

    Ok(())
  }

  pub async fn list_graphs(&self) -> GraphResult<Vec<String>> {
    Ok(self.storage_manager.list_graph_names().await)
  }

  pub async fn add_node(
    &self,
    graph_name: String,
    node_id: Option<usize>,
    label: String,
    properties: HashMap<String, String>,
  ) -> GraphResult<Node> {
    let mut graph = self.get_graph(&graph_name).await?;

    if node_id.is_none() {
      let node = graph.add_node(label, properties);

      self
        .storage_manager
        .add_node(graph_name, node.clone())
        .await;

      return Ok(node);
    }

    let node_id = node_id.unwrap();

    if graph.get_node(node_id).is_some() {
      return Err(GraphError::NodeAlreadyExists(node_id));
    }

    let node = Node::new(node_id, label, properties);

    self
      .storage_manager
      .add_node(graph_name, node.clone())
      .await;

    Ok(node)
  }

  pub async fn add_edge(
    &self,
    graph_name: String,
    edge_id: Option<usize>,
    from: usize,
    to: usize,
    label: String,
    properties: HashMap<String, String>,
  ) -> GraphResult<Edge> {
    let mut graph = self.get_graph(&graph_name).await?;

    if graph.get_node(from).is_none() {
      return Err(GraphError::NodeNotFound(from));
    }
    if graph.get_node(to).is_none() {
      return Err(GraphError::NodeNotFound(to));
    }

    if edge_id.is_none() {
      let edge = graph.add_edge(label, from, to, properties);

      self
        .storage_manager
        .add_edge(graph_name, edge.clone())
        .await;

      return Ok(edge);
    }

    let edge_id = edge_id.unwrap();

    if graph.get_edge(edge_id).is_some() {
      return Err(GraphError::EdgeAlreadyExists(edge_id));
    }

    let edge = Edge::new(edge_id, label, from, to, properties);

    self
      .storage_manager
      .add_edge(graph_name, edge.clone())
      .await;

    Ok(edge)
  }

  pub async fn get_graph_adjacency(
    &self,
    graph_name: String,
  ) -> GraphResult<HashMap<usize, Vec<usize>>> {
    let graph = self.get_graph(&graph_name).await?;
    Ok(graph.adjacency_list().clone())
  }

  pub async fn get_graph_relations(
    &self,
    graph_name: String,
  ) -> GraphResult<Vec<(usize, String, String, usize, String)>> {
    let graph = self.get_graph(&graph_name).await?;
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
    let graph = self.get_graph(&graph_name).await?;
    let path = graph.bfs(origin, goal, TAM_MIN_GRPAH);
    Ok(path)
  }

  pub async fn dfs_path(
    &self,
    graph_name: String,
    origin: usize,
    goal: usize,
  ) -> GraphResult<Vec<usize>> {
    let graph = self.get_graph(&graph_name).await?;
    let path = graph.dfs(origin, goal, TAM_MIN_GRPAH);
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

  pub async fn get_graph(&self, graph_name: &str) -> GraphResult<Graph> {
    self
      .storage_manager
      .get_graph(graph_name)
      .await
      .ok_or_else(|| GraphError::GraphNotFound(graph_name.to_string()))
  }
}
