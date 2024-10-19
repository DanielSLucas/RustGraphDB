use std::sync::Arc;  // Esta importação não está sendo usada, pode ser removida se desnecessária.
use std::error::Error;  // Adicione esta importação para o manejo de erros.
use reqwest::Client;
use serde::{Deserialize, Serialize}; // Inclua Serialize se for usar em JSON.
use tokio::task;
use chrono::Utc;

use seedDataFrame::lib::data::reader::CSVReader;
use seedDataFrame::lib::api::rest::GraphService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let file_path = "traffic_navigation_dataset copy.csv";  // Substitua com o caminho para o seu arquivo CSV

    // Lê os dados do CSV de forma assíncrona
    let csv_reader = CSVReader::read_csv(file_path).await?; // Lê os dados e armazena em um CSVReader

    // Inicializa o cliente HTTP
    let client = Arc::new(Client::new());

    let dt = Utc::now();
    let timestamp = dt.timestamp();
    let graph_name = format!("some_graph_{}", timestamp);

    // Define a URL base da API para onde os dados serão enviados
    let base_url = "http://localhost:8080".to_string();  // Substitua pelo endereço correto da API

    // Cria o GraphService com o cliente HTTP e URL base
    let graph_service = GraphService {
        client,
        base_url,
        graph_name,
        data: Arc::new(csv_reader), // Agora aqui é Arc<CSVReader>
    };

    // Envia os dados do CSV para os endpoints da API de forma simultânea
    graph_service.post_graph().await?;

    Ok(())
}
