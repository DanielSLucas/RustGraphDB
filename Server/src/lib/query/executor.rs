use crate::lib::errors::graph_error::GraphError;
use crate::lib::graph::node::Node;
use crate::lib::graph::edge::Edge;
use crate::lib::services::graph_service::GraphService;
use crate::lib::query::parser::{Query, Operation, WhereClause, Condition, Value as QueryValue, Direction};
use std::sync::Arc;
use serde_json::{Value as JsonValue};
use std::collections::HashMap;

pub struct Executor {
    graph_service: Arc<GraphService>,
}

impl Executor {
    pub fn new(graph_service: Arc<GraphService>) -> Self {
        Self { graph_service }
    }

    pub async fn execute(&self, query: &Query) -> Result<JsonValue, GraphError> {
        match (&query.operation, &query.from_clause) {
            (Some(Operation::Match), Some(from_type)) => {
                match from_type.as_str() {
                    "edges" => self.execute_edge_match(query).await,
                    "nodes" => self.execute_node_match(query).await,
                    _ => Err(GraphError::UnsupportedOperation("Unsupported entity type".to_string())),
                }
            },
            (Some(op), _) => Err(GraphError::UnsupportedOperation(
                format!("Unsupported operation: {:?}", op)
            )),
            _ => Err(GraphError::InvalidQuery("Missing operation".to_string())),
        }
    }

    async fn execute_edge_match(&self, query: &Query) -> Result<JsonValue, GraphError> {
        let graph_pattern = query.graph_pattern.as_ref()
            .ok_or_else(|| GraphError::InvalidQuery("Missing graph pattern".into()))?;
        
        // Get edges from the specified graph
        let mut edges = self.graph_service.list_edges(graph_pattern.graph_name.clone()).await?;
        
        // Filter by direction if specified
        match graph_pattern.direction {
            Direction::Outgoing => {
                edges.retain(|edge| edge.from < edge.to);
            },
            Direction::Incoming => {
                edges.retain(|edge| edge.from > edge.to);
            },
            Direction::Bidirectional => {}
        }

        // Apply filters based on WHERE clause
        let filtered_edges = if let Some(where_clause) = &query.where_clause {
            self.apply_where_filter_edges(edges, where_clause)?
        } else {
            edges
        };

        // Format result based on RETURN clause
        self.format_edge_results(&filtered_edges, query.return_clause.as_deref())
    }

    async fn execute_node_match(&self, query: &Query) -> Result<JsonValue, GraphError> {
        let graph_pattern = query.graph_pattern.as_ref()
            .ok_or_else(|| GraphError::InvalidQuery("Missing graph pattern".into()))?;
        
        // Get nodes from the specified graph
        let nodes = self.graph_service.list_nodes(graph_pattern.graph_name.clone()).await?;
        
        // Apply filters based on WHERE clause
        let filtered_nodes = if let Some(where_clause) = &query.where_clause {
            self.apply_where_filter_nodes(nodes, where_clause)?
        } else {
            nodes
        };

        // Format result based on RETURN clause
        self.format_node_results(&filtered_nodes, query.return_clause.as_deref())
    }

    fn apply_where_filter_edges(&self, edges: Vec<Edge>, where_clause: &WhereClause) -> Result<Vec<Edge>, GraphError> {
        Ok(edges.into_iter()
            .filter(|edge| {
                where_clause.conditions.iter().all(|condition| {
                    self.check_property_condition(&edge.properties, condition)
                })
            })
            .collect())
    }

    fn apply_where_filter_nodes(&self, nodes: Vec<Node>, where_clause: &WhereClause) -> Result<Vec<Node>, GraphError> {
        Ok(nodes.into_iter()
            .filter(|node| {
                where_clause.conditions.iter().all(|condition| {
                    self.check_property_condition(&node.properties, condition)
                })
            })
            .collect())
    }

    fn check_property_condition(&self, properties: &HashMap<String, String>, condition: &Condition) -> bool {
        if let Some(prop_value) = properties.get(&condition.field) {
            match (condition.operator.as_str(), &condition.value) {
                ("=", QueryValue::String(val)) => prop_value == val,
                ("=", QueryValue::Number(val)) => {
                    if let Ok(pv) = prop_value.parse::<f64>() {
                        (pv - val).abs() < f64::EPSILON
                    } else {
                        false
                    }
                },
                ("=", QueryValue::Boolean(val)) => {
                    if let Ok(pv) = prop_value.parse::<bool>() {
                        pv == *val
                    } else {
                        false
                    }
                },
                ("LIKE", QueryValue::String(val)) => prop_value.contains(val),
                (">", QueryValue::Number(val)) => {
                    if let Ok(pv) = prop_value.parse::<f64>() {
                        pv > *val
                    } else {
                        false
                    }
                },
                ("<", QueryValue::Number(val)) => {
                    if let Ok(pv) = prop_value.parse::<f64>() {
                        pv < *val
                    } else {
                        false
                    }
                },
                _ => false
            }
        } else {
            false
        }
    }

    fn format_edge_results(&self, edges: &[Edge], return_clause: Option<&str>) -> Result<JsonValue, GraphError> {
        match return_clause {
            Some("e") | Some("edge") => Ok(serde_json::to_value(edges)?),
            Some(return_expr) => {
                // Handle property access expressions (e.g., "e.property")
                if return_expr.starts_with("e.") {
                    let property = return_expr.split('.').nth(1)
                        .ok_or_else(|| GraphError::InvalidQuery("Invalid property access".into()))?;
                    
                    let values: Vec<Option<&String>> = edges.iter()
                        .map(|edge| edge.properties.get(property))
                        .collect();
                    
                    Ok(serde_json::to_value(values)?)
                } else {
                    Err(GraphError::InvalidQuery(format!("Unsupported return expression: {}", return_expr)))
                }
            }
            None => Err(GraphError::InvalidQuery("Missing RETURN clause".into())),
        }
    }

    fn format_node_results(&self, nodes: &[Node], return_clause: Option<&str>) -> Result<JsonValue, GraphError> {
        match return_clause {
            Some("n") | Some("node") => Ok(serde_json::to_value(nodes)?),
            Some(return_expr) => {
                // Handle property access expressions (e.g., "n.property")
                if return_expr.starts_with("n.") {
                    let property = return_expr.split('.').nth(1)
                        .ok_or_else(|| GraphError::InvalidQuery("Invalid property access".into()))?;
                    
                    let values: Vec<Option<&String>> = nodes.iter()
                        .map(|node| node.properties.get(property))
                        .collect();
                    
                    Ok(serde_json::to_value(values)?)
                } else {
                    Err(GraphError::InvalidQuery(format!("Unsupported return expression: {}", return_expr)))
                }
            }
            None => Err(GraphError::InvalidQuery("Missing RETURN clause".into())),
        }
    }
}

impl GraphService {
    pub async fn execute_query(&self, query: &Query) -> Result<JsonValue, GraphError> {
        let executor = Executor::new(Arc::new(self.clone()));
        executor.execute(query).await
    }
}