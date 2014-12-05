#![feature(phase, macro_rules)]

#[phase(plugin, link)] extern crate cayley;

use cayley::{Graph, V1};

use cayley::graph::Nodes;

use cayley::path::Vertex;

use cayley::path::Final::All;
use cayley::selector::NodeSelector::AnyNode;

#[test]
fn main() {

    let graph = match Graph::new("localhost", 64210, V1) {

        Err(error) => panic!(error),
        Ok(graph) => graph

    };

    match graph.find(vertex!(AnyNode => All)) {

        Err(error) => panic!(error.to_string()),
        Ok(Nodes(nodes)) => {
            assert!(nodes.len() > 0);
            match nodes.iter().next() {
                Some(first_node) => {
                    // node is a HashMap<String, String>
                    println!("{}", first_node["id".to_string()]);
                },
                None => panic!("first node was not found")
            }
        }

    };

}
