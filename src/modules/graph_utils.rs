use crate::modules::graph::Graph;

pub fn print_adjacency_list(graph: &Graph) {
  for (node_id, edges) in graph.edges() {
    let connected_nodes: Vec<String> = edges.iter()
      .map(|edge| format!("#{}", graph.nodes().get(&edge.to()).unwrap().id().to_string()))
      .collect();

    println!("Node #{} -> {}", node_id, connected_nodes.join(", "));
  }
}

pub fn print_node_relationships(graph: &Graph) {
  for (node_id, edges) in graph.edges() {
    let node_label = graph.nodes().get(node_id).unwrap().label();
    
    for edge in edges {
      let to_id = edge.to();
      let to_label = graph.nodes().get(&edge.to()).unwrap().label();
      let edge_label = edge.label();

      println!("[#{node_id}]{node_label} --[{edge_label}]--> [#{to_id}]{to_label}");
    }
  }
}