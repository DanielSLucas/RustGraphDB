use super::Graph;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
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

  pub fn bfs(&self, start_id: usize, end_id: usize) -> Vec<usize> {
    let visited = Arc::new(Mutex::new(HashSet::new()));
    let queue = Arc::new(Mutex::new(VecDeque::new()));
    let parent_map = Arc::new(Mutex::new(HashMap::new())); // Para rastrear o caminho

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
      parent_map.insert(start_id, None); // O nó inicial não tem pai
    }

    let (tx, rx) = std::sync::mpsc::channel();

    for _ in 0..4 {
      let visited = Arc::clone(&visited);
      let queue = Arc::clone(&queue);
      let parent_map = Arc::clone(&parent_map);
      let tx = tx.clone();
      let adjacency_list = self.adjacency_list.clone();

      thread::spawn(move || {
        /*let thread_id = thread::current().id();
        println!("Número da thread: {:?}", thread_id);*/
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
                parent_map.insert(adjacent_id, Some(node_id)); // Rastreia o pai do nó
              }
            }
          }
        }
        tx.send(None).unwrap();
      });
    }

    while let Ok(result) = rx.recv() {
      if let Some(end_id) = result {
        return self.build_path(end_id, &parent_map.lock().unwrap());
      }
    }

    Vec::new() // Retorna um vetor vazio se o destino não for encontrado
  }

  // Implementação de busca em profundidade (DFS) com multithreads
  pub fn dfs(&self, start_id: usize, end_id: usize) -> Vec<usize> {
    let visited = Arc::new(Mutex::new(HashSet::new()));
    let stack = Arc::new(Mutex::new(vec![start_id]));
    let parent_map = Arc::new(Mutex::new(HashMap::new())); // Para rastrear o caminho

    let (tx, rx) = std::sync::mpsc::channel();

    // Clonando referências para passar para as threads
    let visited_clone = Arc::clone(&visited);
    let stack_clone = Arc::clone(&stack);
    let parent_map_clone = Arc::clone(&parent_map);
    let adjacency_list_clone = self.adjacency_list.clone();

    // Criando a thread principal
    let main_thread = thread::spawn(move || {
      while let Some(node_id) = {
        let mut stack = stack_clone.lock().unwrap();
        stack.pop()
      } {
        if node_id == end_id {
          tx.send(Some(node_id)).unwrap();
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
      tx.send(None).unwrap();
    });

    // Aguardando a thread principal terminar
    main_thread.join().unwrap();

    // Construindo o caminho final
    if let Some(end_id) = rx.recv().unwrap() {
      self.build_path(end_id, &parent_map.lock().unwrap())
    } else {
      Vec::new() // Retorna um vetor vazio se o destino não for encontrado
    }
  }

  // Implementação da busca de Dijkstra
  /*pub fn dijkstra(&self, start_id: usize, end_id: usize) -> Vec<usize> {
      let mut dist = HashMap::new();
      let mut parent_map = HashMap::new();
      let mut heap = BinaryHeap::new();

      dist.insert(start_id, 0);
      heap.push(State { cost: 0, node_id: start_id });

      while let Some(State { cost, node_id }) = heap.pop() {
          // Se já temos uma distância menor, ignoramos
          if cost > *dist.get(&node_id).unwrap_or(&usize::MAX) {
              continue;
          }

          if node_id == end_id {
              return self.build_path(end_id, &parent_map);
          }

          if let Some(neighbors) = self.adjacency_list.get(&node_id) {
              for &(neighbor_id, weight) in neighbors {
                  let next_cost = cost + weight;

                  if next_cost < *dist.get(&neighbor_id).unwrap_or(&usize::MAX) {
                      dist.insert(neighbor_id, next_cost);
                      parent_map.insert(neighbor_id, Some(node_id));
                      heap.push(State { cost: next_cost, node_id: neighbor_id });
                  }
              }
          }
      }

      Vec::new() // Retorna um vetor vazio se o destino não for encontrado
  }*/
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
