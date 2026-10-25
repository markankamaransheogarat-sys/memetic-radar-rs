use crate::models::{GraphNode, NodeId, SocialPlatform};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use std::collections::HashMap;

pub struct SocialGraphEngine {
    graph: DiGraph<GraphNode, f64>,
    node_map: HashMap<NodeId, NodeIndex>,
}

impl Default for SocialGraphEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SocialGraphEngine {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
        }
    }

    /// Add or update a node in the graph
    pub fn upsert_node(&mut self, node: GraphNode) -> NodeIndex {
        if let Some(&idx) = self.node_map.get(&node.id) {
            self.graph[idx] = node;
            idx
        } else {
            let id = node.id.clone();
            let idx = self.graph.add_node(node);
            self.node_map.insert(id, idx);
            idx
        }
    }

    /// Add a transmission edge between two nodes with weight
    pub fn add_transmission(&mut self, from: &NodeId, to: &NodeId, weight: f64) {
        let from_idx = self.get_or_create_stub(from);
        let to_idx = self.get_or_create_stub(to);

        // Check if edge already exists
        if let Some(edge) = self.graph.find_edge(from_idx, to_idx) {
            self.graph[edge] = (self.graph[edge] + weight).min(10.0);
        } else {
            self.graph.add_edge(from_idx, to_idx, weight);
        }
    }

    fn get_or_create_stub(&mut self, id: &NodeId) -> NodeIndex {
        if let Some(&idx) = self.node_map.get(id) {
            idx
        } else {
            let stub = GraphNode {
                id: id.clone(),
                username: id.0.clone(),
                platform: SocialPlatform::Telegram,
                authority_score: 1.0,
                cluster_id: 0,
                followers_count: 1000,
                historical_win_rate: 0.5,
            };
            let idx = self.graph.add_node(stub);
            self.node_map.insert(id.clone(), idx);
            idx
        }
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    pub fn get_node(&self, id: &NodeId) -> Option<&GraphNode> {
        self.node_map.get(id).map(|&idx| &self.graph[idx])
    }

    /// Check if two nodes belong to different community clusters (bridge jump)
    #[allow(dead_code)]
    pub fn is_cross_cluster_hop(&self, from: &NodeId, to: &NodeId) -> bool {
        match (self.node_map.get(from), self.node_map.get(to)) {
            (Some(&f_idx), Some(&t_idx)) => {
                let from_cluster = self.graph[f_idx].cluster_id;
                let to_cluster = self.graph[t_idx].cluster_id;
                from_cluster != to_cluster
            }
            _ => false,
        }
    }

    /// Simplified PageRank / Centrality calculation
    #[allow(dead_code)]
    pub fn compute_centrality(&self, iterations: usize, damping: f64) -> HashMap<NodeId, f64> {
        let n = self.graph.node_count();
        if n == 0 {
            return HashMap::new();
        }

        let mut ranks: HashMap<NodeIndex, f64> = HashMap::new();
        let initial_rank = 1.0 / n as f64;

        for idx in self.graph.node_indices() {
            ranks.insert(idx, initial_rank);
        }

        for _ in 0..iterations {
            let mut next_ranks: HashMap<NodeIndex, f64> = HashMap::new();
            for idx in self.graph.node_indices() {
                let mut incoming_sum = 0.0;
                for edge in self.graph.edges_directed(idx, Direction::Incoming) {
                    let source = edge.source();
                    let out_degree = self.graph.edges_directed(source, Direction::Outgoing).count();
                    if out_degree > 0 {
                        let source_rank = ranks.get(&source).copied().unwrap_or(0.0);
                        incoming_sum += (source_rank / out_degree as f64) * edge.weight();
                    }
                }
                let rank = (1.0 - damping) / n as f64 + damping * incoming_sum;
                next_ranks.insert(idx, rank);
            }
            ranks = next_ranks;
        }

        let mut result = HashMap::new();
        for (idx, rank) in ranks {
            let id = self.graph[idx].id.clone();
            result.insert(id, rank);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_upsert_and_edges() {
        let mut engine = SocialGraphEngine::new();
        let n1 = GraphNode {
            id: NodeId::from("whale_alpha"),
            username: "whale_alpha".to_string(),
            platform: SocialPlatform::Telegram,
            authority_score: 9.5,
            cluster_id: 1,
            followers_count: 50_000,
            historical_win_rate: 0.85,
        };
        let n2 = GraphNode {
            id: NodeId::from("retail_caller"),
            username: "retail_caller".to_string(),
            platform: SocialPlatform::TwitterX,
            authority_score: 4.0,
            cluster_id: 2,
            followers_count: 5_000,
            historical_win_rate: 0.55,
        };

        engine.upsert_node(n1);
        engine.upsert_node(n2);

        assert_eq!(engine.node_count(), 2);
        assert!(engine.is_cross_cluster_hop(&NodeId::from("whale_alpha"), &NodeId::from("retail_caller")));

        engine.add_transmission(&NodeId::from("whale_alpha"), &NodeId::from("retail_caller"), 1.5);
        assert_eq!(engine.edge_count(), 1);

        let centrality = engine.compute_centrality(10, 0.85);
        assert!(centrality.contains_key(&NodeId::from("retail_caller")));
    }
}
