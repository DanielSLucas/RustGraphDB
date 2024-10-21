use std::collections::HashMap;
use std::{fs, path::PathBuf};
use tokio::fs as async_fs;
use tokio::io::AsyncWriteExt;

use crate::lib::graph::edge::Edge;
use crate::lib::graph::node::Node;
use crate::lib::graph::Graph;

const NODES_FILE: &str = "nodes.csv";
const EDGES_FILE: &str = "edges.csv";
const STORAGE_DIR: &str = "storage";

#[derive(Clone)]
pub struct DiskStorageManager {}

impl DiskStorageManager {
  pub fn new() -> Self {
    DiskStorageManager::create_storage_dir_if_not_exists();

    Self {}
  }

  fn create_storage_dir_if_not_exists() {
    let storage_path = PathBuf::from(STORAGE_DIR);

    if !storage_path.exists() {
      fs::create_dir_all(&storage_path).expect("Failed to create storage directory");
    }
  }

  fn get_graph_folder_names() -> Vec<String> {
    let graph_folders = DiskStorageManager::get_graph_folders();

    graph_folders
      .into_iter()
      .map(|path| path.to_str().unwrap().to_string())
      .collect()
  }

  fn get_graph_folders() -> Vec<PathBuf> {
    fs::read_dir(PathBuf::from(STORAGE_DIR))
      .expect("Failed to read storage directory")
      .filter_map(|entry| {
        let entry = entry.ok()?;
        let path = entry.path();
        if path.is_dir() {
          Some(path)
        } else {
          None
        }
      })
      .collect()
  }

  pub fn get_graph_names(&self) -> Vec<String> {
    DiskStorageManager::get_graph_folder_names()
  }

  pub async fn create_graph_dir(&self, name: &str) -> std::io::Result<()> {
    let graph_dir = PathBuf::from(STORAGE_DIR).join(name);

    if !graph_dir.exists() {
      fs::create_dir_all(&graph_dir)?;

      let mut nodes_file = async_fs::File::create(graph_dir.clone().join(NODES_FILE)).await?;
      nodes_file
        .write_all("id,label,properties\n".as_bytes())
        .await?;

      let mut edges_file = async_fs::File::create(graph_dir.clone().join(EDGES_FILE)).await?;
      edges_file
        .write_all("id,label,from,to,properties\n".as_bytes())
        .await?;
    }

    Ok(())
  }

  pub async fn add_node_to_file(&self, graph_name: &str, node: &Node) -> std::io::Result<()> {
    let graph_dir = PathBuf::from(STORAGE_DIR).join(graph_name);
    let nodes_file = graph_dir.join(NODES_FILE);

    if graph_dir.exists() {
      let mut file = async_fs::OpenOptions::new()
        .append(true)
        .open(nodes_file)
        .await?;

      let properties = node
        .properties
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");

      let line = format!("{},{},{}\n", node.id, node.label, properties);

      file.write_all(line.as_bytes()).await?;
    }

    Ok(())
  }

  pub async fn add_edge_to_file(&self, graph_name: &str, edge: &Edge) -> std::io::Result<()> {
    let graph_dir = PathBuf::from(STORAGE_DIR).join(graph_name);
    let edges_file = graph_dir.join(EDGES_FILE);

    if graph_dir.exists() {
      let mut file = async_fs::OpenOptions::new()
        .append(true)
        .open(edges_file)
        .await?;

      let properties = edge
        .properties
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");
      let line = format!(
        "{},{},{},{},{}\n",
        edge.id, edge.label, edge.from, edge.to, properties
      );
      file.write_all(line.as_bytes()).await?;
    }

    Ok(())
  }

  pub async fn load_graph_from_file(&self, name: &str) -> std::io::Result<Graph> {
    let graph = Graph::new(name.to_string());

    let graph = self.load_nodes_from_file(graph).await?;
    let graph = self.load_edges_from_file(graph).await?;

    Ok(graph)
  }

  async fn load_nodes_from_file(&self, mut graph: Graph) -> std::io::Result<Graph> {
    let nodes_filepath = PathBuf::from(STORAGE_DIR)
      .join(graph.name())
      .join(NODES_FILE);
    let data = async_fs::read_to_string(nodes_filepath).await?;

    for row in data.split("\n") {
      let row: Vec<&str> = row.split(",").collect();

      let id = row.get(0).unwrap().parse().unwrap();
      let label = row.get(1).unwrap().to_string();
      let properties = self.get_properties(row.get(2).unwrap());

      graph.add_node(Node::new(id, label, properties));
    }

    Ok(graph)
  }

  async fn load_edges_from_file(&self, mut graph: Graph) -> std::io::Result<Graph> {
    let edges_filepath = PathBuf::from(STORAGE_DIR)
      .join(graph.name())
      .join(EDGES_FILE);
    let data = async_fs::read_to_string(edges_filepath).await?;

    for row in data.split("\n") {
      let row: Vec<&str> = row.split(",").collect();

      let id = row.get(0).unwrap().parse().unwrap();
      let label = row.get(1).unwrap().to_string();
      let from = row.get(2).unwrap().parse().unwrap();
      let to = row.get(3).unwrap().parse().unwrap();
      let properties = self.get_properties(row.get(4).unwrap());

      graph.add_edge(Edge::new(id, label, from, to, properties));
    }

    Ok(graph)
  }

  fn get_properties(&self, props_string: &str) -> HashMap<String, String> {
    let mut hash_map = HashMap::new();
    let keys_n_values: Vec<(&str, &str)> = props_string
      .split("&")
      .map(|property| {
        let splited_prop: Vec<&str> = property.split("=").collect();

        (*splited_prop.get(0).unwrap(), *splited_prop.get(1).unwrap())
      })
      .collect();

    for (key, value) in keys_n_values {
      hash_map.insert(key.to_string(), value.to_string());
    }

    hash_map
  }
}
