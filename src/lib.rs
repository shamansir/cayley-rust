#![crate_name = "cayley"]

extern crate http;
extern crate url;
extern crate serialize;

pub mod errors;
pub mod selector;
pub mod path;
pub mod graph;


// echo "graph.Vertex('Humphrey Bogart').All()" |
// http --verbose POST localhost:64210/api/v1/query/gremlin Content-Type:text/plain
