use crate::lib::errors::graph_error::GraphError;
use crate::lib::graph::node::Node;
use crate::lib::graph::edge::Edge;
use crate::lib::services::graph_service::GraphService;
use crate::lib::query::parser::Query;
use std::sync::Arc;
use serde_json::Value;
use std::collections::HashMap;

pub struct Executor {
    graph_service: Arc<GraphService>,
}

impl Executor {
    pub fn new(graph_service: Arc<GraphService>) -> Self {
        Self { graph_service }
    }

    pub async fn execute(&self, query: &Query) -> Result<serde_json::Value, GraphError> {
        match query.operation.as_deref() {
            Some("SELECT") => self.execute_select(query).await,
            _ => Err(GraphError::UnsupportedOperation("Only SELECT is supported".into())),
        }
    }

    async fn execute_select(&self, query: &Query) -> Result<serde_json::Value, GraphError> {
        let graph_name = query.graph.as_deref().ok_or(GraphError::InvalidQuery("Graph name is missing".into()))?;
        
        match query.from.as_deref() {
            Some("nodes") => {
                let nodes = self.graph_service.list_nodes(graph_name.to_string()).await?;
                let filtered_nodes = self.filter_and_sort_nodes(nodes, query).await?;
                Ok(serde_json::to_value(filtered_nodes)?)
            }
            Some("edges") => {
                let edges = self.graph_service.list_edges(graph_name.to_string()).await?;
                let filtered_edges = self.filter_and_sort_edges(edges, query).await?;
                Ok(serde_json::to_value(filtered_edges)?)
            }
            _ => Err(GraphError::InvalidQuery("Invalid or missing entity type (nodes or edges)".into())),
        }
    }

    async fn filter_and_sort_nodes(&self, nodes: Vec<Node>, query: &Query) -> Result<Vec<Node>, GraphError> {
        let mut filtered_nodes = nodes;

        // Filtrar os nós com base no `WHERE`
        if let Some(where_clause) = &query.where_clause {
            filtered_nodes = self.apply_where_filter_nodes(filtered_nodes, where_clause).await?;
        }

        // Ordenar os nós com base no `ORDER BY`
        if let Some(order_by) = &query.order_by {
            filtered_nodes = self.apply_order_by_nodes(filtered_nodes, order_by).await?;
        }

        Ok(filtered_nodes)
    }

    async fn apply_where_filter_nodes(&self, nodes: Vec<Node>, where_clause: &str) -> Result<Vec<Node>, GraphError> {
        let conditions = self.parse_conditions(where_clause)?;
    
        let filtered_nodes: Vec<Node> = nodes
            .into_iter()
            .filter(|node| {
                conditions.iter().all(|(attribute, operator, value)| {
                    let (attr, key) = Self::split_attribute_key(attribute);
                    match operator.as_str() {
                        "LIKE" => self.handle_like_filter_nodes(node, &attr, value, key.as_deref()),
                        "=" => self.handle_equal_filter_nodes(node, &attr, value, key.as_deref()),
                        _ => false,
                    }
                })
            })
            .collect();
    
        Ok(filtered_nodes)
    }
    
    fn parse_conditions(&self, where_clause: &str) -> Result<Vec<(String, String, String)>, GraphError> {
        let mut conditions = Vec::new();
        
        let parts: Vec<&str> = where_clause.split_whitespace().collect();
        let mut i = 0;

        while i < parts.len() {
            let attribute = parts[i].to_string();
            let operator = parts[i + 1].to_string();
            let value = parts[i + 2].to_string();
            
            conditions.push((attribute, operator, value));

            // Pular o operador AND/OR
            if i + 3 < parts.len() {
                i += 4; // Pular o próximo operador
            } else {
                i += 3; // Não há operador após o último filtro
            }
        }

        Ok(conditions)
    }

