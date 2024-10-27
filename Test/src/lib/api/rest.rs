use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Instant};
use tokio::task;
use crate::lib::data::{reader_edges::{CSVReaderEdge, Edge}, reader_nodes::{CSVReaderNode, Node}};
use std::collections::HashMap;

#[derive(Clone)]
pub struct GraphService {
    pub client: Arc<Client>,
    pub base_url: String,
    pub graph_name: String,
    pub data_nodes: Arc<CSVReaderNode>, 
    pub data_edges: Arc<CSVReaderEdge>, 
}

impl GraphService {
    // Função principal que cria o grafo e adiciona os nós
    pub async fn post_graph(&self) -> Result<(), Box<dyn std::error::Error>> {
        let start = Instant::now();
        // Cria o grafo
        self.create_graph().await?;

        // Adiciona os nós
        self.add_nodes_concurrently().await?;

        self.add_edges_concurrently().await?;

        let finish = Instant::now();
        println!("Tempo de execução total: {:?}", finish.duration_since(start));

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
        let data = self.data_nodes.clone();

        let start = Instant::now();

        // Processa cada registro em threads paralelas, enviando node_id e label (Street)
        let tasks: Vec<_> = data.iter().enumerate().map(|(index, record)| {
            let client = client.clone();
            let base_url = base_url.clone();
            let node_id = record.Node_ID.clone();  // node_id é o índice do registro
            let label = record.Node.clone();  // label é o valor de Street  
            
            // URL para adicionar um nó
            let node_url = format!("{}/graphs/{}/nodes", base_url, graph_name);
            
            // Criação de uma task assíncrona para adicionar o nó
            task::spawn(async move {
                // Serializando o HashMap de propriedades para um JSON adequado
                let res = client
                    .post(node_url)
                    .json(&serde_json::json!({
                        "node_id": node_id,
                        "label": label
                    }))
                    .send()
                    .await;

                match res {
                    Ok(response) if response.status().is_success() => {
                        // println!("Node added (ID: {}, Label: {}): {:?}", node_id, label, response.status());
                    }
                    Ok(response) => {
                        eprintln!("Failed to add node (ID: {}, Label: {}): {:?}", node_id, label, response.status());
                    }
                    Err(e) => {
                        eprintln!("Failed to add node (ID: {}, Label: {}): {:?}", node_id, label, e);
                    }
                }
            })
        }).collect();

        // Aguarda todas as tasks terminarem
        for task in tasks {
            let _ = task.await; // Espera cada tarefa terminar
        }

        let finish = Instant::now();
        println!("Tempo de execução Nodes: {:?}\nQuantidade de Nodes: {}\n------------------------------------", finish.duration_since(start), data.len());

        Ok(())
    }

    pub async fn add_edges_concurrently(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.client.clone();
        let base_url = self.base_url.clone();
        let graph_name = self.graph_name.clone();
        let data = self.data_edges.clone();

        let start = Instant::now();

        // Processa cada registro em threads paralelas, enviando node_id e label (Street)
        let tasks: Vec<_> = data.iter().enumerate().map(|(index, record)| {
            let client = client.clone();
            let base_url = base_url.clone();
            let edge_id = index;  // node_id é o índice do registro
            let label = record.Street.clone();  // label é o valor de Street  
            let from = record.From.clone();
            let to = record.To.clone();

            let mut properties = HashMap::new();
            properties.insert("Distance_km", record.Distance_km.clone().to_string());
            properties.insert("Travel_time_min", record.Travel_time_min.clone().to_string());
            properties.insert("Congestion_level", record.Congestion_level.clone().to_string());

            // URL para adicionar um nó
            let node_url = format!("{}/graphs/{}/edges", base_url, graph_name);
            
            // Criação de uma task assíncrona para adicionar o nó
            task::spawn(async move {
                // Serializando o HashMap de propriedades para um JSON adequado
                let res = client
                    .post(node_url)
                    .json(&serde_json::json!({
                        "edege_id": edge_id,
                        "label": label,
                        "from": from,
                        "to": to,
                        "properties": properties
                    }))
                    .send()
                    .await;

                match res {
                    Ok(response) if response.status().is_success() => {
                        // println!("Node added (ID: {}, Label: {}): {:?}", node_id, label, response.status());
                    }
                    Ok(response) => {
                        eprintln!("Failed to add node (ID: {}, Label: {}): {:?}", edge_id, label, response.status());
                    }
                    Err(e) => {
                        eprintln!("Failed to add node (ID: {}, Label: {}): {:?}", edge_id, label, e);
                    }
                }
            })
        }).collect();

        // Aguarda todas as tasks terminarem
        for task in tasks {
            let _ = task.await; // Espera cada tarefa terminar
        }

        let finish = Instant::now();
        println!("Tempo de execução Edges: {:?}\nQuantidade de Edges: {}\n------------------------------------", finish.duration_since(start), data.len());

        Ok(())
    }
}

