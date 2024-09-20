use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::lib::storage::StorageManager;
use crate::lib::graph::{Graph, node::Node, edge::Edge};
use crate::lib::utils::logger::{log_info, log_error};

#[derive(Debug)]
enum Command {
  Exit,
  CreateGraph { name: String },
  ListGraphs,
  AddNode {
    graph_name: String,
    node_id: usize,
    label: String,
    properties: HashMap<String, String>,
  },
  AddEdge {
    graph_name: String,
    edge_id: usize,
    from: usize,
    to: usize,
    label: String,
    properties: HashMap<String, String>,
  },
  PrintGraphAdjacency { graph_name: String },
  PrintGraphRelations { graph_name: String },
  Unknown,
}

pub fn run_cli(storage_manager: Arc<Mutex<StorageManager>>) {
  loop {
    print!("> ");
    io::stdout().flush().unwrap();

    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();

    let command = parse_command(&input_line);

    match command {
      Command::Exit => {
        log_info("Exiting...");
        let storage_manager = storage_manager.lock().unwrap();
        storage_manager.save_all_graphs();
        break;
      }
      Command::CreateGraph { name } => {
        handle_create_graph(&storage_manager, name);
      }
      Command::ListGraphs => {
        handle_list_graphs(&storage_manager);
      }
      Command::AddNode {
        graph_name,
        node_id,
        label,
        properties,
      } => {
        handle_add_node(&storage_manager, graph_name, node_id, label, properties);
      }
      Command::AddEdge {
        graph_name,
        edge_id,
        from,
        to,
        label,
        properties,
      } => {
        handle_add_edge(&storage_manager, graph_name, edge_id, from, to, label, properties);
      }
      Command::PrintGraphAdjacency { graph_name } => {
        handle_print_graph_adjacency(&storage_manager, graph_name);
      }
      Command::PrintGraphRelations { graph_name } => {
        handle_print_graph_relations(&storage_manager, graph_name);
      }
      Command::Unknown => {
        log_error("Unknown command or incorrect arguments.");
      }
    }
  }
}

fn split_command_line(input: &str) -> Vec<String> {
  let mut args = Vec::new();
  let mut chars = input.chars().peekable();

  while let Some(&c) = chars.peek() {
    if c.is_whitespace() {
      chars.next();
      continue;
    } else if c == '"' || c == '\'' {
      // Start of a quoted string
      let quote_char = c;
      
      chars.next(); // Consume the quote
      
      let mut arg = String::new();
      while let Some(&c) = chars.peek() {
        if c == quote_char {
          chars.next(); // Consume the closing quote
          break;
        } else {
          arg.push(c);
          chars.next();
        }
      }

      args.push(arg);
    } else {
      // Unquoted word
      let mut arg = String::new();
      while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
          break;
        } else {
          arg.push(c);
          chars.next();
        }
      }

      args.push(arg);
    }
  }
  args
}

fn parse_properties(args: &[String]) -> HashMap<String, String> {
  let mut properties = HashMap::new();
  for arg in args {
    if let Some(idx) = arg.find('=') {
      let key = arg[..idx].to_string();
      let value = arg[idx + 1..].to_string();
      properties.insert(key, value);
    }
  }
  properties
}

fn parse_command(input: &str) -> Command {
  let args = split_command_line(input);
  if args.is_empty() {
    return Command::Unknown;
  }
  match args[0].as_str() {
    "exit" => Command::Exit,
    "create" if args.get(1).map(|s| s.as_str()) == Some("graph") && args.len() == 3 => {
      Command::CreateGraph { name: args[2].clone() }
    }
    "list" if args.get(1).map(|s| s.as_str()) == Some("graphs") && args.len() == 2 => {
      Command::ListGraphs
    }
    "add" if args.get(1).map(|s| s.as_str()) == Some("node") && args.len() >= 5 => {
      let graph_name = args[2].clone();
      if let Ok(node_id) = args[3].parse() {
        let label = args[4].clone();
        let properties = parse_properties(&args[5..]);
        Command::AddNode {
          graph_name,
          node_id,
          label,
          properties,
        }
      } else {
        Command::Unknown
      }
    }
    "add" if args.get(1).map(|s| s.as_str()) == Some("edge") && args.len() >= 7 => {
      let graph_name = args[2].clone();
      if let (Ok(edge_id), Ok(from_id), Ok(to_id)) = (
        args[3].parse(),
        args[4].parse(),
        args[5].parse(),
      ) {
        let label = args[6].clone();
        let properties = parse_properties(&args[7..]);
        Command::AddEdge {
          graph_name,
          edge_id,
          from: from_id,
          to: to_id,
          label,
          properties,
        }
      } else {
        Command::Unknown
      }
    }
    "print" if args.get(1).map(|s| s.as_str()) == Some("graph") && args.len() == 4 => {
      match args[2].as_str() {
        "adjacency" => Command::PrintGraphAdjacency {
          graph_name: args[3].clone(),
        },
        "relations" => Command::PrintGraphRelations {
          graph_name: args[3].clone(),
        },
        _ => Command::Unknown,
      }
    }
    _ => Command::Unknown,
  }
}

