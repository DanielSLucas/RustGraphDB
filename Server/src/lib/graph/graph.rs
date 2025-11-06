use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::lib::storage::id_generator::IdGenerator;

use super::edge::{CreateEdgeDTO, Edge};
use super::node::{CreateNodeDTO, Node};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
  name: String,
  nodes: HashMap<usize, Arc<RwLock<Node>>>,
  edges: HashMap<usize, Arc<RwLock<Edge>>>,
  id_generator: Arc<IdGenerator>,
}

impl Graph {
  pub fn new(name: String, id_generator: Arc<IdGenerator>) -> Self {
    Self {
      name,
      nodes: HashMap::new(),
      edges: HashMap::new(),
      id_generator,
    }
  }

  // GETTERS
  pub fn name(&self) -> &String {
    &self.name
  }

  pub fn nodes(&self) -> &HashMap<usize, Arc<RwLock<Node>>> {
    &self.nodes
  }

  pub fn edges(&self) -> &HashMap<usize, Arc<RwLock<Edge>>> {
    &self.edges
  }

  pub fn adjacency_list(&self) -> HashMap<usize, Vec<usize>> {
    let mut adj = HashMap::new();

    for arc_edge in self.edges().values() {
      let edge = arc_edge.read().unwrap();
      adj.entry(edge.from).or_insert_with(Vec::new).push(edge.to);
    }

    adj
  }

  pub fn relations_list(&self) -> HashMap<usize, Vec<(usize, String, String, usize, String)>> {
    let mut edges = HashMap::new();

    for arc_edge in self.edges().values() {
      let edge = arc_edge.read().unwrap();
      if let (Some(from_node), Some(to_node)) = (self.get_node(edge.from), self.get_node(edge.to)) {
        // Adiciona a relação diretamente em `edges` por id do nó de origem
        edges.entry(from_node.id).or_insert_with(Vec::new).push((
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
  pub fn add_node(&mut self, data: &CreateNodeDTO) -> Node {
    let node = Node::new(
      self.id_generator.generate_node_id(),
      data.label.clone(),
      data.category.clone(),
      data.properties.clone(),
    );
    self
      .nodes
      .insert(node.id, Arc::new(RwLock::new(node.clone())));
    node
  }

  pub fn add_full_node(&mut self, node: Node) -> Node {
    self
      .nodes
      .insert(node.id, Arc::new(RwLock::new(node.clone())));
    node
  }

  pub fn get_node(&self, id: usize) -> Option<Node> {
    match self.nodes.get(&id) {
      Some(node) => Some(node.read().unwrap().clone()),
      None => None,
    }
  }

  pub fn update_node(&mut self, updated_node: Node) {
    if let Some(arc_node) = self.nodes.get_mut(&updated_node.id) {
      let mut node = arc_node.write().unwrap();

      node.label = updated_node.label;

      for (k, v) in updated_node.properties {
        node.properties.insert(k, v);
      }
    }
  }

  pub fn delete_node(&mut self, id: usize) {
    self.nodes.remove(&id);
  }

  // EDGES CRUD
  pub fn add_edge(&mut self, data: &CreateEdgeDTO) -> Edge {
    let edge = Edge::new(
      self.id_generator.generate_edge_id(),
      data.label.clone(),
      data.from,
      data.to,
      data.properties.clone(),
    );
    self
      .edges
      .insert(edge.id, Arc::new(RwLock::new(edge.clone())));
    edge
  }

  pub fn add_full_edge(&mut self, edge: Edge) -> Edge {
    self
      .edges
      .insert(edge.id, Arc::new(RwLock::new(edge.clone())));
    edge
  }

  pub fn get_edge(&self, id: usize) -> Option<Edge> {
    match self.edges.get(&id) {
      Some(edge) => Some(edge.read().unwrap().clone()),
      None => None,
    }
  }

  pub fn update_edge(&mut self, updated_edge: Edge) {
    if let Some(arc_edge) = self.edges.get_mut(&updated_edge.id) {
      let mut edge = arc_edge.write().unwrap();

      edge.label = updated_edge.label;

      for (k, v) in updated_edge.properties {
        edge.properties.insert(k, v);
      }
    }
  }

  pub fn delete_edge(&mut self, edge_id: usize) {
    self.edges.remove(&edge_id);
  }
}
