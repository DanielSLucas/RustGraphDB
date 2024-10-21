use super::Graph;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

// Implementação de busca em largura (BFS) com multithreads
impl Graph {
  // Função auxiliar para construir o caminho a partir do mapa de pais
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

  pub fn bfs(&self, start_id: usize, end_id: usize, threshold: usize) -> Vec<usize> {
    // Verifica o tamanho do grafo para decidir se usa multi-thread ou single-thread
    if self.adjacency_list.len() >= threshold {
      // Modo multi-thread
      self.bfs_multi_thread(start_id, end_id)
    } else {
      // Modo single-thread
      self.bfs_single_thread(start_id, end_id)
    }
  }

  // Função para BFS single-thread
  fn bfs_single_thread(&self, start_id: usize, end_id: usize) -> Vec<usize> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut parent_map = HashMap::new();

    queue.push_back(start_id);
    visited.insert(start_id);
    parent_map.insert(start_id, None);

    while let Some(node_id) = queue.pop_front() {
      if node_id == end_id {
        return self.build_path(end_id, &parent_map);
      }

      if let Some(neighbors) = self.adjacency_list.get(&node_id) {
        for &adjacent_id in neighbors {
          if !visited.contains(&adjacent_id) {
            visited.insert(adjacent_id);
            queue.push_back(adjacent_id);
            parent_map.insert(adjacent_id, Some(node_id));
          }
        }
      }
    }

    Vec::new() // Retorna um vetor vazio se o destino não for encontrado
  }

  // Função para BFS multi-thread
  fn bfs_multi_thread(&self, start_id: usize, end_id: usize) -> Vec<usize> {
    let visited = Arc::new(Mutex::new(HashSet::new()));
    let queue = Arc::new(Mutex::new(VecDeque::new()));
    let parent_map = Arc::new(Mutex::new(HashMap::new()));

    {
      let mut queue = queue.lock().unwrap();
      queue.push_back(start_id);
    }
    {
      let mut visited = visited.lock().unwrap();
      visited.insert(start_id);
    }
    {
      let mut parent_map = parent_map.lock().unwrap();
      parent_map.insert(start_id, None);
    }

    let (tx, rx) = mpsc::channel();
    let num_threads = num_cpus::get(); // Número máximo de threads disponíveis

    for _ in 0..num_threads {
      let visited = Arc::clone(&visited);
      let queue = Arc::clone(&queue);
      let parent_map = Arc::clone(&parent_map);
      let tx = tx.clone();
      let adjacency_list = self.adjacency_list.clone();

      thread::spawn(move || {
        while let Some(node_id) = {
          let mut queue = queue.lock().unwrap();
          queue.pop_front()
        } {
          if node_id == end_id {
            tx.send(Some(node_id)).unwrap();
            return;
          }

          if let Some(neighbors) = adjacency_list.get(&node_id) {
            for &adjacent_id in neighbors {
              let mut visited = visited.lock().unwrap();
              if !visited.contains(&adjacent_id) {
                visited.insert(adjacent_id);
                let mut queue = queue.lock().unwrap();
                queue.push_back(adjacent_id);
                let mut parent_map = parent_map.lock().unwrap();
                parent_map.insert(adjacent_id, Some(node_id));
              }
            }
          }
        }
        tx.send(None).unwrap();
      });
    }

    // Espera o resultado
    while let Ok(result) = rx.recv() {
      if let Some(end_id) = result {
        return self.build_path(end_id, &parent_map.lock().unwrap());
      }
    }

    Vec::new() // Retorna um vetor vazio se o destino não for encontrado
  }

  pub fn dfs(&self, start_id: usize, end_id: usize, threshold: usize) -> Vec<usize> {
    // Verifica o tamanho do grafo para decidir se usa multi-thread ou single-thread
    if self.adjacency_list.len() >= threshold {
      // Modo multi-thread
      self.dfs_parallel(start_id, end_id)
    } else {
      // Modo single-thread
      self.dfs_single_thread(start_id, end_id)
    }
  }

  // Função DFS com multithreading
  fn dfs_parallel(&self, start_id: usize, end_id: usize) -> Vec<usize> {
    let visited = Arc::new(Mutex::new(HashSet::new()));
    let stack = Arc::new(Mutex::new(vec![start_id]));
    let parent_map = Arc::new(Mutex::new(HashMap::new())); // Para rastrear o caminho

    let (tx, rx) = mpsc::channel();

    let thread_count = num_cpus::get(); // Usar o número máximo de threads possíveis
    let mut threads = vec![];

    for _ in 0..thread_count {
      // Clonando referências para passar para as threads
      let visited_clone = Arc::clone(&visited);
      let stack_clone = Arc::clone(&stack);
      let parent_map_clone = Arc::clone(&parent_map);
      let adjacency_list_clone = self.adjacency_list.clone();
      let tx_clone = tx.clone();

      let handle = thread::spawn(move || {
        while let Some(node_id) = {
          let mut stack = stack_clone.lock().unwrap();
          stack.pop()
        } {
          if node_id == end_id {
            tx_clone.send(Some(node_id)).unwrap();
            return;
          }

          if visited_clone.lock().unwrap().insert(node_id) {
            let mut parent_map = parent_map_clone.lock().unwrap();
            parent_map.insert(node_id, None); // O nó atual não tem pai ainda

            if let Some(neighbors) = adjacency_list_clone.get(&node_id) {
              for &adjacent_id in neighbors {
                if !visited_clone.lock().unwrap().contains(&adjacent_id) {
                  let mut stack = stack_clone.lock().unwrap();
                  stack.push(adjacent_id);
                  parent_map.insert(adjacent_id, Some(node_id)); // Rastreia o pai do nó
                }
              }
            }
          }
        }
        tx_clone.send(None).unwrap();
      });

      threads.push(handle);
    }

    // Aguardando as threads terminarem
    for thread in threads {
      thread.join().unwrap();
    }

    // Construindo o caminho final
    if let Some(end_id) = rx.recv().unwrap() {
      self.build_path(end_id, &parent_map.lock().unwrap())
    } else {
      Vec::new() // Retorna um vetor vazio se o destino não for encontrado
    }
  }

  // Função DFS simples (single-thread)
  fn dfs_single_thread(&self, start_id: usize, end_id: usize) -> Vec<usize> {
    let mut visited = HashSet::new();
    let mut stack = vec![start_id];
    let mut parent_map = HashMap::new(); // Para rastrear o caminho

    while let Some(node_id) = stack.pop() {
      if node_id == end_id {
        return self.build_path(end_id, &parent_map);
      }

      if visited.insert(node_id) {
        parent_map.insert(node_id, None); // O nó atual não tem pai ainda

        if let Some(neighbors) = self.adjacency_list.get(&node_id) {
          for &adjacent_id in neighbors {
            if !visited.contains(&adjacent_id) {
              stack.push(adjacent_id);
              parent_map.insert(adjacent_id, Some(node_id)); // Rastreia o pai do nó
            }
          }
        }
      }
    }
    Vec::new() // Retorna um vetor vazio se o destino não for encontrado
  }
}

// Estrutura auxiliar para Dijkstra
#[derive(Eq, PartialEq)]
struct State {
  cost: usize,
  node_id: usize,
}

// Para que a fila de prioridade funcione corretamente
impl Ord for State {
  fn cmp(&self, other: &Self) -> Ordering {
    other.cost.cmp(&self.cost) // Inverte a ordem para a fila de prioridade
  }
}

impl PartialOrd for State {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}
