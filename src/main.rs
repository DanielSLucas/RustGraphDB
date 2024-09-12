mod modules;

use modules::graph::Graph;
use modules::graph_utils::{print_adjacency_list, print_node_relationships};
use std::collections::HashMap;

fn main() {
    let mut graph: Graph = Graph::new();

    graph.add_node(1, "Daniel".to_string(), HashMap::new());
    graph.add_node(2, "Vitor Freire".to_string(), HashMap::new());
    graph.add_node(3, "Alice".to_string(), HashMap::new());

    graph.add_edge(1, 2, "friends with".to_string(), HashMap::from([("created_at".to_string(), "2023-10-03".to_string())]));
    graph.add_edge(1, 3, "colleague of".to_string(), HashMap::new());
    graph.add_edge(2, 3, "neighbor of".to_string(), HashMap::new());

    println!("Adjacency List:");
    print_adjacency_list(&graph);

    println!("\nNode Relationships:");
    print_node_relationships(&graph);
}
