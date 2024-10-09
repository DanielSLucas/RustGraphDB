use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::lib::storage::StorageManager;
use crate::lib::graph::{Graph, node::Node, edge::Edge};

pub use super::graph_error::GraphError;

pub type GraphResult<T> = Result<T, GraphError>;

pub struct GraphService {
  storage_manager: Arc<Mutex<StorageManager>>,
}

impl Clone for GraphService {
  fn clone(&self) -> Self {
    Self {
      storage_manager: Arc::clone(&self.storage_manager),
    }
  }
}

impl GraphService {
  pub fn new(storage_manager: Arc<Mutex<StorageManager>>) -> Self {
    Self { storage_manager }
  }

  pub fn create_graph(&self, name: String) -> GraphResult<()> {
    let mut manager = self.storage_manager.lock().unwrap();
    
    if manager.get_graph(&name).is_some() {
      Err(GraphError::GraphAlreadyExists(name))
    } else {
      let graph = Graph::new(name.clone());
      manager.add_graph(graph);
      Ok(())
    }
  }

  pub fn list_graphs(&self) -> GraphResult<Vec<String>> {
    let manager = self.storage_manager.lock().unwrap();
    Ok(manager.get_graph_names())
  }

  pub fn add_node(
    &self,
    graph_name: String,
    node_id: usize,
    label: String,
    properties: HashMap<String, String>,
  ) -> GraphResult<()> {
    let mut manager = self.storage_manager.lock().unwrap();

    if let Some(graph) = manager.get_graph_mut(&graph_name) {
      if graph.get_node(node_id).is_some() {
        Err(GraphError::NodeAlreadyExists(node_id))
      } else {
        let node = Node::new(node_id, label, properties);
        graph.add_node(node);
        Ok(())
      }
    } else {
      Err(GraphError::GraphNotFound(graph_name))
    }
  }

  pub fn add_edge(
    &self,
    graph_name: String,
    edge_id: usize,
    from: usize,
    to: usize,
    label: String,
    properties: HashMap<String, String>,
  ) -> GraphResult<()> {
    let mut manager = self.storage_manager.lock().unwrap();

    if let Some(graph) = manager.get_graph_mut(&graph_name) {
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

      Ok(())
    } else {
      Err(GraphError::GraphNotFound(graph_name))
    }
  }

  pub fn get_graph_adjacency(&self, graph_name: String) -> GraphResult<HashMap<usize, Vec<usize>>> {
    let manager = self.storage_manager.lock().unwrap();

    if let Some(graph) = manager.get_graph(&graph_name) {
      Ok(graph.adjacency_list().clone())
    } else {
      Err(GraphError::GraphNotFound(graph_name))
    }
  }

  pub fn get_graph_relations(&self, graph_name: String) -> GraphResult<Vec<(usize, String, String, usize, String)>> {
    let manager = self.storage_manager.lock().unwrap();
    
    if let Some(graph) = manager.get_graph(&graph_name) {
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
    } else {
      Err(GraphError::GraphNotFound(graph_name))
    }
  }

  pub fn search_path(&self, graph_name: String, method: String, origin: usize, goal: usize) -> GraphResult<Vec<usize>> {
    match method.as_str() {
      "bfs" => self.bfs_path(graph_name, origin, goal),
      "dfs" => self.dfs_path(graph_name, origin, goal),
      "dijkstra" => self.dijkstra_path(graph_name, origin, goal),
      _ => Err(GraphError::GraphNotFound(graph_name)),
    }
  }

  pub fn bfs_path(&self, graph_name: String, origin: usize, goal: usize) -> GraphResult<Vec<usize>> {
    let manager = self.storage_manager.lock().unwrap();

    if let Some(graph) = manager.get_graph(&graph_name) {
      // Implementação do algoritmo BFS para encontrar o caminho do origin ao goal
      let path = graph.bfs_parallel(origin, goal); // Assumindo que você tenha uma função `bfs` no Graph
      Ok(path)
    } else {
      Err(GraphError::GraphNotFound(graph_name))
    }
  }

  // Métodos DFS e Dijkstra devem ser implementados da mesma forma
  pub fn dfs_path(&self, graph_name: String, origin: usize, goal: usize) -> GraphResult<Vec<usize>> {
    let manager = self.storage_manager.lock().unwrap();

    if let Some(graph) = manager.get_graph(&graph_name) {
      // Implementação do algoritmo BFS para encontrar o caminho do origin ao goal
      let path = graph.dfs(origin, goal); // Assumindo que você tenha uma função `bfs` no Graph
      Ok(path)
    } else {
      Err(GraphError::GraphNotFound(graph_name))
    }
  }

  pub fn dijkstra_path(&self, graph_name: String, origin: usize, goal: usize) -> GraphResult<Vec<usize>> {
    // Implementação do Dijkstra
    unimplemented!()
}
}
