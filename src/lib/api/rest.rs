use actix_web::{web, App, HttpServer, Responder};

async fn get_node() -> impl Responder {
  // Implement API endpoint logic here
}

pub async fn run_server() -> std::io::Result<()> {
  HttpServer::new(|| {
    App::new()
      .route("/node", web::get().to(get_node))
      // Add more routes as needed
  })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
