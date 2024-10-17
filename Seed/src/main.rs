use std::sync::Arc;  // Esta importação não está sendo usada, pode ser removida se desnecessária.
use std::error::Error;  // Adicione esta importação para o manejo de erros.
use reqwest::Client;
use serde::Deserialize;
use tokio::task;

use seedDataFrame::lib::data::reader::CSVReader;
use seedDataFrame::lib::api::rest::GraphService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let file_path = "traffic_navigation_dataset.csv";  // Substitua com o caminho para o seu arquivo CSV

    // Lê os dados do CSV de forma assíncrona
    let records = CSVReader::read_csv(file_path).await?;

    // Inicializa o cliente HTTP
    let client = Arc::new(Client::new());

    // Define a URL base da API para onde os dados serão enviados
    let base_url = "http://localhost:8080".to_string();  // Substitua pelo endereço correto da API

    // Cria o GraphService com o cliente HTTP e URL base
    let graph_service = GraphService {
        client,
        base_url,
    };

    // Envia os dados do CSV para os endpoints da API de forma simultânea
    graph_service.send_to_endpoints_concurrently(records).await;

    Ok(())
}