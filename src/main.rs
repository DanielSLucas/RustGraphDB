use std::error::Error;
use std::collections::HashMap;

mod modules;

use modules::graph::Graph;
use modules::query_engine;
use modules::graph_utils::{print_adjacency_list, print_node_relationships};
use modules::persistence::{load_from_file, save_to_file};


fn main() -> Result<(), Box<dyn Error>> {
    let mut graph = Graph::new();

    // Adding nodes
    let id1 = graph.add_node("Daniel".to_string(), HashMap::new());
    let id2 = graph.add_node("Vitor Freire".to_string(), HashMap::new());
    let id3 = graph.add_node("Alice".to_string(), HashMap::new());

    // Adding edges
    graph.add_edge(id1, id2, "friends with".to_string(), HashMap::new())?;
    graph.add_edge(id1, id3, "colleague of".to_string(), HashMap::new())?;

    // Save graph to JSON file
    save_to_file(&graph, "graph.json")?;

    // Load graph from JSON file
    let loaded_graph = load_from_file("graph.json")?;
    // println!("Loaded Graph: {:?}", loaded_graph);

    // Use BFS
    let bfs_result = query_engine::bfs(&loaded_graph, id1);
    println!("BFS from Daniel: {:?}", bfs_result);

    let dfs_result = query_engine::dfs(&loaded_graph, id1);
    println!("DFS from Daniel: {:?}", dfs_result);

    print_adjacency_list(&loaded_graph);
    print_node_relationships(&loaded_graph);
    
    Ok(())
}

