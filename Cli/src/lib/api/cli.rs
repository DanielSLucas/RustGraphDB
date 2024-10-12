use crate::lib::utils::logger::{log_error, log_info};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::{self, Write};

#[derive(Debug)]
enum Command {
  Exit,
  CreateGraph {
    name: String,
  },
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
  PrintGraphAdjacency {
    graph_name: String,
  },
  PrintGraphRelations {
    graph_name: String,
  },
  Unknown,
}

#[derive(Deserialize, Debug)]
struct GraphRelation {
  from_node_id: usize,
  from_node_label: String,
  edge_label: String,
  to_node_id: usize,
  to_node_label: String,
}

pub fn run_cli() {
  let client = Client::new(); // Criação do cliente HTTP

  loop {
    print!("> ");
    io::stdout().flush().unwrap();

    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();

    let command = parse_command(&input_line);

    match command {
      Command::Exit => {
        log_info("Exiting...");
        break;
      }
      Command::CreateGraph { name } => handle_create_graph(&client, &name),
      Command::ListGraphs => handle_list_graphs(&client),
      Command::AddNode {
        graph_name,
        node_id,
        label,
        properties,
      } => {
        handle_add_node(&client, graph_name, node_id, label, properties);
      }
      Command::AddEdge {
        graph_name,
        edge_id,
        from,
        to,
        label,
        properties,
      } => {
        handle_add_edge(&client, graph_name, edge_id, from, to, label, properties);
      }
      Command::PrintGraphAdjacency { graph_name } => handle_print_adjacency(&client, graph_name),
      Command::PrintGraphRelations { graph_name } => handle_print_relations(&client, graph_name),
      Command::Unknown => {
        log_error("Unknown command or incorrect arguments.");
      }
    }
  }
}

fn handle_create_graph(client: &Client, name: &str) {
  let response = client
    .post("http://localhost:8080/graphs")
    .json(&serde_json::json!({ "name": name }))
    .send();

  match response {
    Ok(resp) if resp.status().is_success() => {
      log_info(&format!("Graph '{}' created.", name));
    }
    Ok(resp) => {
      log_error(&format!("Failed to create graph: {:?}", resp.status()));
    }
    Err(e) => {
      log_error(&format!("Error: {:?}", e));
    }
  }
}

fn handle_list_graphs(client: &Client) {
  let response = client.get("http://localhost:8080/graphs").send();

  match response {
    Ok(resp) if resp.status().is_success() => {
      if let Ok(graphs) = resp.json::<Vec<String>>() {
        log_info("Graphs:");
        for name in graphs {
          log_info(&format!("- {}", name));
        }
      }
    }
    Ok(resp) => {
      log_error(&format!("Failed to list graphs: {:?}", resp.status()));
    }
    Err(e) => {
      log_error(&format!("Error: {:?}", e));
    }
  }
}

fn handle_add_node(
  client: &Client,
  graph_name: String,
  node_id: usize,
  label: String,
  properties: HashMap<String, String>,
) {
  let response = client
    .post(&format!(
      "http://localhost:8080/graphs/{}/nodes",
      graph_name
    ))
    .json(&serde_json::json!({
        "node_id": node_id,
        "label": label,
        "properties": properties
    }))
    .send();

  match response {
    Ok(resp) if resp.status().is_success() => {
      log_info(&format!(
        "Node {} added to graph '{}'.",
        node_id, graph_name
      ));
    }
    Ok(resp) => {
      log_error(&format!("Failed to add node: {:?}", resp.status()));
    }
    Err(e) => {
      log_error(&format!("Error: {:?}", e));
    }
  }
}

fn handle_add_edge(
  client: &Client,
  graph_name: String,
  edge_id: usize,
  from: usize,
  to: usize,
  label: String,
  properties: HashMap<String, String>,
) {
  let response = client
    .post(&format!(
      "http://localhost:8080/graphs/{}/edges",
      graph_name
    ))
    .json(&serde_json::json!({
        "edge_id": edge_id,
        "from": from,
        "to": to,
        "label": label,
        "properties": properties
    }))
    .send();

  match response {
    Ok(resp) if resp.status().is_success() => {
      log_info(&format!(
        "Edge {} added to graph '{}'.",
        edge_id, graph_name
      ));
    }
    Ok(resp) => {
      log_error(&format!("Failed to add edge: {:?}", resp.status()));
    }
    Err(e) => {
      log_error(&format!("Error: {:?}", e));
    }
  }
}

fn handle_print_adjacency(client: &Client, graph_name: String) {
  // Faz uma requisição GET para obter a lista de adjacência
  let response = client
    .get(&format!(
      "http://localhost:8080/graphs/{}/adjacency",
      graph_name
    ))
    .send();

  match response {
    Ok(resp) if resp.status().is_success() => {
      // Lê o corpo da resposta como texto
      if let Ok(body) = resp.text() {
        println!("Response body: {}", body); // Imprime o corpo da resposta

      // Tenta deserializar o corpo da resposta em um objeto GraphAdjacency
      /*if let Ok(adjacency) = serde_json::from_str::<GraphAdjacency>(&body) {
          log_info(&format!("Graph '{}' adjacency:", graph_name));
          // Imprime a lista de adjacência
          for (node_id, neighbors) in adjacency.adjacency_list {
              println!("[#{}] -> {:?}", node_id, neighbors);
          }
      } else {
          log_error("Failed to parse the graph adjacency.");
      }*/
      } else {
        log_error("Failed to read the response body as text.");
      }
    }
    Ok(resp) => {
      log_error(&format!(
        "Failed to get graph adjacency: {:?}",
        resp.status()
      ));
    }
    Err(e) => {
      log_error(&format!("Error: {:?}", e));
    }
  }
}

fn handle_print_relations(client: &Client, graph_name: String) {
  let response = client
    .get(&format!(
      "http://localhost:8080/graphs/{}/relations",
      graph_name
    ))
    .send();

  match response {
    Ok(resp) if resp.status().is_success() => {
      if let Ok(body) = resp.text() {
        if let Ok(relations) = serde_json::from_str::<Vec<GraphRelation>>(&body) {
          log_info(&format!("Graph '{}' relations:", graph_name));
          for relation in relations {
            println!(
              "[#{}]{} --[{}]-> [#{}]{}",
              relation.from_node_id,
              relation.from_node_label,
              relation.edge_label,
              relation.to_node_id,
              relation.to_node_label
            );
          }
        } else {
          log_error("Failed to parse the graph relations.");
        }
      }
    }
    Ok(resp) => {
      log_error(&format!(
        "Failed to get graph relations: {:?}",
        resp.status()
      ));
    }
    Err(e) => {
      log_error(&format!("Error: {:?}", e));
    }
  }
}

fn split_command_line(input: &str) -> Vec<String> {
  let mut args = Vec::new();
  let mut chars = input.chars().peekable();

  while let Some(&c) = chars.peek() {
    if c.is_whitespace() {
      chars.next();
    } else if c == '"' || c == '\'' {
      let quote_char = c;
      chars.next();
      let mut arg = String::new();
      while let Some(&c) = chars.peek() {
        if c == quote_char {
          chars.next();
          break;
        } else {
          arg.push(c);
          chars.next();
        }
      }
      args.push(arg);
    } else {
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
      Command::CreateGraph {
        name: args[2].clone(),
      }
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
      if let (Ok(edge_id), Ok(from_id), Ok(to_id)) =
        (args[3].parse(), args[4].parse(), args[5].parse())
      {
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
