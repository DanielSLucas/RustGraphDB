use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
  pub id: usize,
  pub label: String,
  pub from: usize,
  pub to: usize,
  pub properties: HashMap<String, String>,
}

impl Edge {
  pub fn new(
    id: usize,
    label: String,
    from: usize,
    to: usize,
    properties: HashMap<String, String>,
  ) -> Self {
    Self {
      id,
      label,
      from,
      to,
      properties,
    }
  }
}
