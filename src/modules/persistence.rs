use std::fs::File;
use std::io::{Write, Read};
use serde_json;

use super::graph::Graph;

pub fn save_to_file(graph: &Graph, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
  // Serialize the graph to a JSON string
  let json_string = serde_json::to_string(graph)?;
  // Write the JSON string to a file
  let mut file = File::create(filename)?;
  file.write_all(json_string.as_bytes())?;
  Ok(())
}

pub fn load_from_file(filename: &str) -> Result<Graph, Box<dyn std::error::Error>> {
  // Read the JSON string from the file
  let mut file = File::open(filename)?;
  let mut json_string = String::new();
  file.read_to_string(&mut json_string)?;
  // Deserialize the JSON string to a Graph
  let graph: Graph = serde_json::from_str(&json_string)?;
  Ok(graph)
}
