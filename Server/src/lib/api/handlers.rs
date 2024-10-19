use crate::lib::services::graph_service::{GraphError, GraphService};
use crate::lib::utils::logger::{log_error, log_info};
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[get("")]
async fn list_graphs(graph_service: web::Data<GraphService>) -> impl Responder {
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

#[derive(Deserialize)]
struct CreateGraphRequest {
  name: String,
}

#[post("")]
async fn create_graph(
  graph_service: web::Data<GraphService>,
  request: web::Json<CreateGraphRequest>,
) -> impl Responder {
  let graph_name = request.name.clone();

  match graph_service.create_graph(graph_name.clone()).await {
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

#[derive(Deserialize)]
struct AddNodeRequest {
  node_id: usize,
  label: String,
  properties: Option<HashMap<String, String>>,
}

#[post("/{graph_name}/nodes")]
async fn add_node(
    graph_service: web::Data<GraphService>,
    path: web::Path<String>,
    request: web::Json<AddNodeRequest>,
) -> impl Responder {    
    let graph_name = path.clone();
    let node_id = request.node_id;
    let label = request.label.clone();
    let properties: HashMap<String, String> =
        request.properties.clone().unwrap_or_else(|| HashMap::new());

    match graph_service
        .add_node(graph_name.clone(), node_id, label, properties)
        .await
    {
        Ok(_) => {
            log_info(&format!(
                "Node {} added to graph '{}' via REST API.",
                node_id, graph_name
            ));
            HttpResponse::Ok().body(format!("Node {} added to graph '{}'.", node_id, graph_name))
        }
        Err(GraphError::NodeAlreadyExists(_)) => {
            log_error(&format!(
                "Node with ID {} already exists in graph '{}'.",
                node_id, graph_name
            ));
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


#[derive(Deserialize)]
struct AddEdgeRequest {
  edge_id: usize,
  from: usize,
  to: usize,
  label: String,
  properties: Option<HashMap<String, String>>,
}

#[post("/{graph_name}/edges")]
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
  let properties: HashMap<String, String> =
    request.properties.clone().unwrap_or_else(|| HashMap::new());

  match graph_service
    .add_edge(graph_name.clone(), edge_id, from, to, label, properties)
    .await
  {
    Ok(_) => {
      log_info(&format!(
        "Edge {} added to graph '{}' via REST API.",
        edge_id, graph_name
      ));
      HttpResponse::Ok().body(format!("Edge {} added to graph '{}'.", edge_id, graph_name))
    }
    Err(GraphError::EdgeAlreadyExists(_)) => {
      log_error(&format!(
        "Edge with ID {} already exists in graph '{}'.",
        edge_id, graph_name
      ));
      HttpResponse::BadRequest().body("Edge already exists.")
    }
    Err(GraphError::NodeNotFound(id)) => {
      log_error(&format!(
        "Node with ID {} does not exist in graph '{}'.",
        id, graph_name
      ));
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

#[derive(serde::Serialize)]
struct GraphAdjacency {
  adjacency_list: HashMap<usize, Vec<usize>>,
}

#[get("/{graph_name}/adjacency")]
async fn get_graph_adjacency(
  graph_service: web::Data<GraphService>,
  graph_name: web::Path<String>,
) -> impl Responder {
  match graph_service.get_graph_adjacency(graph_name.clone()).await {
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
      log_error(&format!(
        "Error retrieving graph adjacency for '{}': {:?}",
        graph_name, e
      ));
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

#[get("/{graph_name}/relations")]
async fn get_graph_relations(
  graph_service: web::Data<GraphService>,
  graph_name: web::Path<String>,
) -> impl Responder {
  match graph_service.get_graph_relations(graph_name.clone()).await {
    Ok(relations) => {
      log_info(&format!(
        "Retrieved relations for graph '{}' via REST API.",
        graph_name
      ));

      let response: Vec<GraphRelation> = relations
        .into_iter()
        .map(
          |(from_id, from_label, edge_label, to_id, to_label)| GraphRelation {
            from_node_id: from_id,
            from_node_label: from_label,
            edge_label,
            to_node_id: to_id,
            to_node_label: to_label,
          },
        )
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

#[derive(serde::Serialize)]
struct GraphPath {
  path: Vec<usize>,
}

#[derive(serde::Deserialize)]
struct GraphSearchQueryParams {
  origin: usize,
  goal: usize,
}

#[get("/{graph_name}/{search_method}")]
async fn graph_search(
  graph_service: web::Data<GraphService>,
  query: web::Query<GraphSearchQueryParams>,
  path: web::Path<(String, String)>,
) -> impl Responder {
  let (graph_name, search_method) = path.into_inner();
  let origin = query.origin;
  let goal = query.goal;

  let result = graph_service
    .search_path(graph_name.clone(), search_method.clone(), origin, goal)
    .await;

  match result {
    Ok(path) => {
      log_info(&format!(
        "Path from {} to {} using {} in graph '{}' retrieved via REST API.",
        origin, goal, search_method, graph_name
      ));
      HttpResponse::Ok().json(GraphPath { path })
    }

    Err(GraphError::GraphNotFound(_)) => {
      log_error(&format!("Graph '{}' not found.", graph_name));
      HttpResponse::BadRequest().body("Graph not found.")
    }

    Err(GraphError::NodeNotFound(node_id)) => {
      log_error(&format!(
        "Node '{}' not found in graph '{}'.",
        node_id, graph_name
      ));
      HttpResponse::BadRequest().body("Node not found.")
    }

    Err(e) => {
      log_error(&format!("Error retrieving path: {:?}", e));
      HttpResponse::InternalServerError().body("Internal Server Error")
    }
  }
}
