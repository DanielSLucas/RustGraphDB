use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Query {
    pub operation: Option<Operation>,
    pub from_clause: Option<String>,
    pub match_clause: Option<String>,
    pub where_clause: Option<WhereClause>,
    pub return_clause: Option<String>,
    pub set_clause: Option<String>,
    pub order_by: Option<String>,
    pub graph_pattern: Option<GraphPattern>,
}

#[derive(Debug, Clone)]
pub struct WhereClause {
    pub conditions: Vec<Condition>,
}

#[derive(Debug, Clone)]
pub struct Condition {
    pub field: String,
    pub operator: String,
    pub value: Value,
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
}

#[derive(Debug, Clone)]
pub struct GraphPattern {
    pub graph_name: String,
    pub edge_var: String,
    pub edge_label: String,
    pub direction: Direction,
}

#[derive(Debug, Clone)]
pub enum Direction {
    Outgoing,
    Incoming,
    Bidirectional,
}

#[derive(Debug, Clone)]
pub enum Operation {
    Match,
    Create,
    Delete,
    Set,
}

impl FromStr for Operation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "MATCH" => Ok(Operation::Match),
            "CREATE" => Ok(Operation::Create),
            "DELETE" => Ok(Operation::Delete),
            "SET" => Ok(Operation::Set),
            _ => Err(format!("Unknown operation: {}", s)),
        }
    }
}

#[derive(Debug)]
pub enum QueryError {
    InvalidSyntax(String),
    MissingRequiredClause(String),
    InvalidGraphPattern(String),
    ParseError(String),
}

impl Query {
    pub fn new() -> Self {
        Query {
            operation: None,
            from_clause: None,
            match_clause: None,
            where_clause: None,
            return_clause: None,
            set_clause: None,
            order_by: None,
            graph_pattern: None,
        }
    }

    pub fn parse(&mut self, query_str: &str) -> Result<(), QueryError> {
        let tokens: Vec<&str> = query_str.split_whitespace().collect();
        let mut i = 0;

        while i < tokens.len() {
            match tokens[i].to_uppercase().as_str() {
                "FROM" => {
                    i = self.parse_from_clause(&tokens, i)?;
                }
                "MATCH" => {
                    i = self.parse_match_clause(&tokens, i)?;
                }
                "WHERE" => {
                    i = self.parse_where_clause(&tokens, i)?;
                }
                "RETURN" => {
                    i = self.parse_return_clause(&tokens, i)?;
                }
                _ => i += 1,
            }
        }

        self.validate()?;
        Ok(())
    }

    fn parse_from_clause(&mut self, tokens: &[&str], i: usize) -> Result<usize, QueryError> {
        if i + 1 >= tokens.len() {
            return Err(QueryError::InvalidSyntax("FROM clause requires a target".to_string()));
        }
        self.from_clause = Some(tokens[i + 1].to_string());
        Ok(i + 2)
    }

    fn parse_match_clause(&mut self, tokens: &[&str], i: usize) -> Result<usize, QueryError> {
        if i + 1 >= tokens.len() {
            return Err(QueryError::InvalidSyntax("MATCH clause requires a pattern".to_string()));
        }

        self.operation = Some(Operation::Match);
        let pattern = self.parse_graph_pattern(&tokens[i + 1..])?;
        self.graph_pattern = Some(pattern);
        
        // Skip the parsed pattern tokens
        Ok(i + self.count_pattern_tokens(&tokens[i + 1..]))
    }
        fn parse_where_clause(&mut self, tokens: &[&str], i: usize) -> Result<usize, QueryError> {
            if i + 1 >= tokens.len() {
                return Err(QueryError::InvalidSyntax("WHERE clause requires conditions".to_string()));
            }
    
            let mut conditions = Vec::new();
            let mut current_pos = i + 1;
    
            // Junta todos os tokens até RETURN em uma string
            let where_str: String = tokens[current_pos..]
                .iter()
                .take_while(|&&t| t.to_uppercase() != "RETURN")
                .map(|&s| s.to_string()) // Convert each &str to a String
                .collect::<Vec<String>>()
                .join(" ");

    
            // Divide por AND para separar múltiplas condições
            for condition_str in where_str.split(" AND ") {
                if let Some(condition) = self.parse_single_condition(condition_str)? {
                    conditions.push(condition);
                }
            }
    
            self.where_clause = Some(WhereClause { conditions });
            
            // Avança até encontrar RETURN ou fim dos tokens
            while current_pos < tokens.len() && tokens[current_pos].to_uppercase() != "RETURN" {
                current_pos += 1;
            }
            
            Ok(current_pos)
        }
    
