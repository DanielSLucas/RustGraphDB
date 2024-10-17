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
    pub Congestion_level: String,
}

pub struct CSVReader;

impl CSVReader {
    pub async fn read_csv(file_path: &str) -> Result<Vec<RoadData>, Box<dyn Error>> {
        let file = tokio::fs::File::open(file_path).await?;
        let mut rdr = csv::Reader::from_reader(file.into_std().await);
        let mut records = Vec::new();

        for result in rdr.deserialize() {
            let record: RoadData = result?;
            records.push(record);
        }

        Ok(records)
    }
}