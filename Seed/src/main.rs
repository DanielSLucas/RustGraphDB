use std::sync::{Arc, Mutex};  // Inclui Mutex para proteção de dados entre threads
use std::error::Error;
use reqwest::Client;
use serde::{Deserialize, Serialize}; // Inclua Serialize se for usar em JSON.
use tokio::task;
use chrono::Utc;

use seedDataFrame::lib::data::reader::CSVReader;
use seedDataFrame::lib::api::rest::GraphService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let file_path = "traffic_navigation_dataset copy.csv";  // Substitua com o caminho correto para o arquivo CSV

    // Lê os dados do CSV de forma assíncrona
    let csv_reader = CSVReader::read_csv(file_path).await?; // Lê os dados e armazena em um CSVReader

    // Inicializa o cliente HTTP
    let client = Arc::new(Client::new());

    // Gera um timestamp para o nome do grafo
    let dt = Utc::now();
    let timestamp = dt.timestamp();
    let graph_name = format!("some_graph_{}", timestamp);

    // Encapsula o CSVReader em um Mutex e depois em um Arc para acesso seguro
    let csv_reader_arc = Arc::new(Mutex::new(csv_reader)); // Encapsula CSVReader no Mutex e depois no Arc

    // Cria o GraphService com o cliente HTTP e URL base
    let graph_service = GraphService {
        client,
        base_url: "http://localhost:8080".to_string(),  // Substitua pelo endereço correto da API
        graph_name,
        data: csv_reader_arc, // Agora aqui é Arc<Mutex<CSVReader>>
    };

    // Envia os dados do CSV para os endpoints da API de forma simultânea
    graph_service.post_graph().await?;

    Ok(())
}