// Handler functions

fn handle_create_graph(storage_manager: &Arc<Mutex<StorageManager>>, name: String) {
  let mut manager = storage_manager.lock().unwrap();
  if manager.get_graph(&name).is_some() {
    log_error(&format!("Graph '{}' already exists.", name));
  } else {
    let graph = Graph::new(name.clone());
    manager.add_graph(graph);
    log_info(&format!("Graph '{}' created.", name));
  }
}

fn handle_list_graphs(storage_manager: &Arc<Mutex<StorageManager>>) {
  let manager = storage_manager.lock().unwrap();
  let graph_names = manager.get_graph_names();
  log_info("Graphs:");
  for name in graph_names {
    log_info(&format!("- {}", name));
  }
}

fn handle_add_node(
  storage_manager: &Arc<Mutex<StorageManager>>,
  graph_name: String,
  node_id: usize,
  label: String,
  properties: HashMap<String, String>,
) {
  let mut manager = storage_manager.lock().unwrap();
  if let Some(graph) = manager.get_graph_mut(&graph_name) {
    if graph.get_node(node_id).is_some() {
      log_error(&format!(
        "Node with ID {} already exists in graph '{}'.",
        node_id, graph_name
      ));
    } else {
      let node = Node::new(node_id, label, properties);
      graph.add_node(node);
      log_info(&format!("Node {} added to graph '{}'.", node_id, graph_name));
    }
  } else {
    log_error(&format!("Graph '{}' not found.", graph_name));
  }
}

fn handle_add_edge(
  storage_manager: &Arc<Mutex<StorageManager>>,
  graph_name: String,
  edge_id: usize,
  from: usize,
  to: usize,
  label: String,
  properties: HashMap<String, String>,
) {
  let mut manager = storage_manager.lock().unwrap();
  if let Some(graph) = manager.get_graph_mut(&graph_name) {
    if graph.get_edge(edge_id).is_some() {
      log_error(&format!(
        "Edge with ID {} already exists in graph '{}'.",
        edge_id, graph_name
      ));
    } else {
      if graph.get_node(from).is_none() {
        log_error(&format!(
          "Node with ID {} does not exist in graph '{}'.",
          from, graph_name
        ));
        return;
      }
      if graph.get_node(to).is_none() {
        log_error(&format!(
          "Node with ID {} does not exist in graph '{}'.",
          to, graph_name
        ));
        return;
      }
      let edge = Edge::new(edge_id, label, from, to, properties);
      graph.add_edge(edge);
      log_info(&format!("Edge {} added to graph '{}'.", edge_id, graph_name));
    }
  } else {
    log_error(&format!("Graph '{}' not found.", graph_name));
  }
}

fn handle_print_graph_adjacency(
  storage_manager: &Arc<Mutex<StorageManager>>,
  graph_name: String,
) {
  let manager = storage_manager.lock().unwrap();
  if let Some(graph) = manager.get_graph(&graph_name) {
    log_info(&format!("Adjacency List for graph '{}':", graph_name));
    for (node_id, neighbors) in graph.adjacency_list() {
      let neighbor_str: Vec<String> = neighbors.iter().map(|id| id.to_string()).collect();
      log_info(&format!("Node {}: [{}]", node_id, neighbor_str.join(", ")));
    }
  } else {
    log_error(&format!("Graph '{}' not found.", graph_name));
  }
}

fn handle_print_graph_relations(
  storage_manager: &Arc<Mutex<StorageManager>>,
  graph_name: String,
) {
  let manager = storage_manager.lock().unwrap();
  
  if let Some(graph) = manager.get_graph(&graph_name) {
    log_info(&format!("Graph '{}' relations:", graph_name));
    
    for edge in graph.edges().values() {
      let from_node = graph.get_node(edge.from).unwrap();
      let to_node = graph.get_node(edge.to).unwrap();
      
      log_info(&format!(
        "[#{}]{} --[{}]-> [#{}]{}",
        from_node.id,
        from_node.label,
        edge.label,
        to_node.id,
        to_node.label
      ));
    }
  } else {
    log_error(&format!("Graph '{}' not found.", graph_name));
  }
}
