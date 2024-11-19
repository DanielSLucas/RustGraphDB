use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::task;

use testServer::lib::api::{
  get_relations::{Graph, Relation},
  get_search_server::SearchServer,
  post_datas::GraphService,
};
use testServer::lib::data::{reader_edges::CSVReaderEdge, reader_nodes::CSVReaderNode};
use testServer::lib::log::write_log::TextLogger;

const QTD_BUSCAS: usize = 1000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let current_dir = env::current_dir()?;
  let base_dir: PathBuf = current_dir.join("databases");

  let nodes_file = "nodes.csv";
  let edges_file = "edges.csv";

  // Constrói os caminhos completos usando Path
  let file_path_nodes = base_dir.join(nodes_file);
  let file_path_edges = base_dir.join(edges_file);

  // Lê os dados do CSV de forma assíncrona
  let csv_reader_nodes = CSVReaderNode::read_csv(file_path_nodes.to_str().unwrap()).await?;
  let csv_reader_edges = CSVReaderEdge::read_csv(file_path_edges.to_str().unwrap()).await?;

  // Inicializa o cliente HTTP
  let client = Arc::new(Client::new());

  let dt = Utc::now();
  let timestamp = dt.timestamp();
  let graph_name = format!("graph{}", timestamp);

  // Define a URL base da API para onde os dados serão enviados
  let base_url = "http://localhost:8080".to_string();

  // Cria o GraphService com o cliente HTTP e URL base
  let mut graph_service = GraphService {
    client: client.clone(),
    base_url: base_url.clone(),
    graph_name: graph_name.clone(),
    data_nodes: Arc::new(csv_reader_nodes.clone()),
    data_edges: Arc::new(csv_reader_edges.clone()),
    nodes_id: Arc::new(Mutex::new(HashMap::new())),
  };

  let base_dir_log: PathBuf = current_dir.join("logs");
  let log_file_path = base_dir_log.join(format!("{}_output.txt", graph_name));
  let logger = TextLogger::new(log_file_path.to_str().unwrap_or_default().to_string());

  // Loga o início da execução
  logger
    .log(format!("Iniciando criação do grafo '{}'", graph_name))
    .await;

  // Envia os dados do CSV para os endpoints da API de forma simultânea
  match graph_service.post_graph().await {
    Ok(log_text) => {
      logger.log(log_text).await;
      logger
        .log("Grafo criado e nós/arestas adicionados com sucesso.".to_string())
        .await;
    }
    Err(e) => {
      logger
        .log(format!(
          "Erro ao criar grafo ou adicionar nós/arestas: {:?}",
          e
        ))
        .await;
      return Err(e);
    }
  };

  // Configura o SearchServer para buscar relações no grafo
  let search_server = SearchServer {
    client: client.clone(),
    base_url: base_url.clone(),
    graph_name: graph_name.clone(),
    data: Arc::new(csv_reader_nodes.clone()),
    num_search: 1000,
    edges: Arc::new(csv_reader_edges.clone()),
  };

  match search_server.search().await {
    Ok(log_text) => {
      logger.log(log_text).await;
      logger
        .log("Buscas realizada com sucesso.".to_string())
        .await;
    }
    Err(e) => {
      logger.log(format!("Erro na busca BFS: {:?}", e)).await;
      return Err(e);
    }
  };

  // Finaliza e salva o log
  logger.log("Execução completa.".to_string()).await;
  logger.write_to_file().await?;

  let log_file_path = base_dir_log.join(format!("{}_relation.txt", graph_name));
  let logger = TextLogger::new(log_file_path.to_str().unwrap_or_default().to_string());

  let mut graph = Graph::new(client.clone(), graph_name.clone(), base_url.clone());
  let relation_log = graph.get_relation().await?;
  logger.log(relation_log).await;

  // Finaliza e salva o log
  logger
    .log("Finalizando registro das relações.\n".to_string())
    .await;
  logger.write_to_file().await?;

  Ok(())
}
