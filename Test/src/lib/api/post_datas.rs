use crate::lib::data::{reader_edges::CSVReaderEdge, reader_nodes::CSVReaderNode};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
  time::Instant,
};
use tokio::task;

#[derive(Clone)]
pub struct GraphService {
  pub client: Arc<Client>,
  pub base_url: String,
  pub graph_name: String,
  pub data_nodes: Arc<CSVReaderNode>,
  pub data_edges: Arc<CSVReaderEdge>,
  pub nodes_id: Arc<Mutex<HashMap<usize, usize>>>,
}

impl GraphService {
  // Função principal que cria o grafo e adiciona os nós
  pub async fn post_graph(&mut self) -> Result<String, Box<dyn std::error::Error>> {
    let start = Instant::now();
    let mut text_log = String::new();

    // Cria o grafo
    self.create_graph().await?;
    text_log.push_str("Graph created successfully.\n------------------------------------\n");

    // Adiciona os nós
    text_log.push_str(&self.add_nodes_concurrently().await?);

    // Adiciona as arestas
    text_log.push_str(&self.add_edges_concurrently().await?);

    self.graph_name = format!("{}_1", self.graph_name);
    text_log.push_str("Envio Unico:\n------------------------------------\n");
    self.create_graph().await?;

    // Adiciona os nós
    text_log.push_str(&self.add_all_nodes().await?);

    // Adiciona as arestas
    text_log.push_str(&self.add_all_edges().await?);

    let finish = Instant::now();
    let execution_time = format!(
      "Tempo de execução total: {:.2?}\n------------------------------------",
      finish.duration_since(start)
    );
    text_log.push_str(&execution_time);
    println!("{}", execution_time);

    Ok(text_log)
  }

  // Função para criar o grafo
  pub async fn create_graph(&self) -> Result<(), Box<dyn std::error::Error>> {
    let graph_url = format!("{}/graphs", self.base_url);

    let graph_data = serde_json::json!({ "name": self.graph_name });

    let create_graph_res = self
      .client
      .post(&graph_url)
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
    let tasks: Vec<_> = data
      .iter()
      .enumerate()
      .map(|(_, record)| {
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
                "nodes": [{"label": label, "category": "Intersection", "properties": {}}]
            }))
            .send()
            .await;

