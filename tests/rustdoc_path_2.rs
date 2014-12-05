#![feature(globs)]
#![feature(phase, macro_rules)]

#[phase(plugin, link)]
extern crate cayley;

use cayley::Graph;

use cayley::path::Vertex;
use cayley::path::Traversal::*;
use cayley::path::Final::*;

use cayley::selector::NodeSelector::*;
use cayley::selector::PredicateSelector::*;
use cayley::selector::TagSelector::*;

#[test]
fn main() {
    let graph = Graph::default().unwrap();
    graph.find(vertex![ Node("foo")
                        -> As(Tags(vec!("tag-a", "tag-b")))
                        -> OutP(Predicate("follows"))
                        => All ]).unwrap();
}
