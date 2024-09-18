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

      if let Some(neighbors) = self.adjacency_list.get(&node_id) {
        for &adjacent_id in neighbors {
          if !visited.contains(&adjacent_id) {
            visited.insert(adjacent_id);
            queue.push_back(adjacent_id);
          }
        }
      }
    }

    order
  }

  // Implement Dijkstra's algorithm or other traversal methods as needed
}
