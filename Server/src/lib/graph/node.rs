use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
  pub id: usize,
  pub label: String,
  pub properties: HashMap<String, String>,
}

impl Node {
  pub fn new(id: usize, label: String, properties: HashMap<String, String>) -> Self {
    Self {
      id,
      label,
      properties,
    }
  }

  // CRUD operations for Node can be added here
}
