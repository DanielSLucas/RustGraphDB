use std::sync::atomic::{AtomicUsize, Ordering};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct IdGenerator {
  next_node_id: AtomicUsize,
  next_edge_id: AtomicUsize,
}

impl IdGenerator {
  pub fn new() -> Self {
    Self {
      next_node_id: AtomicUsize::new(1),
      next_edge_id: AtomicUsize::new(1),
    }
  }

  pub fn from(initial_node_id: usize, initial_edge_id: usize) -> Self {
    Self {
      next_node_id: AtomicUsize::new(initial_node_id),
      next_edge_id: AtomicUsize::new(initial_edge_id),
    }
  }

  pub fn generate_node_id(&self) -> usize {
    self.next_node_id.fetch_add(1, Ordering::SeqCst)
  }

  pub fn generate_edge_id(&self) -> usize {
    self.next_edge_id.fetch_add(1, Ordering::SeqCst)
  }
}
