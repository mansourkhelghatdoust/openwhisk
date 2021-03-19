use crate::{epoch_cache::EpochCache, CallGraph, NodeIndicies};
use std::fmt;

use petgraph::{Directed, stable_graph::StableGraph, visit::EdgeRef};
use petgraph::prelude::NodeIndex;
use petgraph::algo::*;
use petgraph::stable_graph::EdgeReference;

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


pub fn flatten(indices: NodeIndicies, graph: &mut StableGraph<u64, f64>) {
    let new_graph: StableGraph<u64, f64> = StableGraph::new();


    for &node in indices.values() {

        // - find node with self-loop
        if graph.contains_edge(node, node) {

            // - remove the edge that causes self-loop
            let e_index = graph.find_edge(node, node).unwrap();
            let e_weight = graph[e_index];
            graph.remove_edge(e_index);


            let tmp = graph.clone();

            // Update other edges with new probability
            for edge in tmp.edges(node) {
                graph.update_edge(edge.source(), edge.target(),
                edge.weight() / (1.0 - e_weight));
            }

            // - add the additional execution time to the node according
            
        }
    }


    // While graph has cycles or branches
    while is_cyclic_directed(&new_graph) ||
     all_simple_paths::<Vec<NodeIndex>, _>(&new_graph, 
        indices["entry_point"],
        indices["end_point"],
        1,
        None
    ).count() > 1 {



        // remove cycles

        // remove parallels
        // remove branches


    }

}

