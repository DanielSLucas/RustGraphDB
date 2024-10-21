use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use std::error::Error;
use tokio::task;

#[derive(Debug, Deserialize, serde::Serialize)]
pub struct RoadData {
    pub street: String,
    pub from: String,
    pub to: String,
    pub distance_km: f64,
    pub travel_time_min: f64,
    pub congestion_level: f64,
}

pub struct CSVReader {
    pub records: Vec<RoadData>, // Armazena os registros lidos
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

    pub fn get_records(&self) -> Vec<RoadData> {
        self.records.clone() // Retorna uma cópia dos registros
    }

    // Retorna um iterador sobre os registros
    pub fn iter(&self) -> std::slice::Iter<RoadData> {
        self.records.iter()
    }

    // Remove um registro baseado na `street`
    pub fn remove_record(&mut self, street: &str) -> bool {
        println!("FFFFFFF");
        if let Some(pos) = self.records.iter().position(|r| r.street == street) {
            println!("PORA");
            self.records.remove(pos);
            true // Registro removido com sucesso
        } else {
            false // Registro não encontrado
        }
    }
}
