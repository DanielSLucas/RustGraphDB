pub mod edge;
pub mod node;
pub mod traversal;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use self::edge::Edge;
use self::node::Node;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
  name: String,
  nodes: HashMap<usize, Node>,
  edges: HashMap<usize, Edge>,
  adjacency_list: HashMap<usize, Vec<usize>>,
}

impl Graph {
  pub fn new(name: String) -> Self {
    Self {
      name,
      nodes: HashMap::new(),
      edges: HashMap::new(),
      adjacency_list: HashMap::new(),
    }
  }

  // GETTERS
  pub fn name(&self) -> &String {
    &self.name
  }

  pub fn nodes(&self) -> &HashMap<usize, Node> {
    &self.nodes
  }

  pub fn edges(&self) -> &HashMap<usize, Edge> {
    &self.edges
  }

  pub fn adjacency_list(&self) -> &HashMap<usize, Vec<usize>> {
    &self.adjacency_list
  }

  // NODES CRUD
  pub fn add_node(&mut self, node: Node) {
    self.nodes.insert(node.id, node);
  }

  pub fn get_node(&self, id: usize) -> Option<&Node> {
    self.nodes.get(&id)
  }

  pub fn update_node(&mut self, updated_node: Node) {
    if let Some(node) = self.nodes.get_mut(&updated_node.id) {
      node.properties = updated_node.properties;
      node.label = updated_node.label;
    }
  }

  pub fn delete_node(&mut self, id: usize) {
    self.nodes.remove(&id);
  }

  // EDGES CRUD
  pub fn add_edge(&mut self, edge: Edge) {
    self
      .adjacency_list
      .entry(edge.from)
      .or_insert_with(Vec::new)
      .push(edge.to);
    self.edges.insert(edge.id, edge);
  }

  pub fn get_edge(&self, id: usize) -> Option<&Edge> {
    self.edges.get(&id)
  }

  pub fn update_edge(&mut self, updated_edge: Edge) {
    if let Some(edge) = self.edges.get_mut(&updated_edge.id) {
      edge.properties = updated_edge.properties;
      edge.label = updated_edge.label;
    }
  }

  pub fn delete_edge(&mut self, edge_id: usize) {
    if let Some(edge) = self.edges.remove(&edge_id) {
      if let Some(neighbors) = self.adjacency_list.get_mut(&edge.from) {
        neighbors.retain(|&node_id| node_id != edge.to);
      }
    }
  }
}
