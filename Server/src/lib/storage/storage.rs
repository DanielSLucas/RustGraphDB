use serde_json;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use tokio::fs as async_fs;
use tokio::io::AsyncWriteExt;

use crate::lib::graph::Graph;

#[derive(Debug)]
pub struct StorageManager {
  graphs: RwLock<HashMap<String, Arc<Mutex<Graph>>>>,
  storage_dir: PathBuf,
}

impl Clone for StorageManager {
  fn clone(&self) -> Self {
    Self {
      graphs: RwLock::new(self.graphs.read().unwrap().clone()),
      storage_dir: self.storage_dir.clone(),
    }
  }
}

impl StorageManager {
  pub fn new(storage_dir: &str) -> Self {
    let storage_path = PathBuf::from(storage_dir);
    if !storage_path.exists() {
      fs::create_dir_all(&storage_path).expect("Failed to create storage directory");
    }

    Self {
      graphs: RwLock::new(HashMap::new()),
      storage_dir: storage_path,
    }
  }

  pub async fn get_graph(&self, name: &str) -> Option<Arc<Mutex<Graph>>> {
    {
      // First, try to read from the in-memory cache
      let graphs = self.graphs.read().unwrap();
      if let Some(graph) = graphs.get(name) {
        return Some(Arc::clone(graph));
      }
    }

    // If not in memory, try to load it from disk
    match self.load_graph_from_file(name).await {
      Ok(graph) => {
        let graph_arc = Arc::new(Mutex::new(graph));
        let mut graphs = self.graphs.write().unwrap();
        graphs.insert(name.to_string(), Arc::clone(&graph_arc));
        Some(graph_arc)
      }
      Err(_) => None,
    }
  }

  async fn load_graph_from_file(&self, name: &str) -> std::io::Result<Graph> {
    let filename = format!("{}.json", name);
    let filepath = self.storage_dir.join(filename);

    let data = async_fs::read_to_string(filepath).await?;
    let graph: Graph = serde_json::from_str(&data).expect("Failed to deserialize graph");
    Ok(graph)
  }

  pub async fn save_graph(&self, name: &str) -> std::io::Result<()> {
    let graphs = self.graphs.read().unwrap();

    if let Some(graph_arc) = graphs.get(name) {
      let graph = graph_arc.lock().unwrap();
      let data = serde_json::to_string_pretty(&*graph).expect("Failed to serialize graph");
      let filename = format!("{}.json", name);
      let filepath = self.storage_dir.join(filename);

      let mut file = async_fs::File::create(filepath).await?;
      file.write_all(data.as_bytes()).await?;
    }

    Ok(())
  }

  pub async fn add_graph(&self, graph: Graph) -> std::io::Result<()> {
    let graph_arc = Arc::new(Mutex::new(graph));
    let name = graph_arc.lock().unwrap().name().clone();

    {
      let mut graphs = self.graphs.write().unwrap();
      graphs.insert(name.clone(), Arc::clone(&graph_arc));
    }

    self.save_graph(&name).await
  }

  pub fn get_graph_names(&self) -> Vec<String> {
    let graph_files = self.get_graph_files();

    graph_files
      .into_iter()
      .filter_map(|path| {
        path
          .file_stem() // remove o .json do nome
          .and_then(OsStr::to_str)
          .map(|name| name.to_string())
      })
      .collect()
  }

  fn get_graph_files(&self) -> Vec<PathBuf> {
    fs::read_dir(&self.storage_dir)
      .expect("Failed to read storage directory")
      .filter_map(|entry| {
        let entry = entry.ok()?;
        let path = entry.path();
        if path.is_file() && path.extension() == Some(OsStr::new("json")) {
          Some(path)
        } else {
          None
        }
      })
      .collect()
  }

  pub fn save_all_graphs_sync(&self) -> std::io::Result<()> {
    let graphs = self.graphs.read().unwrap();

    for (name, graph_arc) in graphs.iter() {
      let graph = graph_arc.lock().unwrap();
      let data = serde_json::to_string_pretty(&*graph).expect("Failed to serialize graph");
      let filename = format!("{}.json", name);
      let filepath = self.storage_dir.join(filename);

      fs::write(filepath, data.as_bytes())?;
    }

    Ok(())
  }
}
