use super::Graph;
use std::collections::{VecDeque, HashSet};

impl Graph {
  pub fn bfs(&self, start_id: usize) -> Vec<usize> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut order = Vec::new();

    queue.push_back(start_id);
    visited.insert(start_id);

    while let Some(node_id) = queue.pop_front() {
      order.push(node_id);
      for edge in self.edges.values() {
        if edge.from == node_id && !visited.contains(&edge.to) {
          visited.insert(edge.to);
          queue.push_back(edge.to);
        }
      }
    }
    
    order
  }

  // Implement Dijkstra's algorithm or other traversal methods as needed
}
