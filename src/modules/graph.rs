use std::collections::HashMap;

pub struct Node {
  id: u64,
  label: String,
  properties: HashMap<String, String>,
}

impl Node {
  pub fn id(&self) -> u64 {
    self.id
  }

  pub fn label(&self) -> String {
    self.label.clone()
  }

  pub fn properties(&self) -> &HashMap<String, String> {
    &self.properties
  }
}

pub struct Edge {
  from: u64,
  to: u64,
  label: String,
  properties: HashMap<String, String>,
}

impl Edge {
  pub fn from(&self) -> u64 {
    self.from
  }

  pub fn to(&self) -> u64 {
    self.to
  }

  pub fn label(&self) -> String {
    self.label.clone()
  }

  pub fn properties(&self) -> &HashMap<String, String> {
    &self.properties
  }
}

pub struct Graph {
  nodes: HashMap<u64, Node>,
  edges: HashMap<u64, Vec<Edge>>,
}

impl Graph {
  pub fn new() -> Self {
    Graph {
      nodes: HashMap::new(),
      edges: HashMap::new()
    }
  }

  pub fn nodes(&self) -> &HashMap<u64, Node> {
    &self.nodes
  }

  pub fn edges(&self) -> &HashMap<u64, Vec<Edge>> {
    &self.edges
  }

  pub fn add_node(&mut self, id: u64, label: String, properties: HashMap<String, String>) {
    self.nodes.insert(id, Node { id, label, properties });
  }

  pub fn add_edge(&mut self, from: u64, to: u64, label: String, properties: HashMap<String, String>) {
    let edge = Edge { from, to, label, properties };
    self.edges.entry(from).or_insert(Vec::new()).push(edge);
  }
}