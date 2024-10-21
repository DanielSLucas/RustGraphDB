use std::collections::HashMap;

use super::disk_storage::DiskStorageManager;
use crate::lib::graph::edge::Edge;
use crate::lib::graph::node::Node;
use crate::lib::graph::Graph;

pub struct StorageManager {
  graphs: HashMap<String, Graph>,
  disk_storage_manager: DiskStorageManager,
}

impl StorageManager {
  pub fn new() -> Self {
    Self {
      graphs: HashMap::new(),
      disk_storage_manager: DiskStorageManager::new(),
    }
  }

  pub fn get_graph_names(&self) -> Vec<String> {
    return self.disk_storage_manager.get_graph_names();
  }

  pub async fn get_graph(&mut self, name: &str) -> Option<Graph> {
    {
      // First, try to read from the in-memory cache
      if let Some(graph) = self.graphs.get(name) {
        return Some(graph.clone());
      }
    }

    // If not in memory, try to load it from disk
    match self.disk_storage_manager.load_graph_from_file(name).await {
      Ok(graph) => {
        self.graphs.insert(name.to_string(), graph.clone());
        Some(graph)
      }
      Err(_) => None,
    }
  }

  pub async fn add_node(&self, graph_name: String, node: Node) -> std::io::Result<()> {
    self.disk_storage_manager.add_node_to_file(&graph_name, &node).await?;    
    Ok(())
  }

  pub async fn add_edge(&self, graph_name: String, edge: Edge) -> std::io::Result<()> {
    self.disk_storage_manager.add_edge_to_file(&graph_name, &edge).await?;
    Ok(())
  }

  pub async fn save_graph(&mut self, graph: Graph) -> std::io::Result<()> {
    let name = graph.name().clone();

    self.disk_storage_manager.create_graph_dir(&name).await?;

    for (_key, value) in graph.nodes().iter() {
      self.add_node(name.clone(), value.clone()).await?;
    }

    for (_key, value) in graph.edges().iter() {
      self.add_edge(name.clone(), value.clone()).await?;
    }

    self.graphs.insert(name.clone(), graph);

    Ok(())
  }
}
