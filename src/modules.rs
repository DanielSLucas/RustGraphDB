// o básico -> Node, Edge e Graph (métodos para lidar com Nodes e Edges dentro de um grafo)
pub mod graph; 

// qualquer coisa que ajude a lidar com os Grafos (tipos os métodos de printar)
pub mod graph_utils; 

// deve implementar uma maneira de persistir os grafos (em memória ou disco)
pub mod persistence;

// deve implementar uma interface para:
// - fazer buscas (bfd, dfs, Dijkstra...)
// - buscar por padrões padrões ou subgrafos
// - funções de grafos como encontrar vizinho, grau de um nó...
pub mod query_engine;
pub mod api;

// deve permitir buscar Node ou Edges, por um index, tornando a busca mais rápida
// pub mod index;

// deve lidar com acesso concorrente (paralelo) aos grafos (forma segura de acesso a dados compartilhados)
// pub mod concurrency;

// maneira de interagir com o banco, pode ser CLI ou API Rest...
// pub mod rust_db_api;

// Falta um módulo de testes e benchmarking
// e um de algortimos comuns usados em grafos