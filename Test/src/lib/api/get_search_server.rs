use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{sync::{Arc, Mutex}, time::Instant};
use tokio::task;
use rand::Rng; // Import the rand crate
use crate::lib::data::{reader_edges::CSVReaderEdge, reader_nodes::CSVReaderNode};

#[derive(Clone)]
pub struct SearchServer {
    pub client: Arc<Client>,
    pub base_url: String,
    pub graph_name: String,
    pub data: Arc<CSVReaderNode>,
    pub num_search: usize,
    pub edges: Arc<CSVReaderEdge>
}

impl SearchServer {
    pub async fn search(&self) -> Result<String, Box<dyn std::error::Error>> {
        let client = self.client.clone();
        let base_url = self.base_url.clone();
        let graph_name = self.graph_name.clone();

        let search_fail = Arc::new(Mutex::new(0));
        let search_ok = Arc::new(Mutex::new(0));

        let start = Instant::now();
        let mut rng = rand::thread_rng();
        let search_types = ["bfs", "dfs", "dijkstra"];

        let tasks: Vec<_> = (0..self.num_search).map(|_| {
            let client = client.clone();
            let origin = self.data.random_node_id();
            let goal = self.data.random_node_id();
            let search_type = search_types[rng.gen_range(0..search_types.len())];

            // Construindo um objeto de parâmetros
            let mut params: Vec<(&str, String)> = vec![
                ("origin", origin.to_string()),
                ("goal", goal.to_string()),
            ];

            if search_type == "dijkstra" {
                let propriedade = self.edges.random_last_attribute();
                params.push(("property_name", propriedade.to_string())); 
            }

            // Constrói a URL com os parâmetros de consulta
            let url = format!(
                "{}/graphs/{}/{}?{}",
                base_url,
                graph_name,
                search_type,
                serde_urlencoded::to_string(&params).unwrap()
            );

            let search_fail = Arc::clone(&search_fail);
            let search_ok = Arc::clone(&search_ok);

            task::spawn(async move {
                match client.get(&url).send().await {
                    Ok(response) if response.status().is_success() => {
                        let mut ok_count = search_ok.lock().unwrap();
                        *ok_count += 1;
                    }
                    Ok(response) => {
                        //eprintln!("Failed to search {:?} | Origin: {} to Goal: {}", response.status(), origin, goal);
                        let mut fail_count = search_fail.lock().unwrap();
                        *fail_count += 1;
                    }
                    Err(e) => {
                        //eprintln!("Failed to search {:?} | Origin: {} to Goal: {}", e, origin, goal);
                        let mut fail_count = search_fail.lock().unwrap();
                        *fail_count += 1;
                    }
                }
            })
        }).collect();

        for task in tasks {
            let _ = task.await;
        }

        let finish = Instant::now();
        let time_execute = finish.duration_since(start);

        let text_log = format!(
            "Tempo de execução: {:.2?}\nQuantidade de Buscas Feitas: {}\nTempo por Busca: {:.2?} ms\nSearch Fail: {}\nSearch OK: {}\n------------------------------------",
            time_execute,
            self.num_search,
            time_execute.as_millis() as f64 / self.num_search as f64,
            *search_fail.lock().unwrap(),
            *search_ok.lock().unwrap()
        );

        println!("{}", text_log);
        Ok(text_log)
    }
}

