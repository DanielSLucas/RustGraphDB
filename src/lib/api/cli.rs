use std::io::{self, Write};
use crate::lib::{graph::Graph, storage::StorageManager};
use std::sync::{Arc, Mutex};

pub fn run_cli(storage_manager: Arc<Mutex<StorageManager>>) {
  loop {
    print!("> ");
    io::stdout().flush().unwrap();

    let mut command = String::new();
    io::stdin().read_line(&mut command).unwrap();

    let command = command.trim();

    if command == "exit" {
      println!("Exiting...");

      let storage_manager = storage_manager.lock().unwrap();
      storage_manager.save_all_graphs();

      break;
    } else if command.starts_with("create graph") {
      let parts: Vec<&str> = command.split_whitespace().collect();
      
      if parts.len() == 3 {
        let graph_name = parts[2].to_string();
        let mut storage_manager = storage_manager.lock().unwrap();
        let graph = Graph::new(graph_name.clone());
        storage_manager.add_graph(graph);
        println!("Graph '{}' created.", graph_name);
      } else {
        println!("Usage: create graph <graph_name>");
      }

    } else if command == "list graphs" {
      let storage_manager = storage_manager.lock().unwrap();
      let graphs = storage_manager.list_graphs();
      
      println!("Graphs in the database:");
      
      for name in graphs {
        println!("- {}", name);
      }
    } else {
      println!("Unknown command.");
    }
  }
}
