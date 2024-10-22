use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Instant};
use tokio::task;
use crate::lib::data::reader::{CSVReader, RoadData};
use std::collections::HashMap;

#[derive(Clone)]
pub struct GraphService {
    pub client: Arc<Client>,
    pub base_url: String,
    pub graph_name: String,
    pub data: Arc<CSVReader>, 
}

impl GraphService {
    // Função principal que cria o grafo e adiciona os nós
    pub async fn post_graph(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Cria o grafo
        self.create_graph().await?;

        // Adiciona os nós
        self.add_nodes_concurrently().await?;

        Ok(())
    }

    // Função para criar o grafo
    pub async fn create_graph(&self) -> Result<(), Box<dyn std::error::Error>> {
        let graph_url = format!("{}/graphs", self.base_url);

        // Dados do grafo a ser criado
        let graph_data = serde_json::json!({ "name": self.graph_name });

        let create_graph_res = self.client.post(&graph_url)
            .json(&graph_data)
            .send()
            .await?;

        if create_graph_res.status().is_success() {
            println!("Graph '{}' created successfully.", self.graph_name);
            Ok(())
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to create graph",
            )))
        }
    }

    pub async fn add_nodes_concurrently(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.client.clone();
        let base_url = self.base_url.clone();
        let graph_name = self.graph_name.clone();
        let data = self.data.clone(); // data é Arc<Vec<RoadData>>

        let start = Instant::now();

        // Processa cada registro em threads paralelas, enviando node_id e label (Street)
        let tasks: Vec<_> = data.iter().enumerate().map(|(index, record)| {
            let client = client.clone();
            let base_url = base_url.clone();
            let node_id = index;  // node_id é o índice do registro
            let label = record.Street.clone();  // label é o valor de Street  
            
            // Construindo o HashMap para propriedades
            let mut properties = HashMap::new();
            properties.insert("Distance_km", record.Distance_km.clone().to_string());
            properties.insert("Travel_time_min", record.Travel_time_min.clone().to_string());
            properties.insert("Congestion_level", record.Congestion_level.clone().to_string());
            
            // URL para adicionar um nó
            let node_url = format!("{}/graphs/{}/nodes", base_url, graph_name);
            
            // Criação de uma task assíncrona para adicionar o nó
            task::spawn(async move {
                // Serializando o HashMap de propriedades para um JSON adequado
                let res = client
                    .post(node_url)
                    .json(&serde_json::json!({
                        "node_id": node_id,
                        "label": label,
                        "properties": properties
                    }))
                    .send()
                    .await;

                match res {
                    Ok(response) if response.status().is_success() => {
                        // println!("Node added (ID: {}, Label: {}): {:?}", node_id, label, response.status());
                    }
                    Ok(response) => {
                        eprintln!("Failed to add node (ID: {}, Label: {}, Properties: {:?}): {:?}", node_id, label, properties, response.status());
                    }
                    Err(e) => {
                        eprintln!("Failed to add node (ID: {}, Label: {}): Properties: {:?}): {:?}", node_id, label, properties, e);
                    }
                }
            })
        }).collect();

        // Aguarda todas as tasks terminarem
        for task in tasks {
            let _ = task.await; // Espera cada tarefa terminar
        }

        let finish = Instant::now();
        println!("Tempo de execução em ms: {:?}", finish.duration_since(start));

        Ok(())
    }
}