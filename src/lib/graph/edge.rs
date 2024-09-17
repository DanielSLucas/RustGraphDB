use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Edge {
  pub id: usize,
  pub from: usize,
  pub to: usize,
  pub properties: HashMap<String, String>,
}

impl Edge {
  pub fn new(id: usize, from: usize, to: usize, properties: HashMap<String, String>) -> Self {
    Self { id, from, to, properties }
  }

  // CRUD operations for Edge can be added here
}
