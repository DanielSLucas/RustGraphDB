use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Seek, SeekFrom};
use std::path::PathBuf;

use crate::lib::graph::edge::Edge;
use crate::lib::graph::node::Node;
use crate::lib::graph::Graph;

use super::manager::WriteOperation;

const STORAGE_DIR: &str = "storage";

#[derive(Serialize, Deserialize)]
struct GraphMetadata {
  node_count: usize,
  edge_count: usize,
  next_node_id: usize,
  next_edge_id: usize,
}

#[derive(Clone)]
pub struct DiskStorage {
  storage_dir: PathBuf,
}

impl DiskStorage {
  pub fn new() -> io::Result<Self> {
    let storage_dir = DiskStorage::create_storage_dir_if_not_exists()?;

    Ok(Self { storage_dir })
  }

  fn create_storage_dir_if_not_exists() -> io::Result<PathBuf> {
    let storage_dir = PathBuf::from(STORAGE_DIR);

    if !storage_dir.exists() {
      fs::create_dir_all(&storage_dir).expect("Failed to create storage directory");
    }

    Ok(storage_dir)
  }

  pub async fn process_write_operation(&self, operation: WriteOperation) {
    match operation {
      WriteOperation::CreateGraph(graph_name, graph) => {
        let _ = self.create_graph(graph_name, graph);
      }
      WriteOperation::AddNode(graph_name, node) => {
        let _ = self.add_node(&graph_name, node);
      }
      WriteOperation::AddEdge(graph_name, edge) => {
        let _ = self.add_edge(&graph_name, edge);
      }
      WriteOperation::UpdateNode(graph_name, node) => {
        let _ = self.update_node(&graph_name, node);
      }
      WriteOperation::UpdateEdge(graph_name, edge) => {
        let _ = self.update_edge(&graph_name, edge);
      }
      WriteOperation::DeleteGraph(graph_name) => {
        let _ = self.delete_graph(&graph_name);
      }
      WriteOperation::DeleteNode(graph_name, node_id) => {
        let _ = self.delete_node(&graph_name, node_id);
      }
      WriteOperation::DeleteEdge(graph_name, edge_id) => {
        let _ = self.delete_edge(&graph_name, edge_id);
      }
    }
  }

  pub fn list_graph_names(&self) -> io::Result<Vec<String>> {
    let mut graph_names = Vec::new();
    for entry in fs::read_dir(&self.storage_dir)? {
      let entry = entry?;
      let path = entry.path();
      if let Some(extension) = path.extension() {
        if extension == "gph" {
          if let Some(name) = path.file_stem() {
            graph_names.push(name.to_string_lossy().into_owned());
          }
        }
      }
    }
    Ok(graph_names)
  }

