use std::sync::Arc;
use std::error::Error; 
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::task;
use chrono::Utc;

use seedDataFrame::lib::data::{reader_edges::CSVReaderEdge, reader_nodes::CSVReaderNode};
use seedDataFrame::lib::api::rest::GraphService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let file_path_nodes = "nodes.csv";

    // Lê os dados do CSV de forma assíncrona
    let csv_reader_nodes = CSVReaderNode::read_csv(file_path_nodes).await?; // Lê os dados e armazena em um CSVReader

    let file_path_edges = "edges.csv";

    // Lê os dados do CSV de forma assíncrona
    let csv_reader_edges = CSVReaderEdge::read_csv(file_path_edges).await?; // Lê os dados e armazena em um CSVReader

    // Inicializa o cliente HTTP
    let client = Arc::new(Client::new());

    let dt = Utc::now();
    let timestamp = dt.timestamp();
    let graph_name = format!("some_graph_{}", timestamp);

    // Define a URL base da API para onde os dados serão enviados
    let base_url = "http://localhost:8080".to_string();

    // Cria o GraphService com o cliente HTTP e URL base
    let graph_service = GraphService {
        client,
        base_url,
        graph_name,
        data_nodes: Arc::new(csv_reader_nodes),
        data_edges: Arc::new(csv_reader_edges)
    };

    // Envia os dados do CSV para os endpoints da API de forma simultânea
    graph_service.post_graph().await?;

    Ok(())
}
