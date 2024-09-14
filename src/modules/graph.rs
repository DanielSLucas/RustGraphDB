use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Node {
  id: usize,
  label: String,
  properties: HashMap<String, String>,
}

impl Node {
  pub fn id(&self) -> usize {
    self.id
  }

  pub fn label(&self) -> String {
    self.label.clone()
  }

  pub fn properties(&self) -> &HashMap<String, String> {
    &self.properties
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Edge {
  id: usize,
  from: usize,
  to: usize,
  label: String,
  properties: HashMap<String, String>,
}

impl Edge {
  pub fn id(&self) -> usize {
    self.id
  }

  pub fn from(&self) -> usize {
    self.from
  }

  pub fn to(&self) -> usize {
    self.to
  }

  pub fn label(&self) -> String {
    self.label.clone()
  }

  pub fn properties(&self) -> &HashMap<String, String> {
    &self.properties
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Graph {
  nodes: HashMap<usize, Node>,
  edges: HashMap<usize, Edge>,
  adjacency_list: HashMap<usize, Vec<usize>>,
  next_node_id: usize,
  next_edge_id: usize,
}

impl Graph {
  pub fn new() -> Self {
    Graph {
      nodes: HashMap::new(),
      edges: HashMap::new(),
      adjacency_list: HashMap::new(),
      next_node_id: 0,
      next_edge_id: 0,
    }
  }

  pub fn nodes(&self) -> &HashMap<usize, Node> {
    &self.nodes
  }

  pub fn edges(&self) -> &HashMap<usize, Edge> {
    &self.edges
  }

  pub fn next_node_id(&self) -> usize {
    self.next_node_id
  }

  pub fn next_edge_id(&self) -> usize {
    self.next_edge_id
  }

  pub fn adjacency_list(&self) -> &HashMap<usize, Vec<usize>> {
    &self.adjacency_list
  }

  pub fn add_node(&mut self, label: String, properties: HashMap<String, String>) -> usize {
    self.next_node_id += 1;
    let node_id = self.next_node_id;

    let node = Node {
      id: node_id,
      label,
      properties,
    };

    self.nodes.insert(node_id, node);
    node_id
  }

  pub fn add_edge(&mut self, from: usize, to: usize, label: String, properties: HashMap<String, String>) -> Result<usize, String> {
    if !self.nodes.contains_key(&from) || !self.nodes.contains_key(&to) {
      return Err("Source or target node does not exist".to_string());
    }

    self.next_edge_id += 1;
    let edge_id = self.next_edge_id;

    let edge = Edge {
      id: edge_id,
      from,
      to,
      label,
      properties,
    };

    self.edges.insert(edge_id, edge);

    self.adjacency_list
      .entry(from)
      .or_insert_with(Vec::new)
      .push(to);

    Ok(edge_id)
  }
}