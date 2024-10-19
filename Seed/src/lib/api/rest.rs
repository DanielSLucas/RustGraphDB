use reqwest::Client;
use serde::Deserialize;
use std::{fmt::format, sync::Arc};
use std::error::Error;
use tokio::task;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::lib::data::reader::RoadData;

#[derive(Clone)]
pub struct GraphService {
    pub client: Arc<Client>,
    pub base_url: String,
}

impl GraphService {
    pub async fn send_to_endpoints_concurrently(&self, data: Vec<RoadData>) {
        let client = self.client.clone();
        let base_url = self.base_url.clone();

        let start = Instant::now();
        let time_now = SystemTime::now();
        // Convertendo para segundos desde a Unix Epoch
        let since_the_epoch = time_now.duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        // Nome do grafo a ser criado
        let graph_name = format!("some_graph_{}", since_the_epoch);

        // URL para criar o grafo
        let graph_url = format!("{}/graphs", base_url);
        
        // Criar o grafo antes de adicionar nós
        let graph_data = serde_json::json!({ "name": graph_name });
        let create_graph_res = client.post(&graph_url)
            .json(&graph_data)
            .send()
            .await;

        match create_graph_res {
            Ok(response) if response.status().is_success() => {
                //println!("Graph '{}' created successfully.", graph_name);
            }
            Ok(response) => {
                //eprintln!("Failed to create graph: {:?}", response.status());
                return; // Retorna se a criação do grafo falhar
            }
            Err(e) => {
                //eprintln!("Error creating graph: {}", e);
                return; // Retorna se houver um erro na requisição
            }
        }

        // Processa cada registro em threads paralelas, enviando node_id e label (Street)
        let tasks: Vec<_> = data.into_iter().enumerate().map(|(index, record)| {
            let client = client.clone();
            let base_url = base_url.clone();
            let node_id = index;  // node_id é o índice do registro
            let label = record.Street.clone();  // label é o valor de Street

            // URL para adicionar um nó
            let node_url = format!("{}/graphs/{}/nodes", base_url, graph_name);

            // Criação de uma task assíncrona para adicionar o nó
            task::spawn(async move {
                let node_data = serde_json::json!({
                    "node_id": node_id,
                    "label": label
                });

                //println!("Node data to send: {}", node_data);

                let res = client.post(&node_url)
                    .json(&node_data)
                    .send()
                    .await;

                match res {
                    Ok(response) if response.status().is_success() => {
                        println!("Node added (ID: {}, Label: {}): {:?}", node_id, label, response.status());
                    }
                    Ok(response) => {
                        eprintln!("Failed to add node (ID: {}, Label: {}): {:?}", node_id, label, response.status());
                    }
                    Err(e) => {
                        eprintln!("Failed to add node (ID: {}, Label: {}): {}", node_id, label, e);
                    }
                }
            })
        }).collect();

        // Aguarda todas as tasks terminarem
        for task in tasks {
            let _ = task.await;
        }

        let finish = Instant::now();

        println!("Tempo de execução em ms: {:?}", finish.duration_since(start));
    }
}
