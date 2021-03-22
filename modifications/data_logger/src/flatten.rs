use crate::{epoch_cache::EpochCache, CallGraph, NodeIndicies};
use std::fmt;

use petgraph::prelude::NodeIndex;
use petgraph::stable_graph::EdgeReference;
use petgraph::{algo::*, visit::IntoNodeReferences};
use petgraph::{stable_graph::StableGraph, visit::EdgeRef, Directed};

#[derive(Default, Debug)]
pub struct ActionInfo {
    pub action_name: String,
    pub invoke_count: usize,
    pub buffer: EpochCache,
}

impl fmt::Display for ActionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Default, Debug)]
pub struct EdgeInfo {
    pub call_count: usize,
}

impl fmt::Display for EdgeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

pub fn remove_self_loops(graph: &mut StableGraph<u64, f64>) {
    let tmp = graph.clone();

    // remove self-loops
    for node in tmp.node_indices() {
        // - find node with self-loop
        if graph.contains_edge(node, node) {
            // - remove the edge that causes self-loop
            let e_index = graph.find_edge(node, node).unwrap();
            let e_weight = graph[e_index];
            graph.remove_edge(e_index);

            // Update other edges with new probability
            for edge in tmp.edges(node) {
                graph.update_edge(
                    edge.source(),
                    edge.target(),
                    edge.weight() / (1.0 - e_weight),
                );
            }

            // - add the additional execution time to the node according to paper
            graph[node] += ((e_weight / (1.0 - e_weight)) * (graph[node] as f64)) as u64;
        }
    }
}

#[test]
fn test_removing_self_loops() {
    let mut graph: StableGraph<u64, f64> = StableGraph::new();
    
    let a = graph.add_node(520);
    let b = graph.add_node(150);

    graph.add_edge(a, a, 0.2);
    graph.add_edge(a, b, 0.8);

    remove_self_loops(&mut graph);
    
    assert_eq!(graph[a], 650);
}

pub fn flatten(indices: NodeIndicies, graph: &mut StableGraph<u64, f64>) {
    let new_graph: StableGraph<u64, f64> = StableGraph::new();

    // While graph has cycles or branches
    while is_cyclic_directed(&new_graph)
        || all_simple_paths::<Vec<NodeIndex>, _>(
            &new_graph,
            indices["entry_point"],
            indices["end_point"],
            1,
            None,
        )
        .count()
            > 1
    {

        // remove cycles

        // remove parallels
        // remove branches
    }
}