  // Método para criar um novo grafo
  pub fn create_graph(&self, grafo_id: String, graph: Graph) -> io::Result<()> {
    let file_path = self.storage_dir.join(format!("{}.gph", grafo_id));
    if file_path.exists() {
      return Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "Grafo já existe",
      ));
    }

    let metadata = GraphMetadata {
      node_count: 0,
      edge_count: 0,
      next_node_id: 1,
      next_edge_id: 1,
    };

    let mut file = File::create(file_path)?;
    self.write_graph_metadata(&mut file, &metadata)?;
    self.write_graph_data(&mut file, &graph)
  }

  // Método para buscar um grafo pelo nome
  pub fn get_graph(&self, grafo_id: &str) -> io::Result<Option<Graph>> {
    let file_path = self.storage_dir.join(format!("{}.gph", grafo_id));
    if !file_path.exists() {
      return Ok(None);
    }

    let mut file = File::open(file_path)?;
    let _metadata = self.read_graph_metadata(&mut file)?;
    let graph = self.read_graph_data(&mut file)?;
    Ok(Some(graph))
  }

  // Método para adicionar um nó a um grafo
  pub fn add_node(&self, grafo_id: &str, node: Node) -> io::Result<()> {
    let file_path = self.storage_dir.join(format!("{}.gph", grafo_id));
    let mut file = File::options().read(true).write(true).open(file_path)?;

    let mut metadata = self.read_graph_metadata(&mut file)?;
    metadata.node_count += 1;
    metadata.next_node_id += 1;
    file.seek(SeekFrom::Start(0))?;
    self.write_graph_metadata(&mut file, &metadata)?;

    file.seek(SeekFrom::End(0))?;
    bincode::serialize_into(&mut file, &node).expect("Failed do add node");
    Ok(())
  }

  // Método para adicionar uma aresta a um grafo
  pub fn add_edge(&self, grafo_id: &str, edge: Edge) -> io::Result<()> {
    let file_path = self.storage_dir.join(format!("{}.gph", grafo_id));
    let mut file = File::options().read(true).write(true).open(file_path)?;

    let mut metadata = self.read_graph_metadata(&mut file)?;
    metadata.edge_count += 1;
    metadata.next_edge_id += 1;
    file.seek(SeekFrom::Start(0))?;
    self.write_graph_metadata(&mut file, &metadata)?;

    file.seek(SeekFrom::End(0))?;
    bincode::serialize_into(&mut file, &edge).expect("Failed do add edge");
    Ok(())
  }

  // Método para deletar um grafo
  pub fn delete_graph(&self, grafo_id: &str) -> io::Result<()> {
    let file_path = self.storage_dir.join(format!("{}.gph", grafo_id));
    if file_path.exists() {
      fs::remove_file(file_path)?;
      Ok(())
    } else {
      Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Grafo não encontrado",
      ))
    }
  }

  fn get_graph_path(&self, grafo_id: &str) -> PathBuf {
    self.storage_dir.join(format!("{}.gph", grafo_id))
  }

  // Método para atualizar um nó existente
  pub fn update_node(&self, grafo_id: &str, updated_node: Node) -> io::Result<()> {
    let file_path = self.get_graph_path(grafo_id);
    let mut file = OpenOptions::new().read(true).write(true).open(&file_path)?;
    let mut nodes_and_edges: Vec<Node> = Vec::new();

    // Ler e desserializar todo o conteúdo do arquivo para a memória
    file.seek(SeekFrom::Start(0))?;
    while let Ok(node) = bincode::deserialize_from::<_, Node>(&mut file) {
      // Se o nó tiver o mesmo ID, atualiza com o nó modificado
      if node.id == updated_node.id {
        nodes_and_edges.push(updated_node.clone());
      } else {
        nodes_and_edges.push(node);
      }
    }

    // Reescrever o arquivo com o nó atualizado
    file.set_len(0)?;
    for node in nodes_and_edges {
      bincode::serialize_into(&mut file, &node).expect("Failed to update node");
    }
    Ok(())
  }

  // Método para atualizar uma aresta existente
  pub fn update_edge(&self, grafo_id: &str, updated_edge: Edge) -> io::Result<()> {
    let file_path = self.get_graph_path(grafo_id);
    let mut file = OpenOptions::new().read(true).write(true).open(&file_path)?;
    let mut edges: Vec<Edge> = Vec::new();

    // Ler e desserializar todas as arestas para a memória
    file.seek(SeekFrom::Start(0))?;
    while let Ok(edge) = bincode::deserialize_from::<_, Edge>(&mut file) {
      if edge.id == updated_edge.id {
        edges.push(updated_edge.clone());
      } else {
        edges.push(edge);
      }
    }

    // Reescrever o arquivo com a aresta atualizada
    file.set_len(0)?;
    for edge in edges {
      bincode::serialize_into(&mut file, &edge).expect("Failed to update edge");
    }
    Ok(())
  }

  // Método para deletar um nó
  pub fn delete_node(&self, grafo_id: &str, node_id: usize) -> io::Result<()> {
    let file_path = self.get_graph_path(grafo_id);
    let mut file = OpenOptions::new().read(true).write(true).open(&file_path)?;
    let mut nodes: Vec<Node> = Vec::new();

    // Ler e desserializar todos os nós
    file.seek(SeekFrom::Start(0))?;
    while let Ok(node) = bincode::deserialize_from::<_, Node>(&mut file) {
      if node.id != node_id {
        nodes.push(node);
      }
    }

    // Reescrever o arquivo com os nós restantes
    file.set_len(0)?;
    for node in nodes {
      bincode::serialize_into(&mut file, &node).expect("Failed to delete node");
    }
    Ok(())
  }

  // Método para deletar uma aresta
  pub fn delete_edge(&self, grafo_id: &str, edge_id: usize) -> io::Result<()> {
    let file_path = self.get_graph_path(grafo_id);
    let mut file = OpenOptions::new().read(true).write(true).open(&file_path)?;
    let mut edges: Vec<Edge> = Vec::new();

    // Ler e desserializar todas as arestas
    file.seek(SeekFrom::Start(0))?;
    while let Ok(edge) = bincode::deserialize_from::<_, Edge>(&mut file) {
      if edge.id != edge_id {
        edges.push(edge);
      }
    }

    // Reescrever o arquivo com as arestas restantes
    file.set_len(0)?;
    for edge in edges {
      bincode::serialize_into(&mut file, &edge).expect("Failed to delete edge");
    }
    Ok(())
  }

  // Métodos para ler e escrever metadados do grafo
  fn write_graph_metadata(&self, file: &mut File, metadata: &GraphMetadata) -> io::Result<()> {
    file.seek(SeekFrom::Start(0))?;
    bincode::serialize_into(file, metadata).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
  }

  fn read_graph_metadata(&self, file: &mut File) -> io::Result<GraphMetadata> {
    file.seek(SeekFrom::Start(0))?;
    bincode::deserialize_from(file).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
  }

  // Métodos para ler e escrever dados do grafo (nós e arestas)
  fn write_graph_data(&self, file: &mut File, graph: &Graph) -> io::Result<()> {
    file.seek(SeekFrom::End(0))?;
    bincode::serialize_into(file, graph).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
  }

  fn read_graph_data(&self, file: &mut File) -> io::Result<Graph> {
    file.seek(SeekFrom::End(0))?;
    bincode::deserialize_from(file).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
  }
}
