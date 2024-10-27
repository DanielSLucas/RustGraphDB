use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize, serde::Serialize, Clone)]
pub struct Node {
    pub Node_ID: usize,
    pub Node: String
}
#[derive(Clone)]
pub struct CSVReaderNode {
    records: Vec<Node>, // Armazena os registros lidos
}

impl CSVReaderNode {
    // Lê o arquivo CSV e armazena os registros
    pub async fn read_csv(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = tokio::fs::File::open(file_path).await?;
        let mut rdr = csv::Reader::from_reader(file.into_std().await);
        let mut records = Vec::new();

        for result in rdr.deserialize() {
            let record: Node = result?;
            records.push(record);
        }

        Ok(CSVReaderNode { records })
    }

    // Retorna um iterador sobre os registros
    pub fn iter(&self) -> std::slice::Iter<Node> {
        self.records.iter()
    }

    // Retorna o número de registros
    pub fn len(&self) -> usize {
        self.records.len()
    }
}
