use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use tokio::task;
use crate::lib::data::reader::{CSVReader, RoadData};
use std::collections::HashMap;
use std::{collections::HashSet, sync::{Arc, Mutex}, time::Instant};
use std::hash::{Hash, Hasher};
use std::hash::Hasher as StdHasher;

fn combine_ids(id: usize, id_to: usize) -> u64 {
    // Combina os IDs em uma string
    let combined_id = format!("{}-{}", id, id_to);
    
    // Gera o hash a partir da string combinada e retorna como u64
    generate_hash(&combined_id)
}

fn generate_hash<T: Hash>(value: T) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[derive(Clone)]
pub struct GraphService {
    pub client: Arc<Client>,
    pub base_url: String,
    pub graph_name: String,
    pub data: Arc<Mutex<CSVReader>>, 
}

impl GraphService {
    // Função principal que cria o grafo e adiciona os nós
    pub async fn post_graph(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Cria o grafo
        self.create_graph().await?;

        // Adiciona os nós
        self.add_nodes_concurrently().await?;

        self.add_edges_concurrently().await?;
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
        let data = self.data.clone(); // data é Arc<Mutex<CSVReader>>
    
        let start = Instant::now();
    
        // Trava o Mutex uma vez e obtém a referência para os dados
        let locked_data = data.lock().unwrap();
    
        // Processa cada registro em threads paralelas, enviando node_id e label (Street)
        let tasks: Vec<_> = locked_data.iter().enumerate().map(|(index, record)| {
            let client = client.clone();
            let base_url = base_url.clone();
            let node_id = index;  // node_id é o índice do registro
            let label = record.street.clone();  // label é o valor de Street  
            
            // Construindo o HashMap para propriedades
            let mut properties = HashMap::new();
            properties.insert("Distance_km", record.distance_km.clone().to_string());
            properties.insert("Travel_time_min", record.travel_time_min.clone().to_string());
            properties.insert("Congestion_level", record.congestion_level.clone().to_string());
            
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
    

    pub async fn add_edges_concurrently(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.client.clone();
        let base_url = self.base_url.clone();
        let graph_name = self.graph_name.clone();
        let data = Arc::clone(&self.data);
    
        let start = Instant::now();
        let locked_data = data.lock().expect("Failed to acquire lock");
        let records = locked_data.get_records();
    
        // Usando um HashSet para evitar duplicatas
        let mut processed_edges = HashSet::new();
    
        let tasks: Vec<_> = records.iter().enumerate().flat_map(|(index, record)| {
            let client = client.clone();
            let base_url = base_url.clone();
            let edge_id = index;
            let from = record.from.clone();
            let to = record.to.clone();
            
            records.iter().enumerate().filter_map(move |(edge_id_to, record_to)| {
                let from_to = record_to.from.clone();
                let to_to = record_to.to.clone();
    
                if edge_id_to != edge_id && (from == to_to || to == from_to) {
                    let (from_json, to_json) = if from == to_to {
                        (edge_id, edge_id_to)
                    } else {
                        (edge_id_to, edge_id)
                    };
    
                    let id = combine_ids(edge_id, edge_id_to);
    
                    // Evita arestas duplicadas
                    if processed_edges.insert(id) {
                        let edges_url = format!("{}/graphs/{}/edges", base_url, graph_name);
    
                        // Criação de uma task assíncrona para adicionar a aresta
                        let task = task::spawn(async move {
                            let res = client.post(&edges_url)
                                .json(&serde_json::json!({
                                    "edge_id": id,
                                    "label": "Connect To...",
                                    "from": from_json,
                                    "to": to_json
                                }))
                                .send()
                                .await;
    
                            match res {
                                Ok(response) if response.status().is_success() => {
                                    println!("Edge added (ID: {}): {:?}", id, response.status());
                                }
                                Ok(response) => {
                                    eprintln!("Failed to add edge (from: {}, to: {}): {:?}", edge_id, edge_id_to, response.text().await.unwrap_or_default());
                                }
                                Err(e) => {
                                    eprintln!("Failed to add edge (from: {}, to: {}): {:?}", edge_id, edge_id_to, e);
                                }
                            }
                        });
                        Some(task)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }).collect::<Vec<_>>() // Coleta as tasks geradas
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