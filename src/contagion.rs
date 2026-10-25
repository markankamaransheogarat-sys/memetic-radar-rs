use crate::models::{ContagionMetrics, NodeId, SignalSeverity};
use chrono::{DateTime, Duration, Utc};
use std::collections::{HashMap, HashSet};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InfectionEvent {
    pub node_id: NodeId,
    pub cluster_id: usize,
    pub timestamp: DateTime<Utc>,
    pub is_bridge_hop: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NarrativeTracker {
    pub ticker: String,
    pub contract_address: Option<String>,
    pub events: Vec<InfectionEvent>,
    pub first_seen: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub infected_nodes: HashSet<NodeId>,
    pub infected_clusters: HashSet<usize>,
    pub bridge_hop_count: usize,
}

impl NarrativeTracker {
    pub fn new(ticker: String, contract: Option<String>, initial_node: NodeId, cluster_id: usize) -> Self {
        let now = Utc::now();
        let mut infected_nodes = HashSet::new();
        let mut infected_clusters = HashSet::new();
        infected_nodes.insert(initial_node.clone());
        infected_clusters.insert(cluster_id);

        let initial_event = InfectionEvent {
            node_id: initial_node,
            cluster_id,
            timestamp: now,
            is_bridge_hop: false,
        };

        Self {
            ticker,
            contract_address: contract,
            events: vec![initial_event],
            first_seen: now,
            last_updated: now,
            infected_nodes,
            infected_clusters,
            bridge_hop_count: 0,
        }
    }

    pub fn record_infection(&mut self, node_id: NodeId, cluster_id: usize, is_bridge: bool) {
        let now = Utc::now();
        self.last_updated = now;
        self.infected_nodes.insert(node_id.clone());
        self.infected_clusters.insert(cluster_id);
        if is_bridge {
            self.bridge_hop_count += 1;
        }

        self.events.push(InfectionEvent {
            node_id,
            cluster_id,
            timestamp: now,
            is_bridge_hop: is_bridge,
        });
    }

    pub fn compute_metrics(&self, window_seconds: i64) -> ContagionMetrics {
        let now = Utc::now();
        let window_start = now - Duration::seconds(window_seconds);
        let mid_window_start = now - Duration::seconds(window_seconds / 2);

        // Count events in recent window
        let recent_events: Vec<&InfectionEvent> = self
            .events
            .iter()
            .filter(|e| e.timestamp >= window_start)
            .collect();

        // T1 = first half of window, T2 = second half of window
        let count_t1 = recent_events
            .iter()
            .filter(|e| e.timestamp < mid_window_start)
            .count() as f64;
        let count_t2 = recent_events
            .iter()
            .filter(|e| e.timestamp >= mid_window_start)
            .count() as f64;

        let delta_t_sec = (window_seconds / 2) as f64;
        let v1 = if delta_t_sec > 0.0 { count_t1 / delta_t_sec } else { 0.0 };
        let v2 = if delta_t_sec > 0.0 { count_t2 / delta_t_sec } else { 0.0 };

        let velocity = v2;
        let acceleration = if delta_t_sec > 0.0 { (v2 - v1) / delta_t_sec } else { 0.0 };

        // Basic reproduction number R0: Ratio of new infections to generation capacity
        let r0 = if count_t1 > 0.0 {
            (count_t2 / count_t1).max(0.1)
        } else if count_t2 > 1.0 {
            count_t2 * 1.5
        } else {
            1.0
        };

        let percolation_threshold_exceeded =
            r0 >= 1.5 && self.bridge_hop_count >= 1 && self.infected_clusters.len() >= 2;

        ContagionMetrics {
            r0_reproduction_rate: (r0 * 100.0).round() / 100.0,
            infection_velocity: (velocity * 60.0 * 100.0).round() / 100.0, // mentions per minute
            acceleration: (acceleration * 3600.0 * 100.0).round() / 100.0,
            infected_nodes_count: self.infected_nodes.len(),
            active_clusters_count: self.infected_clusters.len(),
            cross_cluster_bridge_hits: self.bridge_hop_count,
            percolation_threshold_exceeded,
        }
    }

    pub fn classify_severity(&self, metrics: &ContagionMetrics) -> SignalSeverity {
        if metrics.r0_reproduction_rate > 3.0 && metrics.cross_cluster_bridge_hits >= 3 {
            SignalSeverity::ParabolicBlowoff
        } else if metrics.percolation_threshold_exceeded || (metrics.r0_reproduction_rate >= 1.8 && metrics.active_clusters_count >= 2) {
            SignalSeverity::ExponentialBreakout
        } else if metrics.r0_reproduction_rate < 0.8 && metrics.infected_nodes_count > 10 {
            SignalSeverity::ExhaustionFading
        } else {
            SignalSeverity::EarlySeed
        }
    }
}

pub struct ContagionEngine {
    trackers: HashMap<String, NarrativeTracker>,
    analysis_window_seconds: i64,
}

impl Default for ContagionEngine {
    fn default() -> Self {
        Self::new(600) // 10 minutes default sliding analysis window
    }
}

impl ContagionEngine {
    pub fn new(analysis_window_seconds: i64) -> Self {
        Self {
            trackers: HashMap::new(),
            analysis_window_seconds,
        }
    }

    pub fn ingest_mention(
        &mut self,
        ticker: &str,
        contract: Option<String>,
        node_id: NodeId,
        cluster_id: usize,
        is_bridge: bool,
    ) -> (ContagionMetrics, SignalSeverity) {
        let key = ticker.to_uppercase();
        let tracker = self
            .trackers
            .entry(key.clone())
            .or_insert_with(|| NarrativeTracker::new(key, contract, node_id.clone(), cluster_id));

        tracker.record_infection(node_id, cluster_id, is_bridge);
        let metrics = tracker.compute_metrics(self.analysis_window_seconds);
        let severity = tracker.classify_severity(&metrics);
        (metrics, severity)
    }

    pub fn get_tracker(&self, ticker: &str) -> Option<&NarrativeTracker> {
        self.trackers.get(&ticker.to_uppercase())
    }

    pub fn active_narratives_count(&self) -> usize {
        self.trackers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contagion_model_exponential_breakout() {
        let mut engine = ContagionEngine::new(300);

        // Seed mention
        engine.ingest_mention("PEPE2", None, NodeId::from("alpha_1"), 1, false);

        // Rapid subsequent infections jumping across clusters
        engine.ingest_mention("PEPE2", None, NodeId::from("alpha_2"), 1, false);
        engine.ingest_mention("PEPE2", None, NodeId::from("caller_1"), 2, true);
        engine.ingest_mention("PEPE2", None, NodeId::from("caller_2"), 2, false);
        let (metrics, severity) = engine.ingest_mention("PEPE2", None, NodeId::from("influencer_3"), 3, true);

        assert!(metrics.infected_nodes_count >= 4);
        assert!(metrics.active_clusters_count >= 2);
        assert!(metrics.cross_cluster_bridge_hits >= 2);
        assert_eq!(severity, SignalSeverity::ExponentialBreakout);
    }
}
