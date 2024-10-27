use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant,
};
use tokio::task;
use crate::lib::data::{reader_edges::CSVReaderEdge, reader_nodes::CSVReaderNode};

#[derive(Clone)]
pub struct GraphService {
    pub client: Arc<Client>,
    pub base_url: String,
    pub graph_name: String,
    pub data_nodes: Arc<CSVReaderNode>,
    pub data_edges: Arc<CSVReaderEdge>,
    pub nodes_id: Arc<Mutex<HashMap<usize, usize>>>
}

impl GraphService {
    // Função principal que cria o grafo e adiciona os nós
    pub async fn post_graph(&self) -> Result<String, Box<dyn std::error::Error>> {
        let start = Instant::now();
        let mut text_log = String::new();

        // Cria o grafo
        self.create_graph().await?;
        text_log.push_str("Graph created successfully.\n------------------------------------\n");

        // Adiciona os nós
        text_log.push_str(&self.add_nodes_concurrently().await?);

        // Adiciona as arestas
        text_log.push_str(&self.add_edges_concurrently().await?);

        let finish = Instant::now();
        let execution_time = format!("Tempo de execução total: {:.2?}\n------------------------------------", finish.duration_since(start));
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
        let nodes_id = Arc::clone(&self.nodes_id); // Usando Arc e Mutex para acesso seguro

        let start = Instant::now();
        let mut nodes_log = String::new();

        // Processa cada registro em threads paralelas
        let tasks: Vec<_> = data.iter().enumerate().map(|(_, record)| {
            let client = client.clone();
            let base_url = base_url.clone();
            let label = record.Node.clone();
            let id_node = record.Node_ID; // Supondo que este seja do tipo usize
            let nodes_id = Arc::clone(&nodes_id);

            let node_url = format!("{}/graphs/{}/nodes", base_url, graph_name);

            task::spawn(async move {
                let res = client
                    .post(&node_url)
                    .json(&serde_json::json!({
                        "label": label
                    }))
                    .send()
                    .await;

                match res {
                    Ok(response) => {
                        if response.status().is_success() {
                            // Parse o JSON para obter o id
                            let json: serde_json::Value = response.json().await.unwrap();
                            if let Some(id) = json.get("id") {
                                // Insere a label e o id no HashMap
                                let mut id_map = nodes_id.lock().unwrap(); // Bloqueia para acesso seguro
                                if let Some(id_value) = id.as_u64() {
                                    id_map.insert(id_node, id_value as usize); // Insere no dicionário
                                }
                            }
                        } else {
                            eprintln!("Failed to add node (Label: {}): {:?}", label, response.status());
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to add node (Label: {}): {:?}", label, e);
                    }
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
        let nodes_id = Arc::clone(&self.nodes_id); // Clonando o Arc para acesso seguro
    
        let start = Instant::now();
        let mut edges_log = String::new();
    
        let tasks: Vec<_> = data.iter().enumerate().map(|(index, record)| {
            let client = client.clone();
            let base_url = base_url.clone();
            let label = record.Street.clone();
            let from = record.From.clone();
            let to = record.To.clone();
            let nodes_id = Arc::clone(&nodes_id); // Clone do Arc para cada task
    
            let mut properties = HashMap::new();
            properties.insert("Distance_km", record.Distance_km.clone().to_string());
            properties.insert("Travel_time_min", record.Travel_time_min.clone().to_string());
            properties.insert("Congestion_level", record.Congestion_level.clone().to_string());
    
            let edge_url = format!("{}/graphs/{}/edges", base_url, graph_name);
    
            task::spawn(async move {
                // Bloqueia o mutex para acessar o HashMap de nodes_id
                let from_id: Option<usize>;
                let to_id: Option<usize>;
    
                {
                    let id_map = nodes_id.lock().unwrap(); // Bloqueia o mutex
                    from_id = id_map.get(&from).cloned(); // Busca o ID de 'from'
                    to_id = id_map.get(&to).cloned(); // Busca o ID de 'to'
                } // O mutex é liberado aqui
    
                // Verifica se ambos os IDs foram encontrados
                if let (Some(from_id), Some(to_id)) = (from_id, to_id) {
                    let res = client
                        .post(&edge_url)
                        .json(&serde_json::json!({
                            "label": label,
                            "from": from_id,
                            "to": to_id,
                            "properties": properties
                        }))
                        .send()
                        .await;
    
                    if let Err(e) = res {
                        eprintln!("Failed to add edge (ID: {}): {:?}", index, e);
                    }
                } else {
                    eprintln!("Failed to find node IDs for from: {}, to: {}", from, to);
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

