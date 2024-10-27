use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{sync::{Arc, Mutex}, time::Instant};
use tokio::task;
use rand::Rng; // Import the rand crate

#[derive(Clone)]
pub struct SearchServer {
    pub client: Arc<Client>,
    pub base_url: String,
    pub graph_name: String,
    pub max_edges: usize,
    pub num_search: usize,
}

impl SearchServer {
    pub async fn search(&self) -> Result<String, Box<dyn std::error::Error>> {
        let client = self.client.clone();
        let base_url = self.base_url.clone();
        let graph_name = self.graph_name.clone();
        
        // Use Arc<Mutex<...>> to allow shared mutable access to counters
        let search_fail = Arc::new(Mutex::new(0));
        let search_ok = Arc::new(Mutex::new(0));

        let start = Instant::now();

        // Cria um gerador de números aleatórios
        let mut rng = rand::thread_rng();

        // Define um array com os tipos de busca
        let search_types = ["bfs", "dfs"];

        let tasks: Vec<_> = (0..self.num_search).map(|_| {
            let client = client.clone();
            let origin = rng.gen_range(0..=self.max_edges);
            let goal = rng.gen_range(0..=self.max_edges);

            // Escolhe aleatoriamente entre "bfs" e "dfs"
            let search_type = search_types[rng.gen_range(0..search_types.len())];
            let url = format!("{}/graphs/{}/{}", base_url, graph_name, search_type);

            let search_fail = Arc::clone(&search_fail);
            let search_ok = Arc::clone(&search_ok);

            task::spawn(async move {
                // Serializando o HashMap de propriedades para um JSON adequado
                let res = client
                    .post(&url)
                    .json(&serde_json::json!( {
                        "origin": origin,
                        "goal": goal
                    }))
                    .send()
                    .await;

                match res {
                    Ok(response) if response.status().is_success() => {
                        // Incrementa o contador de buscas bem-sucedidas
                        let mut ok_count = search_ok.lock().unwrap();
                        *ok_count += 1;
                        // println!("Search successfully: {:?} | Origin: {} to Goal: {}", response.status(), origin, goal);
                    }
                    Ok(response) => {
                        eprintln!("Failed to search {:?} | Origin: {} to Goal: {}", response.status(), origin, goal);
                        // Incrementa o contador de buscas falhas
                        let mut fail_count = search_fail.lock().unwrap();
                        *fail_count += 1;
                    }
                    Err(e) => {
                        eprintln!("Failed to search {:?} | Origin: {} to Goal: {}", e, origin, goal);
                        // Incrementa o contador de buscas falhas
                        let mut fail_count = search_fail.lock().unwrap();
                        *fail_count += 1;
                    }
                }
            })
        }).collect();

        // Aguarda todas as tasks terminarem
        for task in tasks {
            let _ = task.await; 
        }

        let finish = Instant::now();
        let text_log = format!("Tempo de execução: {:?}\nQuantidade de Buscas Feitas: {}\nSearch Fail: {}\nSearch OK: {}\n------------------------------------", 
        finish.duration_since(start), 
        self.num_search,
        *search_fail.lock().unwrap(), 
        *search_ok.lock().unwrap()
        );

        println!("{}", text_log);

        Ok((text_log))
    }
}
