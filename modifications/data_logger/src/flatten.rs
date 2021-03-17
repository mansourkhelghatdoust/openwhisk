use crate::epoch_cache::EpochCache;
use crate::graph::Graph;

#[derive(Default)]
pub struct ActionInfo {
    pub invoke_count: usize,
    pub buffer: EpochCache,
}

#[derive(Default)]
pub struct EdgeInfo {
    pub call_count: usize,
}

type PGraph = Graph<f64, u64>;

fn to_probability_graph(base: &Graph<EdgeInfo, ActionInfo>) -> PGraph {
    let mut graph = Graph::new();
    for (from_name, (_id, value)) in &base.nodes {
        graph.add_node(from_name, value.buffer.current().unwrap_or(0));

        let invoke_count = value.invoke_count as f64;

        for edge in base.edges(from_name).values() {
            let name = base.lookup(edge.to);
            graph.add_edge(from_name, name, edge.data.call_count as f64 / invoke_count);
        }
    }

    graph
}

fn flatten(graph: &mut PGraph) {

}

