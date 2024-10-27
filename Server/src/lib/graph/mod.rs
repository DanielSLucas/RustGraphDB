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
  next_node_id: usize,
  next_edge_id: usize,
}

impl Graph {
  pub fn new(name: String) -> Self {
    Self {
      name,
      nodes: HashMap::new(),
      edges: HashMap::new(),
      next_node_id: 0,
      next_edge_id: 0,
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

  pub fn adjacency_list(&self) -> HashMap<usize, Vec<usize>> {
    let mut adj = HashMap::new();

    for edge in self.edges().values() {
        // Insere o nó `to` no vetor de adjacência do nó `from`
        adj.entry(edge.from)
            .or_insert_with(Vec::new)
            .push(edge.to);
    }

    adj
  }

  pub fn relations_list(&self) -> HashMap<usize, Vec<(usize, String, String, usize, String)>> {
    let mut edges = HashMap::new();

    for edge in self.edges().values() {
        if let (Some(from_node), Some(to_node)) = (self.get_node(edge.from), self.get_node(edge.to)) {
            // Adiciona a relação diretamente em `edges` por id do nó de origem
            edges.entry(from_node.id)
                .or_insert_with(Vec::new)
                .push((
                    from_node.id,
                    from_node.label.clone(),
                    edge.label.clone(),
                    to_node.id,
                    to_node.label.clone(),
                ));
        }
    }

    edges
  }

  // NODES CRUD
  pub fn add_node(&mut self, label: String, properties: HashMap<String, String>) -> Node {
    self.next_node_id += 1;
    let node = Node::new(self.next_node_id, label, properties);
    self.nodes.insert(node.id, node.clone());
    node
  }

  pub fn add_full_node(&mut self, node: Node) -> Node {
    self.next_node_id = node.id + 1;
    self.nodes.insert(node.id, node.clone());
    node
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
  pub fn add_edge(
    &mut self,
    label: String,
    from: usize,
    to: usize,
    properties: HashMap<String, String>,
  ) -> Edge {
    self.next_edge_id += 1;
    let edge = Edge::new(self.next_edge_id, label, from, to, properties);
    self.edges.insert(edge.id, edge.clone());
    edge
  }

  pub fn add_full_edge(&mut self, edge: Edge) -> Edge {
    self.next_edge_id = edge.id + 1;
    self.edges.insert(edge.id, edge.clone());
    edge
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
    self.edges.remove(&edge_id);
  }
}
