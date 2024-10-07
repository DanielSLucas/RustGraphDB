pub fn log_info(message: &str) {
  println!("{}", message);
}

pub fn log_error(message: &str) {
  eprintln!("Error: {}", message);
}
