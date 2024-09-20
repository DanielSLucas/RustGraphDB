use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::lib::storage::StorageManager;
use crate::lib::services::graph_service::{GraphService, GraphError};
use crate::lib::utils::logger::{log_info, log_error};

pub async fn run_server(storage_manager: Arc<Mutex<StorageManager>>) -> std::io::Result<()> {
  let graph_service = GraphService::new(storage_manager);

  HttpServer::new(move || {
    App::new()
      .app_data(web::Data::new(graph_service.clone()))
      .route("/graphs", web::get().to(list_graphs))
      .route("/graphs", web::post().to(create_graph))
      .route("/graphs/{graph_name}/nodes", web::post().to(add_node))
      .route("/graphs/{graph_name}/edges", web::post().to(add_edge))
      .route("/graphs/{graph_name}/adjacency", web::get().to(get_graph_adjacency))
      .route("/graphs/{graph_name}/relations", web::get().to(get_graph_relations))
  })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

#[derive(Deserialize)]
struct CreateGraphRequest {
  name: String,
}

#[derive(Deserialize)]
struct AddNodeRequest {
  node_id: usize,
  label: String,
  properties: Option<HashMap<String, String>>,
}

#[derive(Deserialize)]
struct AddEdgeRequest {
  edge_id: usize,
  from: usize,
  to: usize,
  label: String,
  properties: Option<HashMap<String, String>>,
}

async fn list_graphs(
  graph_service: web::Data<GraphService>,
) -> impl Responder {
  match graph_service.list_graphs() {
    Ok(graphs) => {
      log_info("Listed graphs via REST API");
      HttpResponse::Ok().json(graphs)
    }
    Err(e) => {
      log_error(&format!("{:?}", e));
      HttpResponse::InternalServerError().body("Internal Server Error")
    }
  }
}

async fn create_graph(
  graph_service: web::Data<GraphService>,
  request: web::Json<CreateGraphRequest>,
) -> impl Responder {
  let graph_name = request.name.clone();

  match graph_service.create_graph(graph_name.clone()) {
    Ok(_) => {
      log_info(&format!("Graph '{}' created via REST API.", graph_name));
      HttpResponse::Ok().body(format!("Graph '{}' created.", graph_name))
    }
    Err(GraphError::GraphAlreadyExists(_)) => {
      log_error(&format!("Graph '{}' already exists.", graph_name));
      HttpResponse::BadRequest().body("Graph already exists.")
    }
    Err(e) => {
      log_error(&format!("{:?}", e));
      HttpResponse::InternalServerError().body("Internal Server Error")
    }
  }
}

async fn add_node(
  graph_service: web::Data<GraphService>,
  path: web::Path<String>,
  request: web::Json<AddNodeRequest>,
) -> impl Responder {
  let graph_name = path.clone();
  let node_id = request.node_id;
  let label = request.label.clone();
  let properties: HashMap<String, String> = request.properties.clone().unwrap_or_else(|| HashMap::new());  

  match graph_service.add_node(graph_name.clone(), node_id, label, properties) {
    Ok(_) => {
      log_info(&format!("Node {} added to graph '{}' via REST API.", node_id, graph_name));
      HttpResponse::Ok().body(format!("Node {} added to graph '{}'.", node_id, graph_name))
    }
    Err(GraphError::NodeAlreadyExists(_)) => {
      log_error(&format!("Node with ID {} already exists in graph '{}'.", node_id, graph_name));
      HttpResponse::BadRequest().body("Node already exists.")
    }
    Err(GraphError::GraphNotFound(_)) => {
      log_error(&format!("Graph '{}' not found.", graph_name));
      HttpResponse::BadRequest().body("Graph not found.")
    }
    Err(e) => {
      log_error(&format!("{:?}", e));
      HttpResponse::InternalServerError().body("Internal Server Error")
    }
  }
}

async fn add_edge(
  graph_service: web::Data<GraphService>,
  path: web::Path<String>,
  request: web::Json<AddEdgeRequest>,
) -> impl Responder {
  let graph_name = path.clone();
  let edge_id = request.edge_id;
  let from = request.from;
  let to = request.to;
  let label = request.label.clone();
  let properties: HashMap<String, String> = request.properties.clone().unwrap_or_else(|| HashMap::new());  

  match graph_service.add_edge(graph_name.clone(), edge_id, from, to, label, properties) {
    Ok(_) => {
      log_info(&format!("Edge {} added to graph '{}' via REST API.", edge_id, graph_name));
      HttpResponse::Ok().body(format!("Edge {} added to graph '{}'.", edge_id, graph_name))
    }
    Err(GraphError::EdgeAlreadyExists(_)) => {
      log_error(&format!("Edge with ID {} already exists in graph '{}'.", edge_id, graph_name));
      HttpResponse::BadRequest().body("Edge already exists.")
    }
    Err(GraphError::NodeNotFound(id)) => {
      log_error(&format!("Node with ID {} does not exist in graph '{}'.", id, graph_name));
      HttpResponse::BadRequest().body("Node not found.")
    }
    Err(GraphError::GraphNotFound(_)) => {
      log_error(&format!("Graph '{}' not found.", graph_name));
      HttpResponse::BadRequest().body("Graph not found.")
    }
    Err(e) => {
      log_error(&format!("{:?}", e));
      HttpResponse::InternalServerError().body("Internal Server Error")
    }
  }
}

#[derive(Serialize)]
struct GraphAdjacency {
  adjacency_list: HashMap<usize, Vec<usize>>,
}

async fn get_graph_adjacency(
  graph_service: web::Data<GraphService>,
  graph_name: web::Path<String>,
) -> impl Responder {
  match graph_service.get_graph_adjacency(graph_name.clone()) {
    Ok(adjacency_list) => {
      log_info(&format!(
        "Retrieved adjacency list for graph '{}' via REST API.",
        graph_name
      ));
      HttpResponse::Ok().json(GraphAdjacency { adjacency_list })
    }
    Err(GraphError::GraphNotFound(_)) => {
      log_error(&format!("Graph '{}' not found.", graph_name));
      HttpResponse::BadRequest().body("Graph not found.")
    }
    Err(e) => {
      log_error(&format!("{:?}", e));
      HttpResponse::InternalServerError().body("Internal Server Error")
    }
  }
}

#[derive(Serialize)]
struct GraphRelation {
  from_node_id: usize,
  from_node_label: String,
  edge_label: String,
  to_node_id: usize,
  to_node_label: String,
}

async fn get_graph_relations(
  graph_service: web::Data<GraphService>,
  graph_name: web::Path<String>,
) -> impl Responder {
  match graph_service.get_graph_relations(graph_name.clone()) {
    Ok(relations) => {
      log_info(&format!(
        "Retrieved relations for graph '{}' via REST API.",
        graph_name
      ));

      let response: Vec<GraphRelation> = relations
        .into_iter()
        .map(|(from_id, from_label, edge_label, to_id, to_label)| GraphRelation {
          from_node_id: from_id,
          from_node_label: from_label,
          edge_label,
          to_node_id: to_id,
          to_node_label: to_label,
        })
      .collect();

      HttpResponse::Ok().json(response)
    }
    Err(GraphError::GraphNotFound(_)) => {
      log_error(&format!("Graph '{}' not found.", graph_name));
      HttpResponse::BadRequest().body("Graph not found.")
    }
    Err(e) => {
      log_error(&format!("{:?}", e));
      HttpResponse::InternalServerError().body("Internal Server Error")
    }
  }
}
