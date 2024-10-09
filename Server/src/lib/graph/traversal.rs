use super::Graph;
use std::collections::{VecDeque, HashSet, HashMap, BinaryHeap};
use std::cmp::Ordering;

// Implementação de busca em largura (BFS)
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

    // Implementação de busca em profundidade (DFS)
    pub fn dfs(&self, start_id: usize, end_id: usize) -> Vec<usize> {
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
