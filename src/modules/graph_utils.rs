use std::collections::HashMap;

use crate::modules::graph::Graph;

pub fn print_adjacency_list(graph: &Graph) {
  // Build an adjacency list from the edges
  let mut adjacency_list: HashMap<usize, Vec<usize>> = HashMap::new();

  // Iterate over all edges to populate the adjacency list
  for edge in graph.edges().values() {
    adjacency_list
      .entry(edge.from())
      .or_insert_with(Vec::new)
      .push(edge.to());
  }

  // Print the adjacency list
  for (node_id, connected_nodes) in &adjacency_list {
    let connected_nodes_str: Vec<String> = connected_nodes
      .iter()
      .map(|id| format!("#{}", id))
      .collect();

    println!("Node #{} -> {}", node_id, connected_nodes_str.join(", "));
  }
}

pub fn print_node_relationships(graph: &Graph) {
  for edge in graph.edges().values() {
    let from_id = edge.from();
    let to_id = edge.to();

    let from_node = graph.nodes().get(&from_id).unwrap();
    let to_node = graph.nodes().get(&to_id).unwrap();

    let from_label = from_node.label();
    let to_label = to_node.label();
    let edge_label = edge.label();

    println!(
        "[#{}]{} --[{}]--> [#{}]{}",
        from_id, from_label, edge_label, to_id, to_label
    );
  }
}
