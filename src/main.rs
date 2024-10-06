use std::error::Error;
use std::collections::HashMap;

mod modules;

use modules::graph::Graph;
use modules::persistence::{load_from_file, save_to_file};
use modules::api::run_api;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut graph = Graph::new();

    // Adicionar alguns nós e arestas iniciais
    let id1 = graph.add_node("Daniel".to_string(), HashMap::new());
    let id2 = graph.add_node("Vitor Freire".to_string(), HashMap::new());
    let id3 = graph.add_node("Alice".to_string(), HashMap::new());

    graph.add_edge(id1, id2, "friends with".to_string(), HashMap::new())?;
    graph.add_edge(id1, id3, "colleague of".to_string(), HashMap::new())?;

    // Salvar o grafo em um arquivo
    save_to_file(&graph, "graph.json")?;

    // Carregar o grafo de um arquivo
    let loaded_graph = load_from_file("graph.json")?;

    // Iniciar o servidor da API
    run_api(loaded_graph).await?;

    Ok(())
}


