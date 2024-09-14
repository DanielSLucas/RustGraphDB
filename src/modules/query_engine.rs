use std::collections::{HashSet, VecDeque};
use crate::Graph;

/// Breadth-First Search (BFS)
pub fn bfs(graph: &Graph, start_node: usize) -> Vec<usize> {
  let mut visited = HashSet::new();
  let mut queue = VecDeque::new();
  let mut order = Vec::new();

  visited.insert(start_node);
  queue.push_back(start_node);

  while let Some(node_id) = queue.pop_front() {
      order.push(node_id);

      if let Some(neighbors) = graph.adjacency_list().get(&node_id) {
          for &neighbor in neighbors {
              if !visited.contains(&neighbor) {
                  visited.insert(neighbor);
                  queue.push_back(neighbor);
              }
          }
      }
  }

  order
}

/// Depth-First Search (DFS)
pub fn dfs(graph: &Graph, start_node: usize) -> Vec<usize> {
  let mut visited = HashSet::new();
  let mut stack = Vec::new();
  let mut order = Vec::new();

  stack.push(start_node);

  while let Some(node_id) = stack.pop() {
      if !visited.contains(&node_id) {
          visited.insert(node_id);
          order.push(node_id);

          if let Some(neighbors) = graph.adjacency_list().get(&node_id) {
              for &neighbor in neighbors.iter().rev() {
                  if !visited.contains(&neighbor) {
                      stack.push(neighbor);
                  }
              }
          }
      }
  }

  order
}