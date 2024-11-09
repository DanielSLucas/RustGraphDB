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
        
        // Lógica para filtrar e ordenar
        let nodes = self.graph_service.list_nodes(graph_name.to_string()).await?;
        let filtered_nodes = self.filter_and_sort_nodes(nodes, query).await?;

        Ok(serde_json::to_value(filtered_nodes)?)
    }

    async fn filter_and_sort_nodes(&self, nodes: Vec<Node>, query: &Query) -> Result<Vec<Node>, GraphError> {
        let mut filtered_nodes = nodes;

        // Filtrar os nós com base no `WHERE`
        if let Some(where_clause) = &query.where_clause {
            filtered_nodes = self.apply_where_filter(filtered_nodes, where_clause).await?;
        }

        // Ordenar os nós com base no `ORDER BY`
        if let Some(order_by) = &query.order_by {
            filtered_nodes = self.apply_order_by(filtered_nodes, order_by).await?;
        }

        Ok(filtered_nodes)
    }

    async fn apply_where_filter(&self, nodes: Vec<Node>, where_clause: &str) -> Result<Vec<Node>, GraphError> {
        // Divida a cláusula WHERE em condições separadas por AND ou OR
        let conditions = self.parse_conditions(where_clause)?;

        let filtered_nodes: Vec<Node> = nodes.into_iter().filter(|node| {
            conditions.iter().all(|(attribute, operator, value)| {
                match operator.as_str() {
                    "LIKE" => self.handle_like_filter(node, attribute, value),
                    "=" => self.handle_equal_filter(node, attribute, value),
                    _ => false,
                }
            })
        }).collect();

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

    fn handle_like_filter(&self, node: &Node, attribute: &str, value: &str) -> bool {
        match attribute {
            "label" => node.label.starts_with(value),
            "id" => node.id.to_string().starts_with(value),
            "category" => node.category.starts_with(value),
            _ => false,
        }
    }

    fn handle_equal_filter(&self, node: &Node, attribute: &str, value: &str) -> bool {
        match attribute {
            "label" => node.label == value,
            "id" => node.id.to_string() == value,
            "category" => node.category == value,
            _ => false,
        }
    }

    async fn apply_order_by(&self, nodes: Vec<Node>, order_by: &str) -> Result<Vec<Node>, GraphError> {
        let mut nodes = nodes;

        if order_by == "label" {
            nodes.sort_by(|a, b| a.label.cmp(&b.label));
        }

        Ok(nodes)
    }
}

impl GraphService {
    pub async fn execute_query(&self, query: &Query) -> Result<serde_json::Value, GraphError> {
        let executor = Executor::new(Arc::new(self.clone()));
        executor.execute(query).await
    }
}