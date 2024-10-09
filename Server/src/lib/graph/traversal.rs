use super::Graph;
use std::collections::{VecDeque, HashSet, HashMap};

impl Graph {
  pub fn bfs(&self, start_id: usize, end_id: usize) -> Vec<usize> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut parent_map = HashMap::new(); // Para rastrear o caminho

    queue.push_back(start_id);
    visited.insert(start_id);
    parent_map.insert(start_id, None); // O nó inicial não tem pai

    while let Some(node_id) = queue.pop_front() {
      if node_id == end_id {
        return self.build_path(end_id, &parent_map);
      }

      if let Some(neighbors) = self.adjacency_list.get(&node_id) {
        for &adjacent_id in neighbors {
          if !visited.contains(&adjacent_id) {
            visited.insert(adjacent_id);
            queue.push_back(adjacent_id);
            parent_map.insert(adjacent_id, Some(node_id)); // Rastreia o pai do nó
          }
        }
      }
    }

    Vec::new() // Retorna um vetor vazio se o destino não for encontrado
  }

  fn build_path(&self, end_id: usize, parent_map: &HashMap<usize, Option<usize>>) -> Vec<usize> {
    let mut path = Vec::new();
    let mut current_id = end_id;

    while let Some(&Some(parent_id)) = parent_map.get(&current_id) {
      path.push(current_id);
      current_id = parent_id;
    }

    path.push(current_id); // Adiciona o nó inicial
    path.reverse(); // Inverte o caminho para que ele seja do início ao fim
    path
  }
}
