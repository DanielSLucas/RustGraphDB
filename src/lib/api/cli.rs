use std::io::{self, Write};
use crate::lib::graph::Graph;

pub fn run_cli() {
  let mut graph = Graph::new();
  
  loop {
    print!("> ");
    io::stdout().flush().unwrap();

    let mut command = String::new();
    io::stdin().read_line(&mut command).unwrap();

    // Parse and execute the command
  }
}
