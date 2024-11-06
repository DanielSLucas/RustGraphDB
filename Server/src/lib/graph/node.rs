use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
  pub id: usize,
  pub label: String,
  pub category: String,
  pub properties: HashMap<String, String>,
}

impl Node {
  pub fn new(
    id: usize,
    label: String,
    category: String,
    properties: HashMap<String, String>,
  ) -> Self {
    Self {
      id,
      label,
      category,
      properties,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNodeDTO {
  pub label: String,
  pub category: String,
  pub properties: HashMap<String, String>,
}

impl CreateNodeDTO {
  pub fn new(label: String, category: String, properties: HashMap<String, String>) -> Self {
    Self {
      label,
      category,
      properties,
    }
  }
}
