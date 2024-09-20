use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use crate::lib::{graph::Graph, storage::StorageManager};
use std::sync::{Arc, Mutex};

async fn list_graphs(storage_manager: web::Data<Arc<Mutex<StorageManager>>>) -> impl Responder {
  let storage_manager = storage_manager.lock().unwrap();
  let graphs = storage_manager.get_graph_names();
  HttpResponse::Ok().json(graphs)
}

async fn create_graph(
  storage_manager: web::Data<Arc<Mutex<StorageManager>>>,
  info: web::Path<String>,
) -> impl Responder {
  let graph_name = info.into_inner();
  let mut storage_manager = storage_manager.lock().unwrap();

  if storage_manager.get_graph(&graph_name).is_some() {
    return HttpResponse::BadRequest().body("Graph already exists.");
  }

  let graph = Graph::new(graph_name.clone());
  storage_manager.add_graph(graph);
  HttpResponse::Ok().body(format!("Graph '{}' created.", graph_name))
}

pub async fn run_server(storage_manager: Arc<Mutex<StorageManager>>) -> std::io::Result<()> {
  HttpServer::new(move || {
    App::new()
      .app_data(web::Data::new(storage_manager.clone()))
      .route("/graphs", web::get().to(list_graphs))
      .route("/graphs/{name}", web::post().to(create_graph))
      // Add more routes as needed
  })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
