use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::Arc;

use crate::lib::graph::edge::Edge;
use crate::lib::graph::node::Node;
use crate::lib::graph::Graph;
use crate::lib::storage::id_generator::IdGenerator;
use crate::lib::utils::logger::log_info;

use super::manager::WriteOperation;

const STORAGE_DIR: &str = "storage";
const HEADER_SIZE: u64 = 1024; // Tamanho fixo para o cabeçalho
const BLOCK_SIZE: usize = 1024; // Tamanho do bloco de leitura/escrita

#[derive(Debug, Serialize, Deserialize)]
struct GraphHeader {
  name: String,
  next_node_id: usize,
  next_edge_id: usize,
  node_count: usize,
  edge_count: usize,
  first_node_position: u64,
  first_edge_position: u64,
  deleted_nodes: Vec<usize>, // IDs dos nós deletados
  deleted_edges: Vec<usize>, // IDs das arestas deletadas
}

impl GraphHeader {
  fn new(name: String) -> Self {
    Self {
      name,
      next_node_id: 1,
      next_edge_id: 1,
      node_count: 0,
      edge_count: 0,
      first_node_position: HEADER_SIZE,
      first_edge_position: HEADER_SIZE + BLOCK_SIZE as u64,
      deleted_nodes: Vec::new(),
      deleted_edges: Vec::new(),
    }
  }
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
      fs::create_dir_all(&storage_dir)?;
    }
    Ok(storage_dir)
  }

  pub async fn process_write_operation(&self, operation: WriteOperation) {
    match operation {
      WriteOperation::CreateGraph(graph_name, _) => {
        let _ = self.create_graph(&graph_name);
      }
      WriteOperation::AddNode(graph_name, node) => {
        let _ = self.append_node(&graph_name, &node);
      }
      WriteOperation::AddEdge(graph_name, edge) => {
        let _ = self.append_edge(&graph_name, &edge);
      }
      WriteOperation::UpdateNode(graph_name, node) => {
        let _ = self.update_node(&graph_name, &node);
      }
      WriteOperation::UpdateEdge(graph_name, edge) => {
        let _ = self.update_edge(&graph_name, &edge);
      }
      WriteOperation::DeleteNode(graph_name, node_id) => {
        let _ = self.mark_node_as_deleted(&graph_name, node_id);
      }
      WriteOperation::DeleteEdge(graph_name, edge_id) => {
        let _ = self.mark_edge_as_deleted(&graph_name, edge_id);
      }
      WriteOperation::DeleteGraph(graph_name) => {
        let _ = self.delete_graph(&graph_name);
      }
    }
  }

  fn get_file_path(&self, graph_name: &str) -> PathBuf {
    self.storage_dir.join(format!("{}.gph", graph_name))
  }

  pub fn create_graph(&self, graph_name: &str) -> io::Result<()> {
    let file_path = self.get_file_path(graph_name);
    let mut file = OpenOptions::new()
      .write(true)
      .create_new(true)
      .open(file_path)?;

    let header = GraphHeader::new(graph_name.to_string());
    self.write_header(&mut file, &header)
  }

  fn write_header(&self, file: &mut File, header: &GraphHeader) -> io::Result<()> {
    file.seek(SeekFrom::Start(0))?;
    let header_data =
      bincode::serialize(header).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    // Garante que o cabeçalho tem tamanho fixo
    let mut padded_header = vec![0u8; HEADER_SIZE as usize];
    padded_header[..header_data.len()].copy_from_slice(&header_data);

    file.write_all(&padded_header)
  }

  fn read_header(&self, file: &mut File) -> io::Result<GraphHeader> {
    let mut header_data = vec![0u8; HEADER_SIZE as usize];
    file.seek(SeekFrom::Start(0))?;
    file.read_exact(&mut header_data)?;

    bincode::deserialize(&header_data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
  }

  pub fn append_node(&self, graph_name: &str, node: &Node) -> io::Result<()> {
    let file_path = self.get_file_path(graph_name);
    let mut file = OpenOptions::new().read(true).write(true).open(file_path)?;

    log_info(&format!("appending node to graph [{}]", graph_name));

    let mut header = self.read_header(&mut file)?;

    log_info(&format!("{:?}", header));

    let mut padded_data = vec![0u8; BLOCK_SIZE];

    {
      let node_data =
        bincode::serialize(node).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

      log_info(&format!("node_data.size: {}", node_data.len()));

      if node_data.len() > BLOCK_SIZE {
        return Err(io::Error::new(
          io::ErrorKind::OutOfMemory,
          "Node data is bigger than 1024 bytes",
        ));
      }

      padded_data[..node_data.len()].copy_from_slice(&node_data);
    }

    // Calcula a posição de escrita
    let node_write_position =
      header.first_node_position + (header.node_count as u64 * BLOCK_SIZE as u64);
    log_info(&format!("node_write_position [{}]", node_write_position));

    if node_write_position >= header.first_edge_position {
      self.move_edges_forward(&mut file, &mut header)?;
    }

    log_info(&format!("node wrote at position [{}]", node_write_position));
    file.seek(SeekFrom::Start(node_write_position))?;
    file.write_all(&padded_data)?;

    // Atualiza o cabeçalho
    header.node_count += 1;
    header.next_node_id = header.next_node_id.max(node.id + 1);
    self.write_header(&mut file, &header)
  }

  fn move_edges_forward(&self, file: &mut File, header: &mut GraphHeader) -> io::Result<()> {
    log_info(&format!("Moving the first edge to the end of the file"));
    let mut buffer = vec![0u8; BLOCK_SIZE];

    file.seek(SeekFrom::Start(header.first_edge_position))?;
    match file.read_exact(&mut buffer) {
      Ok(_) => {
        log_info(&format!(
          "Moving it from [{}] position",
          header.first_edge_position
        ));
        println!(
          "Moving it to [{}] position",
          header.first_edge_position + (header.edge_count as u64 * BLOCK_SIZE as u64),
        );
        file.seek(SeekFrom::Start(
          header.first_edge_position + (header.edge_count as u64 * BLOCK_SIZE as u64),
        ))?;
        file.write_all(&buffer)?;
      }
      Err(_) => log_info(&format!("There is nothing to move!")),
    };

    header.first_edge_position += BLOCK_SIZE as u64;
    println!(
      "first_edge_position updated to [{}]",
      header.first_edge_position
    );
    // self.write_header(file, &header)?;

    Ok(())
  }

  pub fn append_edge(&self, graph_name: &str, edge: &Edge) -> io::Result<()> {
    let file_path = self.get_file_path(graph_name);
    let mut file = OpenOptions::new().read(true).write(true).open(file_path)?;

    log_info(&format!("appending edge to graph [{}]", graph_name));

    let mut header = self.read_header(&mut file)?;

    log_info(&format!("{:?}", header));

    let mut padded_data = vec![0u8; BLOCK_SIZE];
    {
      let edge_data =
        bincode::serialize(edge).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

      log_info(&format!("edge_data.size: {}", edge_data.len()));

      if edge_data.len() > BLOCK_SIZE {
        return Err(io::Error::new(
          io::ErrorKind::OutOfMemory,
          "Edge data is bigger than 1024 bytes",
        ));
      }

      padded_data[..edge_data.len()].copy_from_slice(&edge_data);
    }

    // Calcula a posição de escrita
    let write_position =
      header.first_edge_position + (header.edge_count as u64 * BLOCK_SIZE as u64);

    log_info(&format!("edge_write_position [{}]", write_position));

    // Escreve a aresta
    file.seek(SeekFrom::Start(write_position))?;
    file.write_all(&padded_data)?;

    // Atualiza o cabeçalho
    header.edge_count += 1;
    header.next_edge_id = header.next_edge_id.max(edge.id + 1);
    self.write_header(&mut file, &header)
  }

  pub fn get_graph(&self, graph_name: &str) -> io::Result<Option<Graph>> {
    let file_path = self.get_file_path(graph_name);
    if !file_path.exists() {
      return Ok(None);
    }

    let mut file = File::open(file_path)?;
    let header = self.read_header(&mut file)?;
    log_info(&format!("{:?}", header));

    // Cria o grafo com o IdGenerator inicializado corretamente
    let id_generator = Arc::new(IdGenerator::from(header.next_node_id, header.next_edge_id));
    let mut graph = Graph::new(header.name.clone(), id_generator);
    log_info(&format!("#1 {:?}", graph));

    // Lê os nós em blocos
    if header.node_count > 0 {
      let mut buffer = vec![0u8; BLOCK_SIZE];

      for i in 0..header.node_count {
        let position = header.first_node_position + (i as u64 * BLOCK_SIZE as u64);
        file.seek(SeekFrom::Start(position))?;
        file
          .read_exact(&mut buffer)
          .expect(&format!("Failed to read node at position [{}]", position));

        let node = bincode::deserialize::<Node>(&buffer).expect("Failed to deserialize node");

        if !header.deleted_nodes.contains(&node.id) {
          graph.add_full_node(node);
        }
      }
    }

    // Lê as arestas em blocos
    if header.edge_count > 0 {
      let mut buffer = vec![0u8; BLOCK_SIZE];

      for i in 0..header.edge_count {
        let position = header.first_edge_position + (i as u64 * BLOCK_SIZE as u64);
        file.seek(SeekFrom::Start(position))?;
        file
          .read_exact(&mut buffer)
          .expect(&format!("Failed to read edge at position [{}]", position));

        let edge = bincode::deserialize::<Edge>(&buffer).expect("Failed to deserialize edge");

        if !header.deleted_edges.contains(&edge.id) {
          graph.add_full_edge(edge);
        }
      }
    }

    Ok(Some(graph))
  }

  pub fn update_node(&self, graph_name: &str, node: &Node) -> io::Result<()> {
    let file_path = self.get_file_path(graph_name);
    let mut file = OpenOptions::new().read(true).write(true).open(file_path)?;

    let header = self.read_header(&mut file)?;

    // Procura o nó no arquivo
    let mut buffer = vec![0u8; BLOCK_SIZE];
    for i in 0..header.node_count {
      let position = header.first_node_position + (i as u64 * BLOCK_SIZE as u64);
      file.seek(SeekFrom::Start(position))?;
      file.read_exact(&mut buffer)?;

      if let Ok(existing_node) = bincode::deserialize::<Node>(&buffer) {
        if existing_node.id == node.id {
          // Encontrou o nó, atualiza
          file.seek(SeekFrom::Start(position))?;
          let node_data =
            bincode::serialize(node).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
          file.write_all(&node_data)?;
          return Ok(());
        }
      }
    }

    Err(io::Error::new(io::ErrorKind::NotFound, "Node not found"))
  }

  pub fn update_edge(&self, graph_name: &str, edge: &Edge) -> io::Result<()> {
    let file_path = self.get_file_path(graph_name);
    let mut file = OpenOptions::new().read(true).write(true).open(file_path)?;

    let header = self.read_header(&mut file)?;

    // Procura a aresta no arquivo
    let mut buffer = vec![0u8; BLOCK_SIZE];
    for i in 0..header.edge_count {
      let position = header.first_edge_position + (i as u64 * BLOCK_SIZE as u64);
      file.seek(SeekFrom::Start(position))?;
      file.read_exact(&mut buffer)?;

      if let Ok(existing_edge) = bincode::deserialize::<Edge>(&buffer) {
        if existing_edge.id == edge.id {
          // Encontrou a aresta, atualiza
          file.seek(SeekFrom::Start(position))?;
          let edge_data =
            bincode::serialize(edge).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
          file.write_all(&edge_data)?;
          return Ok(());
        }
      }
    }

    Err(io::Error::new(io::ErrorKind::NotFound, "Edge not found"))
  }

  pub fn mark_node_as_deleted(&self, graph_name: &str, node_id: usize) -> io::Result<()> {
    let file_path = self.get_file_path(graph_name);
    let mut file = OpenOptions::new().read(true).write(true).open(file_path)?;

    let mut header = self.read_header(&mut file)?;
    if !header.deleted_nodes.contains(&node_id) {
      header.deleted_nodes.push(node_id);
      self.write_header(&mut file, &header)?;
    }
    Ok(())
  }

  pub fn mark_edge_as_deleted(&self, graph_name: &str, edge_id: usize) -> io::Result<()> {
    let file_path = self.get_file_path(graph_name);
    let mut file = OpenOptions::new().read(true).write(true).open(file_path)?;

    let mut header = self.read_header(&mut file)?;
    if !header.deleted_edges.contains(&edge_id) {
      header.deleted_edges.push(edge_id);
      self.write_header(&mut file, &header)?;
    }
    Ok(())
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

  pub fn delete_graph(&self, graph_name: &str) -> io::Result<()> {
    let file_path = self.get_file_path(graph_name);
    if file_path.exists() {
      fs::remove_file(file_path)?;
    }
    Ok(())
  }
}
