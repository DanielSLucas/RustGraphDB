use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

use super::graph::Graph;

#[derive(Deserialize)]
struct NodeInput {
    label: String,
    properties: HashMap<String, String>,
}

#[derive(Deserialize)]
struct EdgeInput {
    from: usize,
    to: usize,
    label: String,
    properties: HashMap<String, String>,
}

// Adicionar nó ao grafo
async fn add_node(graph_data: web::Data<Mutex<Graph>>, node: web::Json<NodeInput>) -> impl Responder {
    let mut graph = graph_data.lock().unwrap();
    let node_id = graph.add_node(node.label.clone(), node.properties.clone());
    HttpResponse::Ok().json(node_id)
}

// Adicionar aresta ao grafo
async fn add_edge(graph_data: web::Data<Mutex<Graph>>, edge: web::Json<EdgeInput>) -> impl Responder {
    let mut graph = graph_data.lock().unwrap();
    match graph.add_edge(edge.from, edge.to, edge.label.clone(), edge.properties.clone()) {
        Ok(edge_id) => HttpResponse::Ok().json(edge_id),
        Err(err) => HttpResponse::BadRequest().body(err),
    }
}

// Ler todos os nós do grafo
async fn get_nodes(graph_data: web::Data<Mutex<Graph>>) -> impl Responder {
    let graph = graph_data.lock().unwrap();
    let nodes: Vec<_> = graph.nodes().values().collect();
    HttpResponse::Ok().json(nodes)
}

// Ler todas as arestas do grafo
async fn get_edges(graph_data: web::Data<Mutex<Graph>>) -> impl Responder {
    let graph = graph_data.lock().unwrap();
    let edges: Vec<_> = graph.edges().values().collect();
    HttpResponse::Ok().json(edges)
}

// Inicializar a API
pub async fn run_api(graph: Graph) -> std::io::Result<()> {
    let graph_data = web::Data::new(Mutex::new(graph));

    HttpServer::new(move || {
        App::new()
            .app_data(graph_data.clone())
            .route("/add_node", web::post().to(add_node))
            .route("/add_edge", web::post().to(add_edge))
            .route("/get_nodes", web::get().to(get_nodes))
            .route("/get_edges", web::get().to(get_edges))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