    fn handle_like_filter_nodes(&self, node: &Node, attribute: &str, value: &str, key: Option<&str>) -> bool {
        match attribute {
            "label" => node.label.starts_with(value),
            "id" => node.id.to_string().starts_with(value),
            "category" => node.category.starts_with(value),
            "properties" => {
                if let Some(k) = key {
                    if let Some(prop_value) = node.properties.get(k) {
                        return prop_value.starts_with(value);
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn handle_equal_filter_nodes(&self, node: &Node, attribute: &str, value: &str, key: Option<&str>) -> bool {
        match attribute {
            "label" => node.label == value,
            "id" => node.id.to_string() == value,
            "category" => node.category == value,
            "properties" => {
                if let Some(k) = key {
                    if let Some(prop_value) = node.properties.get(k) {
                        return prop_value == value;
                    }
                }
                false
            }
            _ => false,
        }
    }    

    async fn apply_order_by_nodes(&self, nodes: Vec<Node>, order_by: &str) -> Result<Vec<Node>, GraphError> {
        let mut nodes = nodes;

        if order_by == "label" {
            nodes.sort_by(|a, b| a.label.cmp(&b.label));
        }

        Ok(nodes)
    }

    async fn filter_and_sort_edges(&self, edge: Vec<Edge>, query: &Query) -> Result<Vec<Edge>, GraphError> {
        let mut filtered_edge = edge;

        // Filtrar as arestas com base no `WHERE`
        if let Some(where_clause) = &query.where_clause {
            filtered_edge = self.apply_where_filter_edges(filtered_edge, where_clause).await?;
        }

        // Ordenar as arestas com base no `ORDER BY`
        if let Some(order_by) = &query.order_by {
            filtered_edge = self.apply_order_by_edges(filtered_edge, order_by).await?;
        }

        Ok(filtered_edge)
    }

    async fn apply_where_filter_edges(&self, edge: Vec<Edge>, where_clause: &str) -> Result<Vec<Edge>, GraphError> {
        let conditions = self.parse_conditions(where_clause)?;
    
        let filtered_edge: Vec<Edge> = edge
            .into_iter()
            .filter(|edge| {
                conditions.iter().all(|(attribute, operator, value)| {
                    let (attr, key) = Self::split_attribute_key(attribute);
                    match operator.as_str() {
                        "LIKE" => self.handle_like_filter_edges(edge, &attr, value, key.as_deref()),
                        "=" => self.handle_equal_filter_edges(edge, &attr, value, key.as_deref()),
                        _ => false,
                    }
                })
            })
            .collect();
    
        Ok(filtered_edge)
    }

    fn handle_like_filter_edges(&self, edge: &Edge, attribute: &str, value: &str, key: Option<&str>) -> bool {
        match attribute {
            "label" => edge.label.starts_with(value),
            "id" => edge.id.to_string().starts_with(value),
            "from" => edge.from.to_string().starts_with(value),
            "to" => edge.to.to_string().starts_with(value),
            "properties" => {
                if let Some(k) = key {
                    if let Some(prop_value) = edge.properties.get(k) {
                        return prop_value.starts_with(value);
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn handle_equal_filter_edges(&self, edge: &Edge, attribute: &str, value: &str, key: Option<&str>) -> bool {
        match attribute {
            "label" => edge.label == value,
            "id" => edge.id.to_string() == value,
            "from" => edge.from.to_string() == value,
            "to" => edge.to.to_string() == value,
            "properties" => {
                if let Some(k) = key {
                    if let Some(prop_value) = edge.properties.get(k) {
                        return prop_value == value;
                    }
                }
                false
            }
            _ => false,
        }
    }    

    async fn apply_order_by_edges(&self, edges: Vec<Edge>, order_by: &str) -> Result<Vec<Edge>, GraphError> {
        let mut edges = edges;

        if order_by == "label" {
            edges.sort_by(|a, b| a.label.cmp(&b.label));
        }

        Ok(edges)
    }

    fn split_attribute_key(attribute: &str) -> (String, Option<String>) {
        if let Some((attr, key)) = attribute.split_once('.') {
            (attr.to_string(), Some(key.to_string()))
        } else {
            (attribute.to_string(), None)
        }
    }    
}

impl GraphService {
    pub async fn execute_query(&self, query: &Query) -> Result<serde_json::Value, GraphError> {
        let executor = Executor::new(Arc::new(self.clone()));
        executor.execute(query).await
    }
}