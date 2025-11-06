use reqwest::blocking::get;
use reqwest::Error;
use serde::Deserialize;
use reqwest::Client;
use std::{collections::HashMap, sync::{Arc, Mutex}, time::Instant, string::String};

#[derive(Debug, Deserialize)]
pub struct Relation {
    from_node_id: u32,
    from_node_label: String,
    edge_label: String,
    to_node_id: u32,
    to_node_label: String,
}

#[derive(Debug)]
pub struct Graph {
    nodes: HashMap<u32, String>,
    edges: Vec<(u32, u32, String)>,
    client: Arc<Client>,
    graph_name: String,
    base_url: String
}

impl Graph {
    pub fn new(client: Arc<Client>, graph_name: String, base_url: String) -> Self {
        Graph {
            nodes: HashMap::new(),
            edges: Vec::new(),
            client: client,
            graph_name: graph_name,
            base_url: base_url
        }
    }

    pub async fn get_relation(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let mut text_output = String::new();
    
        let url_relations = format!("{}/graphs/{}/relations", self.base_url, self.graph_name);
        
        text_output.push_str(&format!("Matriz de Relacionamentos do Grafo: {}.\n", self.graph_name));
    
        let start = Instant::now();
    
        let response: Vec<Relation> = self.client.get(&url_relations).send().await?.json().await?;
    
        let finish = Instant::now();
        let time_execute = finish.duration_since(start).as_millis() as f64;
    
        text_output.push_str(&format!("Get de Relações realizado em {:.2?} ms\n", time_execute));
    
        let relations = self.print_relation(&response).await?;
    
        text_output.push_str(&relations);
    
        // Adiciona o tamanho da matriz
        let matrix_size = self.nodes.len();
        text_output.push_str(&format!("\nTamanho da matriz: {} nós.", matrix_size));
    
        text_output.push_str(&format!("\nMatriz de Relacionamentos do Grafo: {} Completa.", self.graph_name));
    
        Ok(text_output)
    }
   

    pub async fn print_relation(&mut self, relations: &[Relation]) -> Result<String, Box<dyn std::error::Error>> {
        let start = Instant::now();

        for relation in relations {
            self.add_relation(&relation);
        }

        let matrix = self.adjacency_matrix();

        let mut text_output = String::new();

        // Converte a matriz de adjacência para uma string formatada
        text_output.push_str(&self.matrix_to_string(&matrix));

        let finish = Instant::now();
        let time_execute = finish.duration_since(start).as_millis() as f64;

        text_output.push_str(&format!("\nRelação Executada em {:.2?} ms", time_execute));

        Ok(text_output)
    }

    pub fn add_relation(&mut self, relation: &Relation) {
        self.nodes.insert(relation.from_node_id, relation.from_node_label.clone());
        self.nodes.insert(relation.to_node_id, relation.to_node_label.clone());
        self.edges.push((
            relation.from_node_id,
            relation.to_node_id,
            relation.edge_label.clone(),
        ));
    }

    pub fn adjacency_matrix(&self) -> Vec<Vec<i32>> {
        let n = self.nodes.len();
        let mut matrix = vec![vec![0; n]; n];

        let node_ids: Vec<&u32> = self.nodes.keys().collect();

        for (from, to, _) in &self.edges {
            let from_index = node_ids.iter().position(|&&id| id == *from).unwrap();
            let to_index = node_ids.iter().position(|&&id| id == *to).unwrap();
            matrix[from_index][to_index] = 1; 
        }

        matrix
    }

    fn matrix_to_string(&self, matrix: &[Vec<i32>]) -> String {
        let formatted_rows: Vec<String> = matrix
            .iter()
            .map(|row| {
                let row_string = row.iter()
                    .map(|&val| val.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                format!("[{}]", row_string) 
            })
            .collect();
    
        formatted_rows.join("\n\n") + "\n"
    }
}