        fn parse_single_condition(&self, condition_str: &str) -> Result<Option<Condition>, QueryError> {
            // Divide a string da condição em partes
            let parts: Vec<&str> = condition_str.trim().split_whitespace().collect();
            
            if parts.len() < 3 {
                return Ok(None);
            }
    
            // Extrai o campo (removendo 'e.properties.' se presente)
            let field = parts[0]
                .trim_start_matches("e.properties.")
                .to_string();
    
            // Extrai o operador
            let operator = parts[1].to_string();
    
            // Extrai o valor (pode estar entre aspas ou ser um número)
            let value_str = parts[2].trim_matches('\'');
    
            let value = if let Ok(num) = value_str.parse::<f64>() {
                Value::Number(num)
            } else if value_str.starts_with('\'') && value_str.ends_with('\'') {
                Value::String(value_str[1..value_str.len()-1].to_string())
            } else if let Ok(bool_val) = value_str.parse::<bool>() {
                Value::Boolean(bool_val)
            } else {
                // Tenta converter para número mesmo se não estiver entre aspas
                if let Ok(num) = value_str.parse::<f64>() {
                    Value::Number(num)
                } else {
                    Value::String(value_str.to_string())
                }
            };
    
            Ok(Some(Condition {
                field,
                operator,
                value,
            }))
        }
    

    fn parse_return_clause(&mut self, tokens: &[&str], i: usize) -> Result<usize, QueryError> {
        if i + 1 >= tokens.len() {
            return Err(QueryError::InvalidSyntax("RETURN clause requires a value".to_string()));
        }
        
        let return_value = tokens[i + 1].to_string();
        self.return_clause = Some(return_value);
        Ok(tokens.len()) // Return clause is always last
    }

    fn parse_graph_pattern(&self, tokens: &[&str]) -> Result<GraphPattern, QueryError> {
        let pattern = tokens.join(" ");
        
        // Parse graph name
        let graph_name = self.extract_graph_name(&pattern)?;
        
        // Parse edge information
        let (edge_var, edge_label, direction) = self.extract_edge_info(&pattern)?;

        Ok(GraphPattern {
            graph_name,
            edge_var,
            edge_label,
            direction,
        })
    }

    fn extract_graph_name(&self, pattern: &str) -> Result<String, QueryError> {
        // Tenta encontrar a parte "name: 'algo'" no padrão
        if let Some(start) = pattern.find("name: '") {
            if let Some(end) = pattern[start + 7..].find('\'') {
                // Retorna o nome do grafo entre as aspas simples
                return Ok(pattern[start + 7..start + 7 + end].to_string());
            }
        }
    
        // Caso não encontre o nome corretamente
        Err(QueryError::InvalidGraphPattern("Missing or invalid graph name".to_string()))
    }
    

    fn extract_edge_info(&self, pattern: &str) -> Result<(String, String, Direction), QueryError> {
        if let Some(start) = pattern.find('[') {
            if let Some(end) = pattern[start..].find(']') {
                let edge_part = &pattern[start + 1..start + end];
                let parts: Vec<&str> = edge_part.split(':').collect();
                
                if parts.len() != 2 {
                    return Err(QueryError::InvalidGraphPattern("Invalid edge pattern format".to_string()));
                }

                let direction = if pattern.contains("->") {
                    Direction::Outgoing
                } else if pattern.contains("<-") {
                    Direction::Incoming
                } else {
                    Direction::Bidirectional
                };

                Ok((
                    parts[0].trim().to_string(),
                    parts[1].trim().to_string(),
                    direction,
                ))
            } else {
                Err(QueryError::InvalidGraphPattern("Missing closing bracket in edge pattern".to_string()))
            }
        } else {
            Err(QueryError::InvalidGraphPattern("Missing edge pattern".to_string()))
        }
    }

    fn parse_condition(&self, tokens: &[&str]) -> Result<Option<Condition>, QueryError> {
        if tokens.len() < 3 {
            return Ok(None);
        }

        let field = tokens[0].to_string();
        let operator = tokens[1].to_string();
        let value_str = tokens[2].to_string();

        let value = if let Ok(num) = value_str.parse::<f64>() {
            Value::Number(num)
        } else if value_str.starts_with('\'') && value_str.ends_with('\'') {
            Value::String(value_str[1..value_str.len()-1].to_string())
        } else if let Ok(bool_val) = value_str.parse::<bool>() {
            Value::Boolean(bool_val)
        } else {
            return Err(QueryError::ParseError(format!("Invalid value format: {}", value_str)));
        };

        Ok(Some(Condition {
            field,
            operator,
            value,
        }))
    }

    fn count_pattern_tokens(&self, tokens: &[&str]) -> usize {
        // Count tokens until we hit the next clause
        tokens.iter()
            .take_while(|&&token| !matches!(token.to_uppercase().as_str(), "WHERE" | "RETURN"))
            .count()
    }

    fn validate(&self) -> Result<(), QueryError> {
        if self.operation.is_none() {
            return Err(QueryError::MissingRequiredClause("Missing operation".to_string()));
        }
        if self.return_clause.is_none() {
            return Err(QueryError::MissingRequiredClause("Missing RETURN clause".to_string()));
        }
        Ok(())
    }
}