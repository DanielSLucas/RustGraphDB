use serde::Deserialize;
use std::error::Error;
use rand::Rng; // Importa o módulo para geração de números aleatórios

#[derive(Debug, Deserialize, serde::Serialize, Clone)]
pub struct Node {
    pub Node_ID: usize,
    pub Node: String,
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

    // Retorna um ID aleatório de Node
    pub fn random_node_id(&self) -> usize {
        if self.records.is_empty() {
            return 0; // Retorna um ID padrão (por exemplo, 0) se não houver registros
        }
        
        // Gera um índice aleatório dentro do intervalo válido
        let mut rng = rand::thread_rng();
        let random_index = rng.gen_range(0..self.records.len());
        
        // Retorna o Node_ID correspondente
        self.records[random_index].Node_ID
    }
}

