#[derive(Debug)]
pub struct Query {
    pub operation: Option<String>,
    pub graph: Option<String>,
    pub from: Option<String>,
    pub where_clause: Option<String>,
    pub order_by: Option<String>,
    pub parts: Vec<String>,
}

impl Query {
    pub fn new() -> Self {
        Query {
            operation: None,
            graph: None,
            from: None,
            where_clause: None,
            order_by: None,
            parts: Vec::new(),
        }
    }

        pub fn parse(&mut self, query_str: &str) {
            let tokens: Vec<&str> = query_str.split_whitespace().collect();
            let mut i = 0;
        
            while i < tokens.len() {
                match tokens[i].to_uppercase().as_str() {
                    "SELECT" | "UPDATE" | "DELETE" | "INSERT" => {
                        self.operation = Some(tokens[i].to_string());
                        self.parts.push(tokens[i].to_string());
                        i += 1;
                    }
                    "GRAPH" => {
                        if i + 1 < tokens.len() {
                            self.graph = Some(tokens[i + 1].to_string());
                            self.parts.push(tokens[i].to_string());
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "FROM" => {
                        if i + 1 < tokens.len() {
                            self.from = Some(tokens[i + 1].to_string());
                            self.parts.push(tokens[i].to_string());
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    "WHERE" => {
                    if i + 1 < tokens.len() {
                        let where_clause = tokens[i + 1..]
                            .iter()
                            .take_while(|&&token| token.to_uppercase() != "ORDER" && token != "'")
                            .cloned()
                            .collect::<Vec<&str>>()
                            .join(" ");
                        self.where_clause = Some(where_clause.clone());  // Usando clone aqui
                        self.parts.push(tokens[i].to_string());
                        i += where_clause.split_whitespace().count() + 1;
                    } else {
                        i += 1;
                    }
                }
                    "ORDER" => {
                        if i + 1 < tokens.len() && tokens[i + 1].to_uppercase() == "BY" {
                            let order_by = tokens[i + 2..]
                                .iter()
                                .map(|&token| token.to_string())
                                .collect::<Vec<String>>()
                                .join(" ");
                            self.order_by = Some(order_by.clone());
                            self.parts.push("ORDER BY".to_string());
                            self.parts.push(order_by);
                            break;
                        }
                    }
                    _ => {
                        i += 1;
                    }
                }
            }
        }
        
}

