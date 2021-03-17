use std::collections::HashMap;

pub type Edges<T: Default> = HashMap<usize, Edge<T>>;

pub struct Graph<T: Default, V: Default> {
    pub nodes: HashMap<String, (usize, V)>,
    edges: Vec<Edges<T>>,
}

pub struct Edge<T: Default> {
    pub from: usize,
    pub to: usize,
    pub data: T,
}

impl<T: Default, V: Default> Graph<T, V> {
    pub fn new() -> Self {
        Graph {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn get_node(&mut self, node_id: &str) -> &mut V {
        self.add_node(node_id, V::default());
        
        &mut self.nodes.get_mut(node_id).unwrap().1
    }

    pub fn add_node(&mut self, node_id: &str, value: V) -> bool {
        if !self.nodes.contains_key(node_id) {
            let id = self.edges.len();
            self.nodes.insert(node_id.into(), (id, value));
            self.edges.push(HashMap::new());

            true
        } else {
            false
        }
    }

    pub fn lookup(&self, node_id: usize) -> &str {
        for (name, (id, _)) in &self.nodes {
            if id == &node_id {
                return name
            }
        }

        panic!("Node does not exists!")
    }

    pub fn add_edge(&mut self, from: &str, to: &str, data: T) {
        self.add_node(from, V::default());
        self.add_node(to, V::default());
        let (from, _) = self.nodes[from];
        let (to, _) = self.nodes[to];

        let edge = Edge { from, to, data };

        self.edges[from].insert(to, edge);
    }

    pub fn edges(&self, node: &str) -> &Edges<T> {
        let (node, _) = self.nodes[node];

        &self.edges[node]
    }

    pub fn edges_mut(&mut self, node: &str) -> &mut Edges<T> {
        let (node, _) = self.nodes[node];

        &mut self.edges[node]
    }

    pub fn edge(&mut self, from: &str, to: &str) -> &mut T {
        self.add_node(from, V::default());
        self.add_node(to, V::default());

        let (from, _) = self.nodes[from];
        let (to, _) = self.nodes[to];

        &mut self.edges[from].entry(to).or_insert(Edge { from, to, data: T::default()}).data
    }
}
