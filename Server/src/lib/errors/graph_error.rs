use std::fmt;

#[derive(Debug)]
pub enum GraphError {
  GraphNotFound(String),
  GraphAlreadyExists(String),
  NodeNotFound(usize),
  NodeAlreadyExists(usize),
  EdgeNotFound(usize),
  EdgeAlreadyExists(usize),
  InvalidOperation(String),
  StorageError(String),
  MethodNotSupported(String),
  UnsupportedOperation(String),
  InvalidQuery(String),
}

impl std::error::Error for GraphError {}

impl fmt::Display for GraphError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      GraphError::GraphNotFound(name) => write!(f, "Graph '{}' not found.", name),
      GraphError::GraphAlreadyExists(name) => write!(f, "Graph '{}' already exists.", name),
      GraphError::NodeNotFound(id) => write!(f, "Node with ID {} not found.", id),
      GraphError::NodeAlreadyExists(id) => write!(f, "Node with ID {} already exists.", id),
      GraphError::EdgeNotFound(id) => write!(f, "Edge with ID {} not found.", id),
      GraphError::EdgeAlreadyExists(id) => write!(f, "Edge with ID {} already exists.", id),
      GraphError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
      GraphError::StorageError(msg) => write!(f, "Storage error: {}", msg),
      GraphError::MethodNotSupported(msg) => write!(f, "Method {} not supported.", msg),
      GraphError::UnsupportedOperation(msg) => write!(f, "Operation {} not supported.", msg),
      GraphError::InvalidQuery(msg) => write!(f, "Query {} not supported.", msg),
    }
  }
}

impl From<serde_json::Error> for GraphError {
  fn from(error: serde_json::Error) -> Self {
      GraphError::StorageError(format!("JSON serialization/deserialization error: {}", error))
  }
}

