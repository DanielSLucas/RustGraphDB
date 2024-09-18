use graphdb::lib::api::cli::run_cli;
use graphdb::lib::api::rest::run_server;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.contains(&"--cli".to_string()) {
        run_cli();
    } else {
        run_server().await?;
    }

    Ok(())
}
