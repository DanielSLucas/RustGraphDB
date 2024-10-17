use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use std::error::Error;
use tokio::task;

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

        // Processa cada registro em threads paralelas, enviando node_id e label (Street)
        let tasks: Vec<_> = data.into_iter().enumerate().map(|(index, record)| {
            let client = client.clone();
            let base_url = base_url.clone();
            let node_id = index;  // node_id é o índice do registro
            let label = record.Street.clone();  // label é o valor de Street

            /*let graph_name = "some_graph";              

            let node_url = format!("{}/graphs", base_url);
            let res = client.post(&node_url)
                .json(&graph_name)
                .send();*/

            task::spawn(async move {
                // Envia dados para "/graphs/{graph_name}/nodes" com node_id e label
                let graph_name = "some_graph";  // Substitua conforme necessário
                let node_data = serde_json::json!({
                    "node_id": node_id,
                    "label": label,
                    "properties": {
                        "len": "52" // Propriedade padrão
                    }
                });
                
                println!("Node data to send: {}", node_data);                

                let node_url = format!("{}/graphs/{}/nodes", base_url, graph_name);
                let res = client.post(&node_url)
                    .json(&node_data)
                    .send()
                    .await;

                match res {
                    Ok(response) => {
                        println!("Node added (ID: {}, Label: {}): {:?}", node_id, label, response.status());
                    }
                    Err(e) => {
                        eprintln!("Failed to add node: {}", e);
                    }
                }

                // Outros endpoints como edges, adjacency, relations podem seguir o mesmo padrão
            })
        }).collect();

        // Aguarda todas as threads terminarem
        for task in tasks {
            let _ = task.await;
        }
    }
}

