use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, time::Instant};
use tokio::task;
use crate::lib::data::{reader_edges::CSVReaderEdge, reader_nodes::CSVReaderNode};

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
    pub async fn post_graph(&self) -> Result<String, Box<dyn std::error::Error>> {
        let start = Instant::now();
        let mut text_log = String::new();

        // Cria o grafo
        self.create_graph().await?;
        text_log.push_str("Graph created successfully.\n");

        // Adiciona os nós
        text_log.push_str(&self.add_nodes_concurrently().await?);

        // Adiciona as arestas
        text_log.push_str(&self.add_edges_concurrently().await?);

        let finish = Instant::now();
        let execution_time = format!("Tempo de execução total: {:.2?}\n", finish.duration_since(start));
        text_log.push_str(&execution_time);
        println!("{}", execution_time);

        Ok(text_log)
    }

    // Função para criar o grafo
    pub async fn create_graph(&self) -> Result<(), Box<dyn std::error::Error>> {
        let graph_url = format!("{}/graphs", self.base_url);

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

    pub async fn add_nodes_concurrently(&self) -> Result<String, Box<dyn std::error::Error>> {
        let client = self.client.clone();
        let base_url = self.base_url.clone();
        let graph_name = self.graph_name.clone();
        let data = self.data_nodes.clone();

        let start = Instant::now();
        let mut nodes_log = String::new();

        // Processa cada registro em threads paralelas
        let tasks: Vec<_> = data.iter().enumerate().map(|(_, record)| {
            let client = client.clone();
            let base_url = base_url.clone();
            let node_id = record.Node_ID.clone();
            let label = record.Node.clone();

            let node_url = format!("{}/graphs/{}/nodes", base_url, graph_name);

            task::spawn(async move {
                let res = client
                    .post(&node_url)
                    .json(&serde_json::json!({
                        "node_id": node_id,
                        "label": label
                    }))
                    .send()
                    .await;

                if let Err(e) = res {
                    eprintln!("Failed to add node (ID: {}, Label: {}): {:?}", node_id, label, e);
                }
            })
        }).collect();

        // Aguarda todas as tasks terminarem
        for task in tasks {
            let _ = task.await;
        }

        let finish = Instant::now();
        let time_execute = finish.duration_since(start).as_millis() as f64;

        nodes_log.push_str(&format!(
            "Tempo de execução Nodes: {:.2?} ms\nTempo por Node: {:.2?} ms\nQuantidade de Nodes: {}\n------------------------------------\n",
            time_execute,
            time_execute / data.len() as f64,
            data.len()
        ));
        Ok(nodes_log)
    }

    pub async fn add_edges_concurrently(&self) -> Result<String, Box<dyn std::error::Error>> {
        let client = self.client.clone();
        let base_url = self.base_url.clone();
        let graph_name = self.graph_name.clone();
        let data = self.data_edges.clone();

        let start = Instant::now();
        let mut edges_log = String::new();

        let tasks: Vec<_> = data.iter().enumerate().map(|(index, record)| {
            let client = client.clone();
            let base_url = base_url.clone();
            let label = record.Street.clone();
            let from = record.From.clone();
            let to = record.To.clone();

            let mut properties = HashMap::new();
            properties.insert("Distance_km", record.Distance_km.clone().to_string());
            properties.insert("Travel_time_min", record.Travel_time_min.clone().to_string());
            properties.insert("Congestion_level", record.Congestion_level.clone().to_string());

            let edge_url = format!("{}/graphs/{}/edges", base_url, graph_name);

            task::spawn(async move {
                let res = client
                    .post(&edge_url)
                    .json(&serde_json::json!({
                        "label": label,
                        "from": from,
                        "to": to,
                        "properties": properties
                    }))
                    .send()
                    .await;

                if let Err(e) = res {
                    eprintln!("Failed to add edge (ID: {}): {:?}", index, e);
                }
            })
        }).collect();

        // Aguarda todas as tasks terminarem
        for task in tasks {
            let _ = task.await;
        }

        let finish = Instant::now();
        let time_execute = finish.duration_since(start);
        edges_log.push_str(&format!(
            "Tempo de execução Edges: {:.2?}\nTempo por Edge: {:.2?} ms\nQuantidade de Edges: {}\n------------------------------------\n",
            time_execute,
            time_execute.as_millis() as f64 / data.len() as f64,
            data.len()
        ));
        Ok(edges_log)
    }
}

