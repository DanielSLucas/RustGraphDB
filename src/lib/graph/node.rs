use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
  pub id: usize,
  pub labels: Vec<String>,
  pub properties: HashMap<String, String>,
}

impl Node {
  pub fn new(id: usize, labels: Vec<String>, properties: HashMap<String, String>) -> Self {
    Self { id, labels, properties }
  }

  // CRUD operations for Node can be added here
}
