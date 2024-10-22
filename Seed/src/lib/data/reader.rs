use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use std::error::Error;
use tokio::task;

#[derive(Debug, Deserialize, serde::Serialize)]
pub struct RoadData {
    pub Street: String,
    pub From: String,
    pub To: String,
    pub Distance_km: f64,
    pub Travel_time_min: f64,
    pub Congestion_level: f64,
}

pub struct CSVReader {
    records: Vec<RoadData>, // Armazena os registros lidos
}

impl CSVReader {
    // Lê o arquivo CSV e armazena os registros
    pub async fn read_csv(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = tokio::fs::File::open(file_path).await?;
        let mut rdr = csv::Reader::from_reader(file.into_std().await);
        let mut records = Vec::new();

        for result in rdr.deserialize() {
            let record: RoadData = result?;
            records.push(record);
        }

        Ok(CSVReader { records })
    }

    // Retorna um iterador sobre os registros
    pub fn iter(&self) -> std::slice::Iter<RoadData> {
        self.records.iter()
    }
}
