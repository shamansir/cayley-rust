extern crate cayley;

use cayley::{Graph, V1};

use cayley::{GraphNode, GraphNodes};

use cayley::path::{Vertex, Query};
use cayley::selector::AnyNode;

#[test]
fn main() {

    let graph = match Graph::new("localhost", 64210, V1) {

        Err(error) => fail!(error),
        Ok(graph) => graph

    };

    match graph.find(Vertex::start(AnyNode).All()) {

        Err(error) => fail!(error.to_string()),
        Ok(GraphNodes(nodes)) => {
            assert!(nodes.len() > 0);
            match nodes.iter().next() {
                Some(&GraphNode(ref first_node)) => {
                    // node is a HashMap<String, String>
                    println!("{:s}", first_node["id".to_string()]);
                },
                None => fail!("first node was not found")
            }
        }

    };

}
