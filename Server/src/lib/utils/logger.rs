use std::env;

pub fn log_info(message: &str) {
  let log = env::var("LOG").is_ok();
  if log {
    println!("{}", message);
  }
}

pub fn log_error(message: &str) {
  eprintln!("Error: {}", message);
}