          match res {
            Ok(response) => {
              if response.status().is_success() {
                // Parse o JSON para obter o id
                let json: serde_json::Value = response.json().await.unwrap();
                if let Some(node) = json.get("nodes") {
                  // Insere a label e o id no HashMap
                  let mut id_map = nodes_id.lock().unwrap(); // Bloqueia para acesso seguro
                  if let Some(id_value) = node[0]["id"].as_u64() {
                    id_map.insert(id_node, id_value as usize); // Insere no dicionário
                  }
                }
              } else {
                eprintln!(
                  "Failed to add node (Label: {}): {:?}",
                  label,
                  response.status()
                );
              }
            }
            Err(e) => {
              eprintln!("Failed to add node (Label: {}): {:?}", label, e);
            }
          }
        })
      })
      .collect();

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

    let tasks: Vec<_> = data
      .iter()
      .enumerate()
      .map(|(index, record)| {
        let client = client.clone();
        let base_url = base_url.clone();
        let label = record.Street.clone();
        let from = record.From.clone();
        let to = record.To.clone();
        let nodes_id = Arc::clone(&nodes_id); // Clone do Arc para cada task

        let mut properties = HashMap::new();
        properties.insert("Distance_km", record.Distance_km.clone().to_string());
        properties.insert(
          "Travel_time_min",
          record.Travel_time_min.clone().to_string(),
        );
        properties.insert(
          "Congestion_level",
          record.Congestion_level.clone().to_string(),
        );

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
                  "edges": [{"label": label,
                  "from": from_id,
                  "to": to_id,
                  "properties": properties}]
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
      })
      .collect();

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

  pub async fn add_all_nodes(&self) -> Result<String, Box<dyn std::error::Error>> {
    let client = self.client.clone();
    let base_url = self.base_url.clone();
    let graph_name = self.graph_name.clone();
    let data = self.data_nodes.clone();
    let nodes_id = Arc::clone(&self.nodes_id); // Usando Arc e Mutex para acesso seguro

    let start = Instant::now();
    let mut nodes_log = String::new();

    let data_vec: Vec<_> = data.iter().map(|record| record.Node_ID).collect();
    // Acumula todos os nós em uma única requisição
    let nodes_data: Vec<_> = data
      .iter()
      .map(|record| {
        serde_json::json!({
            "label": record.Node,
            "category": "Intersection",
            "properties": {}
        })
      })
      .collect();

    let node_url = format!("{}/graphs/{}/nodes", base_url, graph_name);

    let res = client
      .post(&node_url)
      .json(&serde_json::json!({ "nodes": nodes_data }))
      .send()
      .await;

    let finish = Instant::now();
    let time_execute = finish.duration_since(start).as_millis() as f64;

    if let Ok(response) = res {
      if response.status().is_success() {
        let json: serde_json::Value = response.json().await?;
        if let Some(nodes) = json.get("nodes") {
          let mut id_map = nodes_id.lock().unwrap();
          for (i, node) in nodes.as_array().unwrap().iter().enumerate() {
            if let Some(id_value) = node["id"].as_u64() {
              //println!("{:?}", i);
              let node_id = data_vec[i];
              id_map.insert(node_id, id_value as usize); // Insere no dicionário
            }
          }
        }
      } else {
        eprintln!("Failed to add nodes: {:?}", response.status());
      }
    }

    nodes_log.push_str(&format!(
            "Tempo de execução Envio Unico Nodes: {:.2?} ms\nTempo por Node: {:.2?} ms\nQuantidade de Nodes: {}\n------------------------------------\n",
            time_execute,
            time_execute / data.len() as f64,
            data.len()
        ));

    println!("{}", nodes_log);

    Ok(nodes_log)
  }

  pub async fn add_all_edges(&self) -> Result<String, Box<dyn std::error::Error>> {
    let client = self.client.clone();
    let base_url = self.base_url.clone();
    let graph_name = self.graph_name.clone();
    let data = self.data_edges.clone();
    let nodes_id = Arc::clone(&self.nodes_id); // Clonando o Arc para acesso seguro

    let start = Instant::now();
    let mut edges_log = String::new();

    // Acumula todas as arestas em uma única requisição
    let edges_data: Vec<_> = data
      .iter()
      .filter_map(|record| {
        let from_id;
        let to_id;
        {
          let id_map = nodes_id.lock().unwrap();
          from_id = id_map.get(&record.From).cloned();
          to_id = id_map.get(&record.To).cloned();
        }
        if let (Some(from_id), Some(to_id)) = (from_id, to_id) {
          Some(serde_json::json!({
              "label": record.Street,
              "from": from_id,
              "to": to_id,
              "properties": {
                  "Distance_km": record.Distance_km.clone().to_string(),
                  "Travel_time_min": record.Travel_time_min.clone().to_string(),
                  "Congestion_level": record.Congestion_level.clone().to_string()
              }
          }))
        } else {
          eprintln!(
            "Failed to find node IDs for from: {}, to: {}",
            record.From, record.To
          );
          None
        }
      })
      .collect();

    let edge_url = format!("{}/graphs/{}/edges", base_url, graph_name);

    let res = client
      .post(&edge_url)
      .json(&serde_json::json!({ "edges": edges_data }))
      .send()
      .await;

    if let Err(e) = res {
      eprintln!("Failed to add edges: {:?}", e);
    }

    let finish = Instant::now();
    let time_execute = finish.duration_since(start);
    edges_log.push_str(&format!(
            "Tempo de execução Envio Unico Edges: {:.2?}\nTempo por Edge: {:.2?} ms\nQuantidade de Edges: {}\n------------------------------------\n",
            time_execute,
            time_execute.as_millis() as f64 / data.len() as f64,
            data.len()
        ));

    println!("{}", edges_log);

    Ok(edges_log)
  }
}
