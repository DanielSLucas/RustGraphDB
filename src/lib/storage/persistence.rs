use crate::lib::graph::Graph;
use std::fs::File;
use std::io::{Write, Read};
use serde_json;

impl Graph {
  pub fn save_to_file(&self, filename: &str) -> std::io::Result<()> {
    let data = serde_json::to_string(self).unwrap();
    let mut file = File::create(filename)?;
    file.write_all(data.as_bytes())?;
    Ok(())
  }

  pub fn load_from_file(filename: &str) -> std::io::Result<Graph> {
    let mut file = File::open(filename)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    let graph = serde_json::from_str(&data).unwrap();
    Ok(graph)
  }
}
