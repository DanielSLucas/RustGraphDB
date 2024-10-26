use std::sync::Arc;

use actix_web::web::scope;
use actix_web::{web, App, HttpServer};

use super::handlers;
use crate::lib::services::graph_service::GraphService;
use crate::lib::storage::StorageManager;

pub async fn run_server(storage_manager: Arc<StorageManager>) -> std::io::Result<()> {
  let graph_service = Arc::new(GraphService::new(storage_manager));

  HttpServer::new(move || {
    App::new()
      .app_data(web::Data::new(Arc::clone(&graph_service)))
      .service(
        scope("/graphs")
          .service(handlers::list_graphs)
          .service(handlers::create_graph)
          .service(handlers::add_node)
          .service(handlers::add_edge)
          .service(handlers::get_graph_adjacency)
          .service(handlers::get_graph_relations)
          .service(handlers::graph_search),
      )
  })
  .bind("localhost:8080")?
  .run()
  .await
}
