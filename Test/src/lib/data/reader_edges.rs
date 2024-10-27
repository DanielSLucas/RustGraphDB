use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use std::error::Error;
use tokio::task;
use rand::Rng;

#[derive(Debug, Deserialize, serde::Serialize, Clone)]
pub struct Edge {
    pub Street: String,
    pub From: usize,
    pub To: usize,
    pub Distance_km: f64,
    pub Travel_time_min: f64,
    pub Congestion_level: f64,
}
#[derive(Clone)]
pub struct CSVReaderEdge {
    records: Vec<Edge>, // Armazena os registros lidos
}

impl CSVReaderEdge {
    // Lê o arquivo CSV e armazena os registros
    pub async fn read_csv(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = tokio::fs::File::open(file_path).await?;
        let mut rdr = csv::Reader::from_reader(file.into_std().await);
        let mut records = Vec::new();

        for result in rdr.deserialize() {
            let record: Edge = result?;
            records.push(record);
        }

        Ok(CSVReaderEdge { records })
    }

    // Retorna um iterador sobre os registros
    pub fn iter(&self) -> std::slice::Iter<Edge> {
        self.records.iter()
    }

    // Retorna o número de registros
    pub fn len(&self) -> usize {
        self.records.len()
    }

    // Retorna um dos últimos 3 atributos de forma aleatória
    pub fn random_last_attribute(&self) -> String {
        let mut rng = rand::thread_rng();
        let propriedades = ["Distance_km", "Travel_time_min", "Congestion_level"];
        let index = rng.gen_range(0..propriedades.len());

        propriedades[index].to_string()
    }
}
