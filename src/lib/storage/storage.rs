use crate::lib::graph::Graph;
use crate::lib::utils::logger::{log_error, log_info};
use serde_json;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Write, Read};
use std::path::{Path, PathBuf};
use chrono::Utc;

pub struct StorageManager {
  graphs: HashMap<String, Graph>,
  storage_dir: PathBuf,
}

impl StorageManager {
  pub fn new(storage_dir: &str) -> Self {
    let storage_path = PathBuf::from(storage_dir);
    if !storage_path.exists() {
      fs::create_dir_all(&storage_path).expect("Failed to create storage directory");
    }

    let mut manager = Self {
      graphs: HashMap::new(),
      storage_dir: storage_path,
    };

    manager.load_graphs();
    manager
  }

  pub fn graphs(&self) -> &HashMap<String, Graph> {
    &self.graphs
  }

  fn get_graph_files(&self) -> Vec<PathBuf> {
    fs::read_dir(&self.storage_dir)
      .expect("Failed to read storage directory")
      .filter_map(|entry| {
        let entry = entry.ok()?;
        let path = entry.path();
        if path.is_file() {
          Some(path)
        } else {
          None
        }
      })
      .collect()
  }

  pub fn load_graphs(&mut self) {
    let graph_files = self.get_graph_files();

    for file_path in graph_files {
      if let Ok(graph) = self.load_graph_from_file(&file_path) {
        self.graphs.insert(graph.name().clone(), graph);
      }
    }

    if self.graphs.is_empty() {
      log_info("No existing graphs found. Starting with an empty database.");
    } else {
      log_info(&format!("Loaded {} graphs from storage.", self.graphs.len()));
    }
  }

  fn load_graph_from_file(&self, file_path: &Path) -> std::io::Result<Graph> {
    let mut file = File::open(file_path)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    let graph: Graph = serde_json::from_str(&data).expect("Failed to deserialize graph");
    Ok(graph)
  }

  pub fn save_all_graphs(&self) {
    for (_, graph) in &self.graphs {
      if let Err(e) = self.save_graph(graph) {
        log_error(&format!("Failed to save graph '{}': {}", graph.name(), e));
      }
    }
  }

  fn save_graph(&self, graph: &Graph) -> std::io::Result<()> {
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let filename = format!("{}_{}.json", graph.name(), timestamp);
    let filepath = self.storage_dir.join(filename);

    let data = serde_json::to_string_pretty(graph).expect("Failed to serialize graph");
    let mut file = File::create(filepath)?;
    file.write_all(data.as_bytes())?;
    Ok(())
  }

  // Additional methods to manage graphs
  pub fn add_graph(&mut self, graph: Graph) {
    self.graphs.insert(graph.name().clone(), graph);
  }

  pub fn get_graph(&self, name: &str) -> Option<&Graph> {
    self.graphs.get(name)
  }

  pub fn get_graph_mut(&mut self, name: &str) -> Option<&mut Graph> {
    self.graphs.get_mut(name)
  }

  pub fn get_graph_names(&self) -> Vec<String> {
    self.graphs.keys().cloned().collect()
  }
}
