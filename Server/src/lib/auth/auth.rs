use std::collections::HashMap;

pub struct AuthManager {
  users: HashMap<String, String>, // username -> password
}

impl AuthManager {
  pub fn new() -> Self {
    let mut users = HashMap::new();
    users.insert("admin".to_string(), "admin".to_string()); // Default user
    Self { users }
  }

  pub fn authenticate(&self, username: &str, password: &str) -> bool {
    match self.users.get(username) {
      Some(stored_password) => stored_password == password,
      None => false,
    }
  }

  // Add methods for adding/removing users if needed
}
