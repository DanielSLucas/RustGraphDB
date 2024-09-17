pub mod node;
pub mod edge;
pub mod traversal;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use self::node::Node;
use self::edge::Edge;

#[derive(Debug, Serialize, Deserialize)]
pub struct Graph {
  pub nodes: HashMap<usize, Node>,
  pub edges: HashMap<usize, Edge>,
}

impl Graph {
  pub fn new() -> Self {
    Self {
      nodes: HashMap::new(),
      edges: HashMap::new(),
    }
  }

  // Implement CRUD operations for nodes and edges here
}
