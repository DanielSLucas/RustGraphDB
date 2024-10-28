use crate::lib::errors::graph_error::GraphError;
use crate::lib::graph::edge::CreateEdgeDTO;
use crate::lib::graph::node::CreateNodeDTO;
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

    let _ = self.storage_manager.create_graph(name).await;

    Ok(())
  }

  pub async fn list_graphs(&self) -> GraphResult<Vec<String>> {
    Ok(self.storage_manager.list_graph_names().await)
  }

  pub async fn add_nodes(
    &self,
    graph_name: String,
    nodes_data: Vec<CreateNodeDTO>,
  ) -> GraphResult<Vec<Node>> {
    let mut graph = self.get_graph(&graph_name).await?;

    let mut created_nodes = vec![Node::new(1, String::new(), HashMap::new()); nodes_data.len()];

    for (i, data) in nodes_data.iter().enumerate() {
      let node = graph.add_node(data);

      self
        .storage_manager
        .add_node(graph_name.clone(), node.clone())
        .await;

      created_nodes[i] = node;
    }

    return Ok(created_nodes);
  }

  pub async fn add_edges(
    &self,
    graph_name: String,
    edges_data: Vec<CreateEdgeDTO>,
  ) -> GraphResult<Vec<Edge>> {
    let mut graph = self.get_graph(&graph_name).await?;

    let mut created_edges =
      vec![Edge::new(1, String::new(), 0, 0, HashMap::new()); edges_data.len()];

    for (i, data) in edges_data.iter().enumerate() {
      if graph.get_node(data.from).is_none() {
        return Err(GraphError::NodeNotFound(data.from));
      }
      if graph.get_node(data.to).is_none() {
        return Err(GraphError::NodeNotFound(data.to));
      }

      let edge = graph.add_edge(data);

      self
        .storage_manager
        .add_edge(graph_name.clone(), edge.clone())
        .await;

      created_edges[i] = edge;
    }

    return Ok(created_edges);
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
    let relations_map = graph.relations_list();

    // Converte o HashMap em um Vec de todas as relações
    let mut relations_vec = Vec::new();
    for relations in relations_map.values() {
      relations_vec.extend(relations.clone());
    }

    Ok(relations_vec)
  }

  pub async fn search_path(
    &self,
    graph_name: String,
    method: String,
    origin: usize,
    goal: usize,
    property_name: String,
  ) -> GraphResult<Vec<usize>> {
    match method.as_str() {
      "bfs" => self.bfs_path(graph_name, origin, goal).await,
      "dfs" => self.dfs_path(graph_name, origin, goal).await,
      "dijkstra" => {
        self
          .dijkstra_path(graph_name, origin, goal, property_name)
          .await
      }
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
    graph_name: String,
    origin: usize,
    goal: usize,
    property_name: String,
  ) -> GraphResult<Vec<usize>> {
    let graph = self.get_graph(&graph_name).await?;

    let path = graph.dijkstra(origin, goal, property_name, TAM_MIN_GRPAH);
    Ok(path)
  }

  pub async fn get_graph(&self, graph_name: &str) -> GraphResult<Graph> {
    self
      .storage_manager
      .get_graph(graph_name)
      .await
      .ok_or_else(|| GraphError::GraphNotFound(graph_name.to_string()))
  }
}
